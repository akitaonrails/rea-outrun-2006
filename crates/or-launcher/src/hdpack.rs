//! The community HD texture pack, downloaded when the player asks for it.
//!
//! The download itself is the SDK's (`rea_launcher::download`: free-space
//! check, resumable and verified download, safe unzip, `.complete` marker,
//! older versions removed); this module holds OutRun's pin and where the
//! pack lives: `hd-pack\<release>` under the launcher's home folder, the
//! same folder, file names and marker the launcher used before the SDK.

use std::path::{Path, PathBuf};

use rea_launcher::download::{self, Pin};

const PIN: &str = include_str!("../../../packaging/hd-pack.txt");

/// The packs' folder under the launcher's home folder.
pub const FOLDER: &str = "hd-pack";

/// What the window and `--download-hd-pack` call it.
pub const LABEL: &str = "HD texture pack";

/// The pack `packaging/hd-pack.txt` pins.
pub fn pin() -> Pin {
    Pin::parse(PIN).expect("packaging/hd-pack.txt holds URL SHA-256 size unpacked")
}

/// The download's folder (one per component: older versions there are
/// removed).
pub fn root(home: &Path) -> PathBuf {
    home.join(FOLDER)
}

/// The unpacked pack, when a complete download of the pinned release is
/// there.
pub fn installed(home: &Path) -> Option<PathBuf> {
    download::installed(&pin(), &root(home))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher::identify::hex;

    #[test]
    fn the_pinned_pack_parses() {
        let pin = pin();
        assert_eq!(pin.name(), "OR2-HD-GUI-v0.26.09a");
        assert!(
            pin.url
                .starts_with("https://github.com/envido32/OR2006Sprites/releases/download/")
        );
        assert!(pin.size > 300_000_000 && pin.unpacked_size > 3_000_000_000);
    }

    #[test]
    fn a_pack_the_launcher_downloaded_before_the_sdk_is_found() {
        // The old launcher's layout: hd-pack/<release>/ with `.complete`
        // holding the zip's SHA-256 in lower-case hex, and an interrupted
        // download as hd-pack/<release>.zip.part.
        let home = std::env::temp_dir().join(format!("or-hdpack-old-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        let pin = pin();
        let dir = home.join("hd-pack").join("OR2-HD-GUI-v0.26.09a");
        std::fs::create_dir_all(dir.join("textures/load")).unwrap();
        std::fs::write(home.join("hd-pack/OR2-HD-GUI-v0.26.09a.zip.part"), b"12345").unwrap();
        assert_eq!(installed(&home), None, "no marker: not complete");
        assert_eq!(download::partial(&pin, &root(&home)), 5);
        std::fs::write(dir.join(".complete"), hex(&pin.sha256)).unwrap();
        assert_eq!(installed(&home), Some(dir.clone()));
        download::remove(&pin, &root(&home)).unwrap();
        assert_eq!(installed(&home), None);
        assert_eq!(download::partial(&pin, &root(&home)), 0);
        std::fs::remove_dir_all(&home).unwrap();
    }
}
