//! Which `OR2006C2C.exe` the game folder holds, by SHA-256.
//!
//! The 2006 EXE is wrapped in disc DRM that breaks DLL wrappers, so
//! OutRun2006Tweaks ships its own DRM-free replacement EXE. The launcher
//! recognises both: an original install (it swaps in the replacement) and an
//! install already patched by Tweaks (it refreshes the files it manages).

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

static EDITIONS: [Known<Edition>; 2] = [
    Known::new(
        "bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f",
        ORIGINAL,
    ),
    Known::new(
        "68ceb386829066f8455b9d027320af962584321f3e2e8a79c72841495a6134c3",
        TWEAKS,
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
    fn the_bundled_replacement_exe_is_recognised() {
        assert_eq!(
            identify_with_bundled(b"newer tweaks", Some(b"newer tweaks")),
            Ok(TWEAKS)
        );
        assert!(identify_with_bundled(b"something else", Some(b"newer tweaks")).is_err());
        assert!(identify_with_bundled(b"newer tweaks", None).is_err());
    }

    #[test]
    fn unknown_exes_report_their_sha256() {
        let digest = identify::sha256(b"not the game");
        assert_eq!(identify(b"not the game"), Err(digest));
        assert_eq!(to_hex(&digest).len(), 64);
    }
}
