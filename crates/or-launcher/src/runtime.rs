//! The runtime folder: a private, writable copy of the user's game folder,
//! and the player's saves kept in step with it.
//!
//! The engine is the SDK's (`rea_launcher::runtime`, `saves`, `fsx`,
//! `paths`); this module holds what OutRun knows. Files are copied (never
//! linked: the game writes into its own folder, `SaveGame\`). Files the
//! launcher owns (the mod's EXE and DLL, the INI files, the SpecialK
//! d3d9.dll shipped by repacks, the uninstaller) are never copied from the
//! source; the launcher places its own. Files under `SaveGame\` are copied
//! once and then belong to the game.

use std::io;
use std::path::{Path, PathBuf};

use rea_launcher::fsx;
use rea_launcher::runtime::{self as engine, Policy, Stamp, SyncReport};
use rea_launcher::saves::{Deleted, FirstRun, Mirror, Moved};

pub use rea_launcher::paths::check_separate;

pub const MANIFEST: &str = ".or-manifest";

/// The game's save folder, by the spelling the game itself uses.
pub const SAVE_DIR: &str = "SaveGame";

/// The runtime's record of the save files as of the last time the saves
/// folder and the runtime's `SaveGame` were in step.
pub const SAVES_STATE: &str = ".or-saves";

/// What OutRun's files are to the runtime copy.
pub struct OutRun;

impl Policy for OutRun {
    /// The mod's EXE and loader DLL (we place the bundled versions), the INI
    /// files (we write them), the SpecialK d3d9.dll repacks ship (it would
    /// load before the real D3D9), the repack's uninstaller, and the crash
    /// dumps the mod writes.
    fn owned(&self, rel: &str, _source: Stamp) -> bool {
        const OWNED: [&str; 13] = [
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
        OWNED.contains(&lower.as_str()) || lower.starts_with("crashdumps/")
    }

    /// Saves: copied once, then the game's.
    fn game_written(&self, rel: &str, _source: Stamp) -> bool {
        rel.split('/')
            .next()
            .is_some_and(|top| top.eq_ignore_ascii_case(SAVE_DIR))
    }
}

/// Brings `runtime` up to date with `game` and makes sure it has a save
/// folder.
pub fn sync(game: &Path, runtime: &Path) -> io::Result<SyncReport> {
    let report = engine::sync(game, runtime, &OutRun, MANIFEST)?;
    save_dir(runtime)?;
    Ok(report)
}

/// Removes the temp files launchers before the SDK left when a write was
/// interrupted (`name.tmp` beside the file); they are only unlinked.
pub fn remove_old_temps(files: &[&Path]) {
    for file in files {
        let mut name = file.file_name().unwrap_or_default().to_os_string();
        name.push(".tmp");
        let _ = fsx::remove(&file.with_file_name(name));
    }
}

/// The runtime folder's save folder, in whatever spelling it has; created
/// as `SaveGame` when missing (Wine on Linux is case-sensitive on disk).
pub fn save_dir(runtime: &Path) -> io::Result<PathBuf> {
    // Listed, not looked up: on Windows a lookup in our spelling succeeds
    // and would hide the folder's own.
    for entry in std::fs::read_dir(runtime)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.file_name().eq_ignore_ascii_case(SAVE_DIR) {
            return Ok(entry.path());
        }
    }
    let dir = runtime.join(SAVE_DIR);
    std::fs::create_dir(&dir)?;
    Ok(dir)
}

/// The mirror between the runtime's `SaveGame` and the player's saves
/// folder. Without a record the saves folder wins; a save deleted on one
/// side while the other stayed unchanged stays deleted (OR's rule).
pub fn saves<'a>(rt_save: &'a Path, saves: &'a Path, state: &'a Path) -> Mirror<'a> {
    Mirror::folder(rt_save, saves, state, FirstRun::SavesWin).deleted(Deleted::Stays)
}

/// At launch: brings both sides in step (see [`saves`]). A saves folder
/// that does not exist yet starts from what the game has: the record (of
/// another folder) is dropped, so the runtime's saves go there.
pub fn saves_in(saves_dir: &Path, rt_save: &Path, state: &Path) -> io::Result<Moved> {
    if !saves_dir.is_dir() {
        fsx::remove(state)?;
    }
    saves(rt_save, saves_dir, state).before()
}

/// After the game: copies what the game wrote to the saves folder. Returns
/// the files copied.
pub fn saves_out(rt_save: &Path, saves_dir: &Path, state: &Path) -> io::Result<Vec<String>> {
    saves(rt_save, saves_dir, state).after()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("or-launcher-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        p
    }

    fn stamp(size: u64, mtime: u64) -> Stamp {
        Stamp { size, mtime }
    }

    #[test]
    fn owned_files_and_saves_follow_outrun() {
        let src: engine::Listing = [
            ("OR2006C2C.EXE", stamp(1, 1)),
            ("dinput8.dll", stamp(2, 2)),
            ("d3d9.dll", stamp(3, 3)),
            ("Sound/a.ogg", stamp(4, 4)),
            ("SaveGame/License.dat", stamp(5, 5)),
            ("CrashDumps/x.dmp", stamp(6, 6)),
        ]
        .into_iter()
        .map(|(r, s)| (r.to_string(), s))
        .collect();
        let p = engine::plan(&src, &|_| false, &engine::Listing::new(), &OutRun);
        assert_eq!(p.copy, ["SaveGame/License.dat", "Sound/a.ogg"]);
        // Owned files stay recorded, so an older launcher's copy is not
        // seen as new forever.
        assert!(p.manifest.contains_key("d3d9.dll"));
        // A changed save in the game folder is not brought again.
        let mut changed = p.manifest.clone();
        changed.insert("SaveGame/License.dat".into(), stamp(9, 9));
        changed.insert("Sound/a.ogg".into(), stamp(9, 9));
        let again = engine::plan(&changed, &|_| true, &p.manifest, &OutRun);
        assert_eq!(again.copy, ["Sound/a.ogg"]);
    }

    #[test]
    fn sync_copies_then_keeps_saves_and_reads_the_old_manifest() {
        let game = temp("game");
        let rt = temp("rt");
        fs::create_dir_all(game.join("SaveGame")).unwrap();
        fs::write(game.join("OR2006C2C.exe"), b"drm exe").unwrap();
        fs::write(game.join("SaveGame/License.dat"), b"orig").unwrap();
        fs::write(game.join("unins000.exe"), b"uninstaller").unwrap();
        fs::write(game.join("Sound.ogg"), b"song").unwrap();

        // A runtime an older launcher made: its manifest is the same format.
        fs::create_dir_all(&rt).unwrap();
        fs::copy(game.join("Sound.ogg"), rt.join("Sound.ogg")).unwrap();
        let listing = engine::list(&game).unwrap();
        let old: engine::Listing = listing
            .iter()
            .filter(|(rel, _)| rel.as_str() == "Sound.ogg")
            .map(|(r, s)| (r.clone(), *s))
            .collect();
        fs::write(rt.join(MANIFEST), engine::format_manifest(&old)).unwrap();
        fs::write(rt.join(".or-manifest.tmp"), b"left by an old launcher").unwrap();
        remove_old_temps(&[&rt.join(MANIFEST)]);
        assert!(!rt.join(".or-manifest.tmp").exists());

        assert_eq!(sync(&game, &rt).unwrap().brought, 1, "only the save file");
        assert!(!rt.join("OR2006C2C.exe").exists());
        assert!(!rt.join("unins000.exe").exists());
        assert_eq!(sync(&game, &rt).unwrap().brought, 0);

        // The game writes its save; the source changes too.
        fs::write(rt.join("SaveGame/License.dat"), b"played").unwrap();
        fs::write(game.join("SaveGame/License.dat"), b"changed").unwrap();
        assert_eq!(sync(&game, &rt).unwrap().brought, 0);
        assert_eq!(
            fs::read(rt.join("SaveGame/License.dat")).unwrap(),
            b"played"
        );
        assert_eq!(
            fs::read(game.join("SaveGame/License.dat")).unwrap(),
            b"changed"
        );
        for d in [game, rt] {
            fs::remove_dir_all(d).unwrap();
        }
    }

    #[test]
    fn a_missing_save_folder_is_created_once_in_any_spelling() {
        let rt = temp("savedir");
        fs::create_dir_all(rt.join("SAVEGAME")).unwrap();
        assert_eq!(save_dir(&rt).unwrap().file_name().unwrap(), "SAVEGAME");
        fs::remove_dir_all(&rt).unwrap();
        fs::create_dir_all(&rt).unwrap();
        assert_eq!(save_dir(&rt).unwrap(), rt.join(SAVE_DIR));
        assert_eq!(save_dir(&rt).unwrap(), rt.join(SAVE_DIR));
        fs::remove_dir_all(&rt).unwrap();
    }

    fn set_mtime(path: &Path, secs: u64) {
        fs::OpenOptions::new()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(UNIX_EPOCH + Duration::from_secs(secs))
            .unwrap();
    }

    #[test]
    fn saves_round_trip_through_the_runtime() {
        let saves = temp("saves-rt-saves");
        let rt = temp("saves-rt-rt");
        let state = temp("saves-rt-state");
        fs::create_dir_all(&saves).unwrap();
        fs::create_dir_all(&rt).unwrap();
        fs::write(saves.join("License.dat"), b"mine").unwrap();
        fs::write(rt.join("License.dat"), b"seeded from the game folder").unwrap();
        set_mtime(&rt.join("License.dat"), 2_000_000_000);
        // No record: the saves folder wins, even over a newer runtime copy.
        let moved = saves_in(&saves, &rt, &state).unwrap();
        assert_eq!((moved.to_runtime.len(), moved.to_saves.len()), (1, 0));
        assert_eq!(fs::read(rt.join("License.dat")).unwrap(), b"mine");

        // The game saves; the session is killed before the write-back.
        fs::write(rt.join("License.dat"), b"progress").unwrap();
        set_mtime(&rt.join("License.dat"), 2_000_000_100);
        let moved = saves_in(&saves, &rt, &state).unwrap();
        assert_eq!(moved.to_saves, ["License.dat"]);
        assert_eq!(fs::read(saves.join("License.dat")).unwrap(), b"progress");

        // The game saves again; the exit write-back copies it.
        fs::write(rt.join("License.dat"), b"more progress").unwrap();
        set_mtime(&rt.join("License.dat"), 2_000_000_200);
        assert_eq!(saves_out(&rt, &saves, &state).unwrap(), ["License.dat"]);
        assert!(saves_in(&saves, &rt, &state).unwrap().is_empty());

        // The player restores an older backup: it reaches the game.
        fs::write(saves.join("License.dat"), b"backup").unwrap();
        set_mtime(&saves.join("License.dat"), 1_000_000_000);
        assert_eq!(
            saves_in(&saves, &rt, &state).unwrap().to_runtime,
            ["License.dat"]
        );
        assert_eq!(fs::read(rt.join("License.dat")).unwrap(), b"backup");

        // A save the game deleted stays deleted (not copied back at launch).
        fs::write(rt.join("NET.dat"), b"net").unwrap();
        saves_out(&rt, &saves, &state).unwrap();
        fs::remove_file(rt.join("NET.dat")).unwrap();
        assert!(saves_in(&saves, &rt, &state).unwrap().is_empty());
        assert!(!rt.join("NET.dat").exists());
        for d in [saves, rt] {
            fs::remove_dir_all(d).unwrap();
        }
        fs::remove_file(state).unwrap();
    }

    #[test]
    fn a_new_saves_folder_starts_from_the_game() {
        let old = temp("new-saves-old");
        let new = temp("new-saves-new");
        let rt = temp("new-saves-rt");
        let state = temp("new-saves-state");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&rt).unwrap();
        fs::write(rt.join("License.dat"), b"progress").unwrap();
        saves_out(&rt, &old, &state).unwrap();
        // The player points the launcher at another, new folder.
        let moved = saves_in(&new, &rt, &state).unwrap();
        assert_eq!(moved.to_saves, ["License.dat"]);
        assert_eq!(fs::read(new.join("License.dat")).unwrap(), b"progress");
        for d in [old, new, rt] {
            fs::remove_dir_all(d).unwrap();
        }
        fs::remove_file(state).unwrap();
    }

    #[test]
    fn the_old_saves_record_is_read() {
        // `.or-saves` from before the SDK: `size\tmtime\trel`, both sides
        // as they were.
        let saves = temp("old-state-saves");
        let rt = temp("old-state-rt");
        let state = temp("old-state");
        fs::create_dir_all(&saves).unwrap();
        fs::create_dir_all(&rt).unwrap();
        for dir in [&saves, &rt] {
            fs::write(dir.join("License.dat"), b"same").unwrap();
            set_mtime(&dir.join("License.dat"), 1_500_000_000);
        }
        fs::write(&state, "4\t1500000000\tLicense.dat\n").unwrap();
        // The game wrote since: the runtime's copy goes out, not the
        // saves folder's in (it would without the record).
        fs::write(rt.join("License.dat"), b"newer").unwrap();
        set_mtime(&rt.join("License.dat"), 1_500_000_100);
        assert_eq!(
            saves_in(&saves, &rt, &state).unwrap().to_saves,
            ["License.dat"]
        );
        assert_eq!(fs::read(saves.join("License.dat")).unwrap(), b"newer");
        for d in [saves, rt] {
            fs::remove_dir_all(d).unwrap();
        }
        fs::remove_file(state).unwrap();
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
