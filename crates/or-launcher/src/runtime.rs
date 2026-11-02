//! The runtime folder: a private, writable copy of the user's game folder.
//!
//! The user's folder is never written. Files are copied (not linked) because
//! the game writes into its own folder (`SaveGame\`), and a link would write
//! straight through to the original. A manifest records what each copied
//! file looked like in the source, so later launches copy only what changed
//! there. Files the launcher owns (the mod's EXE and DLL, the SpecialK
//! d3d9.dll shipped by repacks, the uninstaller) are never copied from the
//! source; the launcher places its own. Files under `SaveGame\` are copied
//! once and then belong to the game.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const MANIFEST: &str = ".or-manifest";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub size: u64,
    pub mtime: u64,
}

/// Relative path (with `/` separators) to its stamp.
pub type Listing = BTreeMap<String, Stamp>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub copy: Vec<String>,
    /// The new manifest, written after the copies succeed.
    pub manifest: Listing,
}

/// The game's save folder, by the spelling the game itself uses.
pub const SAVE_DIR: &str = "SaveGame";

/// Files and folders the launcher owns in the runtime copy: the mod's EXE
/// and loader DLL (we place the bundled versions), the INI files (we write
/// them), the SpecialK d3d9.dll repacks ship (it would load before the
/// real D3D9), and the repack's uninstaller.
fn is_owned(rel: &str) -> bool {
    let owned = [
        "d3d9.dll",
        "d3d9.ini",
        "dinput8.dll",
        "or2006c2c.exe",
        "or2006c2c.exe.original",
        "outrun2006.ini",
        "outrun2006tweaks.ini",
        "outrun2006tweaks.lods.ini",
        "outrun2006tweaks.user.ini",
        "outrun2006tweaks.imgui.ini",
        "outrun2006tweaks.log",
        "unins000.exe",
        "unins000.dat",
    ];
    let lower = rel.to_ascii_lowercase();
    if owned.contains(&lower.as_str()) {
        return true;
    }
    // CrashDumps the mod writes; nothing there is worth copying.
    lower.starts_with("crashdumps/")
}

fn is_save(rel: &str) -> bool {
    rel.split('/')
        .next()
        .is_some_and(|top| top.eq_ignore_ascii_case(SAVE_DIR))
}

/// Decides which files to copy.
///
/// - missing in the runtime folder: copy, unless the launcher owns it;
/// - changed in the source since the last copy (per the manifest): copy,
///   except save files, which the game owns once they exist.
pub fn plan(source: &Listing, runtime_has: &dyn Fn(&str) -> bool, manifest: &Listing) -> Plan {
    let mut copy = Vec::new();
    let mut next = Listing::new();
    for (rel, stamp) in source {
        let present = runtime_has(rel);
        let changed = manifest.get(rel) != Some(stamp);
        if is_owned(rel) {
            // Recorded so a file the launcher once copied (older launcher)
            // is not seen as missing forever; never copied again.
            next.insert(rel.clone(), *manifest.get(rel).unwrap_or(stamp));
        } else if !present || (changed && !is_save(rel)) {
            copy.push(rel.clone());
            next.insert(rel.clone(), *stamp);
        } else {
            next.insert(rel.clone(), *manifest.get(rel).unwrap_or(stamp));
        }
    }
    Plan {
        copy,
        manifest: next,
    }
}

pub fn parse_manifest(text: &str) -> Listing {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let size = parts.next()?.parse().ok()?;
            let mtime = parts.next()?.parse().ok()?;
            let rel = parts.next()?.to_owned();
            Some((rel, Stamp { size, mtime }))
        })
        .collect()
}

pub fn format_manifest(listing: &Listing) -> String {
    listing
        .iter()
        .map(|(rel, s)| format!("{}\t{}\t{}\n", s.size, s.mtime, rel))
        .collect()
}

fn mtime_secs(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

/// Lists every file below `root`.
pub fn list(root: &Path) -> io::Result<Listing> {
    fn walk(root: &Path, dir: &Path, out: &mut Listing) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;
            if meta.is_dir() {
                walk(root, &path, out)?;
            } else if meta.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .expect("walk stays below root")
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.insert(
                    rel,
                    Stamp {
                        size: meta.len(),
                        mtime: mtime_secs(&meta),
                    },
                );
            }
        }
        Ok(())
    }
    let mut out = Listing::new();
    walk(root, root, &mut out)?;
    Ok(out)
}

pub fn native(root: &Path, rel: &str) -> PathBuf {
    rel.split('/')
        .fold(root.to_path_buf(), |p, part| p.join(part))
}

/// Refuses runtime folders that overlap the game folder.
pub fn check_separate(game: &Path, runtime: &Path) -> Result<(), String> {
    let g = fs::canonicalize(game).map_err(|e| format!("game folder {}: {e}", game.display()))?;
    let r = fs::canonicalize(runtime).unwrap_or_else(|_| runtime.to_path_buf());
    if r.starts_with(&g) || g.starts_with(&r) {
        return Err(format!(
            "the runtime folder {} must be outside the game folder {}",
            runtime.display(),
            game.display()
        ));
    }
    Ok(())
}

fn make_writable(path: &Path) -> io::Result<()> {
    let mut perm = fs::metadata(path)?.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perm.set_readonly(false);
    fs::set_permissions(path, perm)
}

/// Brings `runtime` up to date with `game`. Returns how many files were
/// copied.
pub fn sync(game: &Path, runtime: &Path) -> io::Result<usize> {
    let source = list(game)?;
    fs::create_dir_all(runtime)?;
    let manifest_path = runtime.join(MANIFEST);
    let manifest = fs::read_to_string(&manifest_path)
        .map(|t| parse_manifest(&t))
        .unwrap_or_default();
    let plan = plan(&source, &|rel| native(runtime, rel).is_file(), &manifest);
    for rel in &plan.copy {
        let from = native(game, rel);
        let to = native(runtime, rel);
        if let Some(dir) = to.parent() {
            fs::create_dir_all(dir)?;
        }
        if to.exists() {
            make_writable(&to)?;
        }
        fs::copy(&from, &to)?;
        // Copies of read-only source files (e.g. from a DVD) are read-only
        // too; the game must be able to write.
        make_writable(&to)?;
        let stamp = source[rel];
        let file = fs::OpenOptions::new().write(true).open(&to)?;
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(stamp.mtime))?;
    }
    fs::write(&manifest_path, format_manifest(&plan.manifest))?;
    ensure_save_dir(runtime)?;
    Ok(plan.copy.len())
}

/// The runtime folder's save folder, in whatever spelling it has; created
/// as `SaveGame` when missing (Wine on Linux is case-sensitive on disk).
pub fn save_dir(runtime: &Path) -> io::Result<PathBuf> {
    for entry in fs::read_dir(runtime)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.file_name().eq_ignore_ascii_case(SAVE_DIR) {
            return Ok(entry.path());
        }
    }
    let dir = runtime.join(SAVE_DIR);
    fs::create_dir(&dir)?;
    Ok(dir)
}

fn ensure_save_dir(runtime: &Path) -> io::Result<()> {
    save_dir(runtime).map(|_| ())
}

/// Copies every file below `from` into `to` (creating folders), skipping
/// files whose copy already has the same size and modification time.
/// Returns how many files were copied. Used to keep the player's save
/// folder (`saves` setting) and the runtime's `SaveGame` folder in step.
pub fn mirror(from: &Path, to: &Path) -> io::Result<usize> {
    if !from.is_dir() {
        return Ok(0);
    }
    let mut copied = 0;
    for (rel, stamp) in list(from)? {
        let src = native(from, &rel);
        let dst = native(to, &rel);
        let same = fs::metadata(&dst)
            .map(|m| m.len() == stamp.size && mtime_secs(&m) == stamp.mtime)
            .unwrap_or(false);
        if same {
            continue;
        }
        if let Some(dir) = dst.parent() {
            fs::create_dir_all(dir)?;
        }
        if dst.exists() {
            make_writable(&dst)?;
        }
        fs::copy(&src, &dst)?;
        make_writable(&dst)?;
        let file = fs::OpenOptions::new().write(true).open(&dst)?;
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(stamp.mtime))?;
        copied += 1;
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(size: u64, mtime: u64) -> Stamp {
        Stamp { size, mtime }
    }

    fn listing(items: &[(&str, Stamp)]) -> Listing {
        items.iter().map(|(r, s)| (r.to_string(), *s)).collect()
    }

    #[test]
    fn first_sync_copies_everything_it_does_not_own() {
        let src = listing(&[
            ("OR2006C2C.EXE", stamp(1, 1)),
            ("dinput8.dll", stamp(2, 2)),
            ("d3d9.dll", stamp(3, 3)),
            ("Sound/a.ogg", stamp(4, 4)),
            ("SaveGame/License.dat", stamp(5, 5)),
            ("CrashDumps/x.dmp", stamp(6, 6)),
        ]);
        let p = plan(&src, &|_| false, &Listing::new());
        assert_eq!(p.copy, ["SaveGame/License.dat", "Sound/a.ogg"]);
    }

    #[test]
    fn unchanged_source_copies_nothing() {
        let src = listing(&[("Sound/a.ogg", stamp(1, 1))]);
        let p = plan(&src, &|_| true, &src);
        assert!(p.copy.is_empty());
    }

    #[test]
    fn changed_game_file_is_recopied_but_saves_are_kept() {
        let old = listing(&[
            ("Sound/a.ogg", stamp(1, 1)),
            ("SaveGame/License.dat", stamp(1, 1)),
        ]);
        let new = listing(&[
            ("Sound/a.ogg", stamp(1, 9)),
            ("SaveGame/License.dat", stamp(5, 9)),
        ]);
        let p = plan(&new, &|_| true, &old);
        assert_eq!(p.copy, ["Sound/a.ogg"]);
        assert_eq!(p.manifest["SaveGame/License.dat"], stamp(1, 1));
    }

    #[test]
    fn owned_files_stay_owned_even_after_an_older_launcher_copied_them() {
        // A runtime made by an older launcher contains d3d9.dll from the
        // source; we never copy it again, present or not.
        let src = listing(&[("d3d9.dll", stamp(7, 7)), ("Sound/a.ogg", stamp(1, 1))]);
        let p = plan(&src, &|_| false, &Listing::new());
        assert_eq!(p.copy, ["Sound/a.ogg"]);
        assert!(p.manifest.contains_key("d3d9.dll"));
    }

    #[test]
    fn manifest_round_trips_with_spaces() {
        let l = listing(&[("Sound/a b.ogg", stamp(10, 20))]);
        assert_eq!(parse_manifest(&format_manifest(&l)), l);
    }

    fn temp(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("or-launcher-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn sync_copies_then_keeps_saves() {
        let game = temp("game");
        let rt = temp("rt");
        fs::create_dir_all(game.join("SaveGame")).unwrap();
        fs::write(game.join("OR2006C2C.EXE"), b"drm exe").unwrap();
        fs::write(game.join("SaveGame/License.dat"), b"orig").unwrap();
        fs::write(game.join("unins000.exe"), b"uninstaller").unwrap();

        assert_eq!(sync(&game, &rt).unwrap(), 1, "only the save file");
        assert!(!rt.join("OR2006C2C.EXE").exists());
        assert!(!rt.join("unins000.exe").exists());
        assert_eq!(sync(&game, &rt).unwrap(), 0);

        // The game writes its save; the source changes too.
        fs::write(rt.join("SaveGame/License.dat"), b"played").unwrap();
        fs::write(game.join("SaveGame/License.dat"), b"changed").unwrap();

        assert_eq!(sync(&game, &rt).unwrap(), 0);
        assert_eq!(
            fs::read(rt.join("SaveGame/License.dat")).unwrap(),
            b"played"
        );
        assert_eq!(
            fs::read(game.join("SaveGame/License.dat")).unwrap(),
            b"changed"
        );

        fs::remove_dir_all(&game).unwrap();
        fs::remove_dir_all(&rt).unwrap();
    }

    #[test]
    fn sync_creates_a_missing_save_folder_once() {
        let game = temp("nosave");
        let rt = temp("nosave-rt");
        fs::create_dir_all(game.join("Sound")).unwrap();
        fs::write(game.join("Sound/a.ogg"), b"x").unwrap();
        sync(&game, &rt).unwrap();
        assert!(rt.join(SAVE_DIR).is_dir());
        sync(&game, &rt).unwrap();

        // An existing folder in another spelling is reused.
        let rt2 = temp("nosave-rt2");
        fs::create_dir_all(rt2.join("savegame")).unwrap();
        sync(&game, &rt2).unwrap();
        let dirs: Vec<_> = fs::read_dir(&rt2)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().unwrap().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|n| n != "Sound")
            .collect::<Vec<_>>();
        assert_eq!(dirs, ["savegame"]);

        for d in [game, rt, rt2] {
            fs::remove_dir_all(d).unwrap();
        }
    }

    #[test]
    fn mirror_copies_new_and_changed_files_only() {
        let a = temp("mirror-a");
        let b = temp("mirror-b");
        fs::create_dir_all(&a).unwrap();
        fs::write(a.join("License.dat"), b"one").unwrap();
        fs::write(a.join("NET.dat"), b"net").unwrap();
        assert_eq!(mirror(&a, &b).unwrap(), 2);
        assert_eq!(mirror(&a, &b).unwrap(), 0, "unchanged files are skipped");
        fs::write(a.join("License.dat"), b"two!").unwrap();
        assert_eq!(mirror(&a, &b).unwrap(), 1);
        assert_eq!(fs::read(b.join("License.dat")).unwrap(), b"two!");
        assert_eq!(
            mirror(&temp("mirror-none"), &b).unwrap(),
            0,
            "no source folder"
        );
        for d in [a, b] {
            fs::remove_dir_all(d).unwrap();
        }
    }

    #[test]
    fn save_dir_finds_any_spelling() {
        let rt = temp("savedir");
        fs::create_dir_all(rt.join("SAVEGAME")).unwrap();
        assert_eq!(save_dir(&rt).unwrap().file_name().unwrap(), "SAVEGAME");
        fs::remove_dir_all(&rt).unwrap();
    }

    #[test]
    fn overlapping_folders_are_refused() {
        let game = temp("overlap");
        fs::create_dir_all(&game).unwrap();
        assert!(check_separate(&game, &game.join("rt")).is_err());
        assert!(check_separate(&game, &temp("elsewhere")).is_ok());
        fs::remove_dir_all(&game).unwrap();
    }
}
