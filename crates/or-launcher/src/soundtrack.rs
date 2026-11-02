//! The soundtrack fix: the PC release of "Rush a Difficulty" is missing its
//! first two seconds. OutRun2006Tweaks ships an xdelta3 patch
//! (`14_Rush_a_Difficulty_1989.ogg.vcdiff`) that restores them from the
//! player's own file; we apply it to the runtime copy so no song data is
//! ever distributed by us.
//!
//! Decoding needs xdelta3 (the delta uses DJW secondary compression, so a
//! plain VCDIFF reader is not enough): the bundled `xdelta3.exe` (the one
//! the upstream fix ships, GPL) runs on Windows and under Wine; a system
//! `xdelta3` on PATH is used when the bundled one is missing.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const TRACK: &str = "Sound/14_Rush_a_Difficulty_1989.ogg";
const PATCH: &str = "14_Rush_a_Difficulty_1989.ogg.vcdiff";
/// The truncated file every known release ships.
const BROKEN_SIZE: u64 = 3_055_933;
/// The restored file, as the upstream fix's script documents.
const FIXED_SIZE: u64 = 3_447_457;

/// What applying (or deciding not to) did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// The song is restored.
    Fixed,
    /// Already restored on an earlier run (or a different, unknown file:
    /// left alone).
    AlreadyFixed,
    /// `xdelta3` was nowhere to be found.
    NoTool(String),
    Failed(String),
}

/// Applies the fix in `runtime` (the private copy), using `launcher_dir`'s
/// bundled patch and tool. The original is kept in `Sound/Backup/`.
pub fn apply(runtime: &Path, launcher_dir: &Path) -> Applied {
    let song = runtime.join(TRACK);
    let Ok(meta) = fs::metadata(&song) else {
        return Applied::AlreadyFixed;
    };
    match meta.len() {
        BROKEN_SIZE => {}
        FIXED_SIZE => return Applied::AlreadyFixed,
        other => {
            let _ = other;
            return Applied::AlreadyFixed;
        }
    }
    let patch = launcher_dir.join("soundtrack").join(PATCH);
    if !patch.is_file() {
        return Applied::NoTool(patch.display().to_string());
    }
    let backup_dir = runtime.join("Sound/Backup");
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        return Applied::Failed(format!("cannot create {}: {e}", backup_dir.display()));
    }
    let backup = backup_dir.join("14_Rush_a_Difficulty_1989.ogg");
    if !backup.is_file()
        && let Err(e) = fs::copy(&song, &backup)
    {
        return Applied::Failed(format!("cannot back up the song: {e}"));
    }
    let output = runtime.join(format!("{TRACK}.fixed"));
    let Some((program, args)) = tool(launcher_dir, &song, &patch, &output) else {
        return Applied::NoTool("xdelta3 (bundled or on PATH)".into());
    };
    let run = Command::new(&program)
        .args(&args)
        .output()
        .map_err(|e| e.to_string())
        .and_then(|out| {
            if out.status.success() {
                Ok(())
            } else {
                Err(format!(
                    "xdelta3 failed: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                ))
            }
        })
        .and_then(|()| {
            fs::metadata(&output)
                .map(|m| m.len())
                .map_err(|e| e.to_string())
        });
    match run {
        Ok(FIXED_SIZE) => match fs::rename(&output, &song) {
            Ok(()) => Applied::Fixed,
            Err(e) => Applied::Failed(format!("cannot put the song back: {e}")),
        },
        Ok(other) => {
            let _ = fs::remove_file(&output);
            Applied::Failed(format!(
                "the patch produced {other} bytes, expected {FIXED_SIZE}"
            ))
        }
        Err(e) => {
            let _ = fs::remove_file(&output);
            Applied::Failed(e)
        }
    }
}

/// The xdelta3 command: the bundled Windows build (works under Wine too),
/// else a system `xdelta3`.
fn tool(
    launcher_dir: &Path,
    song: &Path,
    patch: &Path,
    output: &Path,
) -> Option<(PathBuf, Vec<String>)> {
    let bundled = launcher_dir.join("soundtrack").join("xdelta3.exe");
    if bundled.is_file() {
        return Some((
            bundled,
            vec![
                "-d".into(),
                "-s".into(),
                song.display().to_string(),
                patch.display().to_string(),
                output.display().to_string(),
            ],
        ));
    }
    Some((
        PathBuf::from("xdelta3"),
        vec![
            "-d".into(),
            "-s".into(),
            song.display().to_string(),
            patch.display().to_string(),
            output.display().to_string(),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_song_leaves_everything_alone() {
        let dir = std::env::temp_dir().join(format!("or-sound-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(apply(&dir, &dir), Applied::AlreadyFixed);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_unrecognized_song_is_left_alone() {
        let dir = std::env::temp_dir().join(format!("or-sound-2-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Sound")).unwrap();
        fs::write(dir.join(TRACK), vec![0u8; 1024]).unwrap();
        assert_eq!(apply(&dir, &dir), Applied::AlreadyFixed);
        fs::remove_dir_all(&dir).unwrap();
    }
}
