//! Which game the folder holds: its `OR2006C2C.exe` by SHA-256, else its
//! data.
//!
//! The 2006 EXE is wrapped in disc DRM that breaks DLL wrappers, so
//! OutRun2006Tweaks ships its own DRM-free replacement EXE. The launcher
//! recognises original installs (the 2006 disc and Steam EXEs; it swaps in
//! the replacement) and an install already patched by Tweaks (it refreshes
//! the files it manages). Since it never runs or copies the folder's EXE,
//! an EXE it doesn't know (DVD pressings differ) is accepted when the data
//! files the Tweaks EXE loads are the 2006 release's.

use std::path::Path;

use rea_launcher::identify::{self, Known};

pub const EXE: &str = "OR2006C2C.exe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edition {
    pub name: &'static str,
    /// The replacement EXE is already in place.
    pub patched: bool,
}

/// The original EXE from the 2006 retail/repack install (DRM-wrapped,
/// 14,950,400 bytes, PE timestamp 1148663097 = 2006-05-26).
pub const ORIGINAL: Edition = Edition {
    name: "OutRun 2006 (original EXE; the launcher replaces it in its copy)",
    patched: false,
};

/// The DRM-free replacement EXE from OutRun2006Tweaks (3,674,112 bytes,
/// same PE timestamp; already Large Address Aware).
pub const TWEAKS: Edition = Edition {
    name: "OutRun 2006 + OutRun2006Tweaks EXE",
    patched: true,
};

/// The replacement EXE from OutRun2006Tweaks v0.1 to v0.2.5 (2024): the
/// same 3,674,112 bytes without the Large Address Aware flag. Players who
/// installed an early Tweaks still have it; the launcher swaps in its
/// bundled EXE as for every edition.
pub const TWEAKS_EARLY: Edition = Edition {
    name: "OutRun 2006 + OutRun2006Tweaks 0.1-0.2.5 EXE",
    patched: true,
};

/// The EXE of the 2007-2010 Steam release (962,560 bytes, packed).
/// OutRun2006Tweaks' source describes it as the 2006 EXE before its
/// SecuROM wrapper, repacked for Steam, and its README says the
/// replacement EXE works with the Steam release. We have no Steam copy:
/// the digest is the one two Steam owners reported (issue #1), and the
/// launcher plays its bundled Tweaks EXE instead, as for every edition.
pub const STEAM: Edition = Edition {
    name: "OutRun 2006 Steam EXE (reported by owners; the launcher replaces it)",
    patched: false,
};

static EDITIONS: [Known<Edition>; 4] = [
    Known::new(
        "bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f",
        ORIGINAL,
    ),
    Known::new(
        "68ceb386829066f8455b9d027320af962584321f3e2e8a79c72841495a6134c3",
        TWEAKS,
    ),
    Known::new(
        "c926e54a5620e8e47199caf532235c25ddaba512da198bde15db1ee4375f806f",
        TWEAKS_EARLY,
    ),
    Known::new(
        "4876712f501640951d892069db826e431aede3e4ab87686fc1d556e26df9fec1",
        STEAM,
    ),
];

/// Identifies an EXE image; `Err` carries its SHA-256 for the report we ask
/// the player to send.
pub fn identify(bytes: &[u8]) -> Result<Edition, [u8; 32]> {
    identify::identify(bytes, &EDITIONS).copied()
}

/// Like `identify`, also recognising `bundled`: the replacement EXE that
/// ships with the launcher, which may be a newer Tweaks build than the
/// table knows.
pub fn identify_with_bundled(bytes: &[u8], bundled: Option<&[u8]>) -> Result<Edition, [u8; 32]> {
    identify(bytes).or_else(|digest| match bundled {
        Some(exe) if exe == bytes => Ok(TWEAKS),
        _ => Err(digest),
    })
}

pub use rea_launcher::identify::hex as to_hex;

/// One of the game's data files as the 2006 release ships it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFile {
    /// `/`-separated, matched without regard to case.
    pub path: &'static str,
    pub size: u64,
    pub sha256: [u8; 32],
}

impl DataFile {
    const fn new(path: &'static str, size: u64, sha256_hex: &str) -> DataFile {
        DataFile {
            path,
            size,
            sha256: identify::hex32(sha256_hex),
        }
    }
}

/// Data files that identify the 2006 release (the game the Tweaks EXE
/// runs), from the owner's install and its repack installer, which agree.
/// Language-free files no known mod replaces (mods touch `Sprite/`,
/// `RVT/` and `Sound/`); two are Coast 2 Coast stages a demo would lack.
pub static DATA_2006: [DataFile; 5] = [
    DataFile::new(
        "Common/obj_course_obj_common_pmt.sz",
        844_991,
        "0cfc9a4dc659be92f9da7cab3e659638066ca2013125ba90ecc43057c72a6a99",
    ),
    DataFile::new(
        "Common/def_orhiscores.bin",
        67_680,
        "426ad7d8d5e706f67435bec9590c7a356a2ef8f58bdc1f38d52af7f8d6d1aa51",
    ),
    DataFile::new(
        "Driver/obj_driver_rival_pmt.sz",
        1_138_698,
        "132618f4bf7bc379aea74472d1c74b40a1e62c8f00115c582dce092743a71e6d",
    ),
    DataFile::new(
        "Stage/PALM_BT/CS_CS_PALM_BT_bin.sz",
        56_073,
        "93d84b3f7dd10d07981e16f62a21b2867ef17d4ffca23ad50cb53c5d46df2e38",
    ),
    DataFile::new(
        "Stage/YOSE_R/CS_CS_YOSE_R_bin.sz",
        19_811,
        "e0768983cc61734e3431f0b1d75701aef6b5602e8e0b4a04345432a01f0764bc",
    ),
];

/// An `OR2006C2C.exe` we don't know in a folder whose data is the 2006
/// release. The launcher never runs or copies the game's EXE (it plays its
/// bundled Tweaks EXE in its copy), so only the data has to be right.
/// DVD EXEs are SecuROM-wrapped and differ by region and pressing.
pub const DATA_ONLY: Edition = Edition {
    name: "OutRun 2006 (your OR2006C2C.exe isn't one we know, but it isn't used: the game files match the 2006 release)",
    patched: false,
};

/// A folder the launcher doesn't recognise: its EXE's SHA-256 and the
/// first data file that differs from the 2006 release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unknown {
    pub exe_sha256: [u8; 32],
    pub data: String,
}

/// Identifies a game folder: by its EXE (`exe`, the bytes of its
/// `OR2006C2C.exe`), else by its data.
pub fn identify_folder(
    game: &Path,
    exe: &[u8],
    bundled: Option<&[u8]>,
) -> Result<Edition, Unknown> {
    identify_folder_with(game, exe, bundled, &DATA_2006)
}

fn identify_folder_with(
    game: &Path,
    exe: &[u8],
    bundled: Option<&[u8]>,
    data: &[DataFile],
) -> Result<Edition, Unknown> {
    let exe_sha256 = match identify_with_bundled(exe, bundled) {
        Ok(edition) => return Ok(edition),
        Err(digest) => digest,
    };
    match data_mismatch(game, data) {
        None => Ok(DATA_ONLY),
        Some(data) => Err(Unknown { exe_sha256, data }),
    }
}

/// The first file of `table` that `game` lacks or holds differently,
/// described for the player's report; `None` when all match.
fn data_mismatch(game: &Path, table: &[DataFile]) -> Option<String> {
    table.iter().find_map(|file| {
        let Some(path) = rea_launcher::paths::find_ignoring_case(game, file.path) else {
            return Some(format!("{} is missing", file.path));
        };
        let size = match std::fs::metadata(&path) {
            Ok(meta) => meta.len(),
            Err(e) => return Some(format!("{}: {e}", file.path)),
        };
        if size != file.size {
            return Some(format!("{} has {size} bytes, not {}", file.path, file.size));
        }
        match identify::sha256_file(&path) {
            Ok(digest) if digest == file.sha256 => None,
            Ok(digest) => Some(format!("{} has SHA-256 {}", file.path, to_hex(&digest))),
            Err(e) => Some(format!("{}: {e}", file.path)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_table_is_distinct() {
        assert_eq!(
            to_hex(&EDITIONS[0].sha256),
            "bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f"
        );
        assert_ne!(EDITIONS[0].sha256, EDITIONS[1].sha256);
        assert!(!EDITIONS[0].value.patched && EDITIONS[1].value.patched);
    }

    #[test]
    fn the_early_tweaks_exe_is_known() {
        // OutRun2006Tweaks v0.1 to v0.2.5 shipped this replacement EXE: the
        // later one without the Large Address Aware flag (issue #1 audit).
        let early = EDITIONS
            .iter()
            .find(|known| {
                to_hex(&known.sha256)
                    == "c926e54a5620e8e47199caf532235c25ddaba512da198bde15db1ee4375f806f"
            })
            .expect("the Tweaks 0.1-0.2.5 EXE is in the table");
        assert_eq!(early.value, TWEAKS_EARLY);
        assert!(early.value.patched);
    }

    #[test]
    fn the_steam_exe_is_known_as_reported_not_verified() {
        // The 2007-2010 Steam release (962,560 bytes): two Steam owners
        // reported this digest in issue #1. We have no Steam copy, so the
        // name says it was reported, and the launcher plays its bundled
        // Tweaks EXE instead of it, as for every edition.
        let steam = EDITIONS
            .iter()
            .find(|known| {
                to_hex(&known.sha256)
                    == "4876712f501640951d892069db826e431aede3e4ab87686fc1d556e26df9fec1"
            })
            .expect("the Steam EXE is in the table");
        assert_eq!(steam.value, STEAM);
        assert!(!steam.value.patched);
        assert!(STEAM.name.contains("Steam") && STEAM.name.contains("reported"));
    }

    #[test]
    fn the_bundled_replacement_exe_is_recognised() {
        assert_eq!(
            identify_with_bundled(b"newer tweaks", Some(b"newer tweaks")),
            Ok(TWEAKS)
        );
        assert!(identify_with_bundled(b"something else", Some(b"newer tweaks")).is_err());
        assert!(identify_with_bundled(b"newer tweaks", None).is_err());
    }

    /// A game folder holding `files` (path, bytes).
    fn folder(name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("or-edition-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (path, bytes) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        dir
    }

    fn table() -> [DataFile; 2] {
        [
            DataFile {
                path: "Common/a.bin",
                size: 5,
                sha256: identify::sha256(b"alpha"),
            },
            DataFile {
                path: "Stage/PALM_BT/b.sz",
                size: 4,
                sha256: identify::sha256(b"beta"),
            },
        ]
    }

    #[test]
    fn an_unknown_exe_is_accepted_when_the_data_matches() {
        // A DVD whose SecuROM-wrapped EXE we have never seen: the EXE is
        // never run or copied, so the data the Tweaks EXE loads decides.
        let dir = folder(
            "match",
            &[("common/A.BIN", b"alpha"), ("Stage/palm_bt/B.sz", b"beta")],
        );
        assert_eq!(
            identify_folder_with(&dir, b"another dvd exe", None, &table()),
            Ok(DATA_ONLY)
        );
        assert!(DATA_ONLY.name.contains("isn't used") && DATA_ONLY.name.contains("2006"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_known_exe_wins_without_reading_the_data() {
        let dir = folder("known", &[]);
        // No data at all: a recognised EXE never needs it.
        let bundled = b"the bundled tweaks exe";
        assert_eq!(
            identify_folder_with(&dir, bundled, Some(bundled), &table()),
            Ok(TWEAKS)
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_unknown_exe_with_other_data_is_refused_with_both_reasons() {
        let digest = identify::sha256(b"another exe");
        let missing = folder("missing", &[("Common/a.bin", b"alpha")]);
        let unknown = identify_folder_with(&missing, b"another exe", None, &table()).unwrap_err();
        assert_eq!(unknown.exe_sha256, digest);
        assert!(unknown.data.contains("Stage/PALM_BT/b.sz") && unknown.data.contains("missing"));

        let resized = folder(
            "resized",
            &[("Common/a.bin", b"alpha!"), ("Stage/PALM_BT/b.sz", b"beta")],
        );
        let unknown = identify_folder_with(&resized, b"another exe", None, &table()).unwrap_err();
        assert!(unknown.data.contains("Common/a.bin") && unknown.data.contains("6 bytes"));

        // Same size, other content: a modded or different release.
        let modded = folder(
            "modded",
            &[("Common/a.bin", b"alpha"), ("Stage/PALM_BT/b.sz", b"BETA")],
        );
        let unknown = identify_folder_with(&modded, b"another exe", None, &table()).unwrap_err();
        assert!(
            unknown.data.contains("Stage/PALM_BT/b.sz"),
            "{}",
            unknown.data
        );
        assert!(unknown.data.contains(&to_hex(&identify::sha256(b"BETA"))));
        for dir in [missing, resized, modded] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn the_data_table_is_the_2006_release() {
        // From the owner's install and, independently extracted, its
        // repack installer (the one holding the bdafa88a EXE): language-
        // free files no known mod replaces, two of them Coast 2 Coast
        // stages a demo lacks.
        assert_eq!(DATA_2006.len(), 5);
        for file in &DATA_2006 {
            assert!(file.size > 0 && !file.path.contains('\\'), "{}", file.path);
            assert!(
                !["Sound/", "Sprite/", "RVT/", "Text/", "SaveGame/"]
                    .iter()
                    .any(|skip| file.path.starts_with(skip)),
                "{} is a folder mods or languages change",
                file.path
            );
        }
        assert!(
            DATA_2006
                .iter()
                .any(|f| f.path.starts_with("Stage/PALM_BT/"))
        );
    }

    #[test]
    #[ignore = "needs an install: OR2006_GAME_DIR=<game folder> cargo test -- --ignored"]
    fn a_real_install_is_recognised_by_its_data() {
        let dir = std::env::var_os("OR2006_GAME_DIR").expect("OR2006_GAME_DIR");
        assert_eq!(
            identify_folder(Path::new(&dir), b"an EXE nobody knows", None),
            Ok(DATA_ONLY)
        );
    }

    #[test]
    fn unknown_exes_report_their_sha256() {
        let digest = identify::sha256(b"not the game");
        assert_eq!(identify(b"not the game"), Err(digest));
        assert_eq!(to_hex(&digest).len(), 64);
    }
}
