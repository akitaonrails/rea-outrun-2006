//! The community HD texture pack, downloaded when the player asks for it.
//!
//! One job runs on a worker thread: download (resumable, checked by SHA-256
//! while it streams), unpack into a folder of its own, mark it complete. The
//! window reads [`Job::phase`] to draw the progress. Fetching sits behind
//! [`Fetch`], so all of this is tested without a network; Windows (and
//! Wine) use WinHTTP.

use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

const PIN: &str = include_str!("../../../packaging/hd-pack.txt");
/// Written last into an unpacked pack, holding its SHA-256.
const MARKER: &str = ".complete";
/// The packs' folder under the launcher's home folder.
pub const FOLDER: &str = "hd-pack";
const CHUNK: usize = 1 << 20;

/// A pack release: where it is and what it must be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    pub url: String,
    pub sha256: String,
    /// The zip's size in bytes.
    pub size: u64,
    /// The unpacked files' size in bytes.
    pub unpacked: u64,
}

impl Pack {
    /// The pack `packaging/hd-pack.txt` pins.
    pub fn pinned() -> Pack {
        Pack::parse(PIN).expect("packaging/hd-pack.txt holds URL SHA-256 size unpacked")
    }

    fn parse(text: &str) -> Option<Pack> {
        let line = text
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with('#'))?;
        let mut field = line.split_whitespace();
        let pack = Pack {
            url: field.next()?.to_string(),
            sha256: field.next()?.to_ascii_lowercase(),
            size: field.next()?.parse().ok()?,
            unpacked: field.next()?.parse().ok()?,
        };
        (pack.sha256.len() == 64).then_some(pack)
    }

    /// The release's name, e.g. `OR2-HD-GUI-v0.26.09a`.
    pub fn name(&self) -> &str {
        let file = self.url.rsplit('/').next().unwrap_or(&self.url);
        file.strip_suffix(".zip").unwrap_or(file)
    }

    /// Where this pack is unpacked under the launcher's home folder.
    pub fn dir(&self, home: &Path) -> PathBuf {
        home.join(FOLDER).join(self.name())
    }

    /// Free space the rest of the download and the unpacked files need,
    /// with a little headroom.
    pub fn space_needed(&self, downloaded: u64) -> u64 {
        self.size.saturating_sub(downloaded) + self.unpacked + self.unpacked / 20
    }
}

/// The unpacked pack, when a complete install of it is there.
pub fn installed(pack: &Pack, home: &Path) -> Option<PathBuf> {
    let dir = pack.dir(home);
    let mark = fs::read_to_string(dir.join(MARKER)).ok()?;
    (mark.trim() == pack.sha256).then_some(dir)
}

/// Bytes of an interrupted download waiting to be resumed.
pub fn partial(pack: &Pack, home: &Path) -> u64 {
    fs::metadata(part_path(pack, home))
        .map(|m| m.len())
        .unwrap_or(0)
}

/// Deletes the pack and anything left of its download.
pub fn remove(pack: &Pack, home: &Path) -> Result<(), String> {
    let dir = pack.dir(home);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("cannot remove {}: {e}", dir.display()))?;
    }
    for file in [part_path(pack, home), zip_path(pack, home)] {
        let _ = fs::remove_file(file);
    }
    Ok(())
}

fn part_path(pack: &Pack, home: &Path) -> PathBuf {
    home.join(FOLDER).join(format!("{}.zip.part", pack.name()))
}

fn zip_path(pack: &Pack, home: &Path) -> PathBuf {
    home.join(FOLDER).join(format!("{}.zip", pack.name()))
}

/// Where a job is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Starting,
    Downloading { done: u64, total: u64 },
    Verifying,
    Unpacking { done: u64, total: u64 },
    Done(PathBuf),
    Failed(String),
    Cancelled,
}

impl Phase {
    pub fn running(&self) -> bool {
        matches!(
            self,
            Phase::Starting
                | Phase::Downloading { .. }
                | Phase::Verifying
                | Phase::Unpacking { .. }
        )
    }
}

/// A download shared between the worker thread and the window.
#[derive(Debug)]
pub struct Job {
    phase: Mutex<Phase>,
    cancel: AtomicBool,
}

impl Default for Job {
    fn default() -> Job {
        Job {
            phase: Mutex::new(Phase::Starting),
            cancel: AtomicBool::new(false),
        }
    }
}

impl Job {
    pub fn phase(&self) -> Phase {
        self.phase.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Asks the worker to stop; a download stopped this way resumes later.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn set(&self, phase: Phase) {
        *self.phase.lock().unwrap_or_else(|e| e.into_inner()) = phase;
    }
}

/// An HTTP GET that streams its body.
pub trait Fetch {
    /// Streams `url` from byte `from` (a range request when above 0).
    /// `start` is told whether the body begins at `from` (true) or at byte
    /// 0 because the server ignored the range (false). `chunk` receives the
    /// body and returns false to stop.
    fn fetch(
        &self,
        url: &str,
        from: u64,
        start: &mut dyn FnMut(bool),
        chunk: &mut dyn FnMut(&[u8]) -> bool,
    ) -> Result<(), String>;
}

/// Downloads and unpacks `pack` under `home`, reporting through `job`.
/// `free` is the free space on that drive, when known. Returns (and leaves
/// in `job`) the final phase.
pub fn install(pack: &Pack, home: &Path, free: Option<u64>, fetch: &dyn Fetch, job: &Job) -> Phase {
    let phase = match run(pack, home, free, fetch, job) {
        Ok(dir) => Phase::Done(dir),
        Err(_) if job.cancelled() => Phase::Cancelled,
        Err(e) => Phase::Failed(e),
    };
    job.set(phase.clone());
    phase
}

fn run(
    pack: &Pack,
    home: &Path,
    free: Option<u64>,
    fetch: &dyn Fetch,
    job: &Job,
) -> Result<PathBuf, String> {
    if let Some(dir) = installed(pack, home) {
        return Ok(dir);
    }
    let root = home.join(FOLDER);
    fs::create_dir_all(&root).map_err(|e| format!("cannot create {}: {e}", root.display()))?;
    let zip = zip_path(pack, home);
    // A download finished before an interrupted unpack is used again.
    let ready = zip.is_file() && {
        job.set(Phase::Verifying);
        file_sha256(&zip).is_ok_and(|digest| digest == pack.sha256)
    };
    if !ready {
        let part = part_path(pack, home);
        download(pack, &part, free, fetch, job)?;
        fs::rename(&part, &zip).map_err(|e| format!("cannot keep the download: {e}"))?;
    }
    let dir = unpack(pack, &zip, home, job)?;
    let _ = fs::remove_file(&zip);
    remove_other_packs(&root, pack);
    Ok(dir)
}

/// What the download writes as it streams.
struct Sink {
    file: File,
    hasher: Sha256,
    done: u64,
    error: Option<String>,
}

fn download(
    pack: &Pack,
    part: &Path,
    free: Option<u64>,
    fetch: &dyn Fetch,
    job: &Job,
) -> Result<(), String> {
    job.set(Phase::Starting);
    let mut have = fs::metadata(part).map(|m| m.len()).unwrap_or(0);
    if have > pack.size {
        have = 0;
    }
    if let Some(free) = free {
        let need = pack.space_needed(have);
        if free < need {
            return Err(format!(
                "Not enough free space: the pack needs {:.1} GB and {:.1} GB are free.",
                gb(need),
                gb(free)
            ));
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(part)
        .map_err(|e| format!("cannot write {}: {e}", part.display()))?;
    file.set_len(have).map_err(|e| e.to_string())?;
    // The bytes already downloaded count towards the digest.
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; CHUNK];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let sink = RefCell::new(Sink {
        file,
        hasher,
        done: have,
        error: None,
    });
    job.set(Phase::Downloading {
        done: have,
        total: pack.size,
    });
    let result = fetch.fetch(
        &pack.url,
        have,
        &mut |resumed| {
            if !resumed {
                // The server sent the whole file: start over.
                let mut s = sink.borrow_mut();
                s.hasher = Sha256::new();
                s.done = 0;
                if let Err(e) = s
                    .file
                    .set_len(0)
                    .and_then(|_| s.file.seek(SeekFrom::Start(0)))
                {
                    s.error = Some(e.to_string());
                }
            }
        },
        &mut |bytes| {
            if job.cancelled() {
                return false;
            }
            let mut s = sink.borrow_mut();
            if s.error.is_some() {
                return false;
            }
            if let Err(e) = s.file.write_all(bytes) {
                s.error = Some(format!("cannot write the download: {e}"));
                return false;
            }
            s.hasher.update(bytes);
            s.done += bytes.len() as u64;
            job.set(Phase::Downloading {
                done: s.done,
                total: pack.size,
            });
            true
        },
    );
    let sink = sink.into_inner();
    if job.cancelled() {
        return Err("cancelled".into());
    }
    if let Some(e) = sink.error {
        return Err(e);
    }
    result.map_err(|e| format!("The download stopped ({e}). Press Download to resume."))?;
    drop(sink.file);
    if sink.done != pack.size {
        return Err(format!(
            "The download ended early ({:.0} of {:.0} MB). Press Download to resume.",
            mb(sink.done),
            mb(pack.size)
        ));
    }
    job.set(Phase::Verifying);
    if to_hex(&sink.hasher.finalize()) != pack.sha256 {
        let _ = fs::remove_file(part);
        return Err("The download arrived damaged (its checksum doesn't match). Press Download to try again.".into());
    }
    Ok(())
}

/// Unpacks into a temporary folder and moves it into place only when every
/// file is out, so a half-unpacked pack is never used.
fn unpack(pack: &Pack, zip: &Path, home: &Path, job: &Job) -> Result<PathBuf, String> {
    let dir = pack.dir(home);
    let tmp = home.join(FOLDER).join(format!("{}.unpacking", pack.name()));
    for stale in [&tmp, &dir] {
        if stale.exists() {
            fs::remove_dir_all(stale)
                .map_err(|e| format!("cannot clear {}: {e}", stale.display()))?;
        }
    }
    let file = File::open(zip).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("the pack's zip can't be read: {e}"))?;
    let mut total = 0;
    for i in 0..archive.len() {
        total += archive.by_index_raw(i).map_err(|e| e.to_string())?.size();
    }
    let mut done = 0;
    job.set(Phase::Unpacking { done, total });
    let mut buffer = vec![0; CHUNK];
    for i in 0..archive.len() {
        if job.cancelled() {
            return Err("cancelled".into());
        }
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("the pack's zip can't be read: {e}"))?;
        // Names that would land outside the folder are skipped.
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let out = tmp.join(name);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut target =
            File::create(&out).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
        loop {
            let n = entry
                .read(&mut buffer)
                .map_err(|e| format!("the pack's zip is damaged: {e}"))?;
            if n == 0 {
                break;
            }
            target
                .write_all(&buffer[..n])
                .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
            done += n as u64;
            job.set(Phase::Unpacking { done, total });
        }
    }
    fs::rename(&tmp, &dir).map_err(|e| format!("cannot finish {}: {e}", dir.display()))?;
    fs::write(dir.join(MARKER), &pack.sha256).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Older packs this launcher installed (they carry the marker), and
/// leftovers of their downloads, once a newer one is in place.
fn remove_other_packs(root: &Path, pack: &Pack) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(pack.name()) {
            continue;
        }
        if path.join(MARKER).is_file() {
            let _ = fs::remove_dir_all(&path);
        } else if name.ends_with(".zip.part") || name.ends_with(".zip") {
            let _ = fs::remove_file(&path);
        }
    }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; CHUNK];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(to_hex(&hasher.finalize()))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn mb(bytes: u64) -> f64 {
    bytes as f64 / 1_000_000.0
}

pub fn gb(bytes: u64) -> f64 {
    bytes as f64 / 1_000_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::Cursor;

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("or-hdpack-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// Bytes that don't compress, so the zip stays larger than the cuts.
    fn texture() -> Vec<u8> {
        let mut x: u32 = 0x2006;
        (0..5000)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (x >> 24) as u8
            })
            .collect()
    }

    /// A zip laid out like the real pack, and a pin for it.
    fn pack_zip() -> (Pack, Vec<u8>) {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let deflated = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.add_directory("textures/load/spr_a/", deflated).unwrap();
        zip.start_file("textures/load/spr_a/a_64x64.dds", deflated)
            .unwrap();
        zip.write_all(&texture()).unwrap();
        zip.start_file("Readme.txt", deflated).unwrap();
        zip.write_all(b"credits").unwrap();
        // A hostile name must not escape the folder.
        zip.start_file("../escape.txt", deflated).unwrap();
        zip.write_all(b"no").unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        let pack = Pack {
            url: "https://example.org/r/OR2-HD-GUI-vtest.zip".into(),
            sha256: to_hex(&Sha256::digest(&bytes)),
            size: bytes.len() as u64,
            unpacked: 5009,
        };
        (pack, bytes)
    }

    /// Serves `body`, honouring ranges unless told not to, optionally
    /// stopping after some bytes.
    struct Fake {
        body: Vec<u8>,
        ranges: bool,
        stop_after: Option<usize>,
        asked_from: Cell<Option<u64>>,
    }

    impl Fake {
        fn new(body: &[u8]) -> Fake {
            Fake {
                body: body.to_vec(),
                ranges: true,
                stop_after: None,
                asked_from: Cell::new(None),
            }
        }
    }

    impl Fetch for Fake {
        fn fetch(
            &self,
            _url: &str,
            from: u64,
            start: &mut dyn FnMut(bool),
            chunk: &mut dyn FnMut(&[u8]) -> bool,
        ) -> Result<(), String> {
            self.asked_from.set(Some(from));
            let at = if self.ranges { from as usize } else { 0 };
            start(self.ranges || from == 0);
            let end = self
                .stop_after
                .map_or(self.body.len(), |n| (at + n).min(self.body.len()));
            for piece in self.body[at..end].chunks(700) {
                if !chunk(piece) {
                    return Err("stopped".into());
                }
            }
            match self.stop_after {
                Some(_) => Err("connection reset".into()),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn the_pinned_pack_parses() {
        let pack = Pack::pinned();
        assert_eq!(pack.name(), "OR2-HD-GUI-v0.26.09a");
        assert!(
            pack.url
                .starts_with("https://github.com/envido32/OR2006Sprites/releases/download/")
        );
        assert_eq!(pack.sha256.len(), 64);
        assert!(pack.size > 300_000_000 && pack.unpacked > 3_000_000_000);
        assert!(Pack::parse("# only a comment\n").is_none());
        assert!(Pack::parse("https://x/y.zip abc 1 2\n").is_none());
    }

    #[test]
    fn downloads_unpacks_and_marks_complete() {
        let home = temp("full");
        let (pack, bytes) = pack_zip();
        let job = Job::default();
        let phase = install(&pack, &home, None, &Fake::new(&bytes), &job);
        let dir = pack.dir(&home);
        assert_eq!(phase, Phase::Done(dir.clone()));
        assert_eq!(job.phase(), phase);
        assert_eq!(
            fs::read(dir.join("textures/load/spr_a/a_64x64.dds")).unwrap(),
            texture()
        );
        assert!(!home.join(FOLDER).join("escape.txt").exists());
        assert!(!home.join("escape.txt").exists());
        assert_eq!(installed(&pack, &home), Some(dir.clone()));
        // The zip is gone once unpacked; a second run finds it installed.
        assert!(!zip_path(&pack, &home).exists());
        let again = Fake::new(&bytes);
        assert_eq!(
            install(&pack, &home, None, &again, &Job::default()),
            Phase::Done(dir)
        );
        assert_eq!(again.asked_from.get(), None);
        remove(&pack, &home).unwrap();
        assert_eq!(installed(&pack, &home), None);
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn an_interrupted_download_resumes() {
        let home = temp("resume");
        let (pack, bytes) = pack_zip();
        let mut cut = Fake::new(&bytes);
        cut.stop_after = Some(1000);
        let first = install(&pack, &home, None, &cut, &Job::default());
        assert!(
            matches!(first, Phase::Failed(ref e) if e.contains("resume")),
            "{first:?}"
        );
        assert_eq!(partial(&pack, &home), 1000);

        let rest = Fake::new(&bytes);
        let phase = install(&pack, &home, None, &rest, &Job::default());
        assert_eq!(rest.asked_from.get(), Some(1000));
        assert_eq!(phase, Phase::Done(pack.dir(&home)));
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn a_server_that_ignores_ranges_starts_over() {
        let home = temp("norange");
        let (pack, bytes) = pack_zip();
        fs::create_dir_all(home.join(FOLDER)).unwrap();
        fs::write(part_path(&pack, &home), &bytes[..1000]).unwrap();
        let mut whole = Fake::new(&bytes);
        whole.ranges = false;
        assert_eq!(
            install(&pack, &home, None, &whole, &Job::default()),
            Phase::Done(pack.dir(&home))
        );
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn a_damaged_download_is_thrown_away() {
        let home = temp("damaged");
        let (mut pack, bytes) = pack_zip();
        pack.sha256 = "0".repeat(64);
        let phase = install(&pack, &home, None, &Fake::new(&bytes), &Job::default());
        assert!(
            matches!(phase, Phase::Failed(ref e) if e.contains("damaged")),
            "{phase:?}"
        );
        assert_eq!(partial(&pack, &home), 0);
        assert_eq!(installed(&pack, &home), None);
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn cancelling_keeps_the_download_for_later() {
        let home = temp("cancel");
        let (pack, bytes) = pack_zip();
        let job = Job::default();
        job.cancel();
        assert_eq!(
            install(&pack, &home, None, &Fake::new(&bytes), &job),
            Phase::Cancelled
        );
        assert_eq!(installed(&pack, &home), None);
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn too_little_space_stops_before_downloading() {
        let home = temp("space");
        let (pack, bytes) = pack_zip();
        let fake = Fake::new(&bytes);
        let phase = install(&pack, &home, Some(10), &fake, &Job::default());
        assert!(
            matches!(phase, Phase::Failed(ref e) if e.contains("free space")),
            "{phase:?}"
        );
        assert_eq!(fake.asked_from.get(), None);
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn a_newer_pack_replaces_the_old_one() {
        let home = temp("newer");
        let (pack, bytes) = pack_zip();
        let old = pack.dir(&home).with_file_name("OR2-HD-GUI-vold");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join(MARKER), "x").unwrap();
        let mine = home.join(FOLDER).join("my-own-textures");
        fs::create_dir_all(&mine).unwrap();
        install(&pack, &home, None, &Fake::new(&bytes), &Job::default());
        assert!(!old.exists());
        assert!(
            mine.exists(),
            "folders the launcher didn't install are left alone"
        );
        fs::remove_dir_all(&home).unwrap();
    }
}
