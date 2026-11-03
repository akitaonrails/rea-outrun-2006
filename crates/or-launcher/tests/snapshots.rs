//! Golden snapshots of everything the launcher produces (the SDK's
//! STANDARDS, "Golden snapshots"): `--help`, the first-run
//! `launcher.cfg`, and for each look and a few option combinations the
//! two INI files, the environment, the Wine controller values, the
//! runtime components and the gamescope filter (`plan::plan`). A diff
//! must match an intended change or an SDK migration item; regenerate
//! only then, with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots`.
//! No game bytes: only what the launcher writes.

use std::path::Path;

use or_launcher::config::{self, Settings};
use or_launcher::form;
use or_launcher::plan::plan;

const SCREEN: Option<(u32, u32)> = Some((2560, 1440));

/// The SDK's comparison (`rea_launcher::testing`): the crate version
/// becomes `<version>`, CRLF reads as LF, a diff names its first line.
/// `REA_UPDATE_SNAPSHOTS=1` (or this repository's `UPDATE_SNAPSHOTS=1`)
/// writes them instead.
fn snapshot(name: &str, actual: &str) {
    rea_launcher::snapshots!()
        .update_var("UPDATE_SNAPSHOTS")
        .assert(name, actual);
}

fn args(list: &[&str]) -> Settings {
    Settings::parse_args(list.iter().map(|s| s.to_string()))
        .unwrap()
        .0
}

/// The settings a run uses for `file` and `flags`, on Windows or Wine.
fn run(file: &str, flags: &[&str], wine: bool) -> Settings {
    let (saved, warnings) = Settings::parse_file(file);
    assert!(warnings.is_empty(), "{warnings:?}");
    let preset = |look: &str| form::look_settings(look, SCREEN, wine);
    Settings::combine(saved, args(flags), &preset)
}

#[test]
fn help_and_template() {
    // The version changes every release; the helper keeps a placeholder.
    snapshot("help.txt", &config::usage());
    snapshot("launcher.cfg", &config::template());
}

#[test]
fn each_look() {
    for look in ["original", "coast_to_coast", "steam_deck"] {
        for (platform, wine) in [("windows", false), ("wine", true)] {
            let s = run("", &["--look", look], wine);
            snapshot(
                &format!("look-{look}-{platform}.txt"),
                &plan(&s, wine, None).render(),
            );
        }
    }
}

#[test]
fn option_combinations() {
    // A first run with nothing chosen: the launcher's defaults.
    let s = run("", &[], false);
    snapshot("defaults-windows.txt", &plan(&s, false, None).render());

    // Exclusive fullscreen at 4K, a high cap (held at 60), anti-aliasing,
    // DirectInput, merged rumble, the HD pack where it was unzipped.
    let s = run(
        "resolution = 3840x2160\nfullscreen = on\nfps_limit = 144\nantialiasing = 2\ninput_backend = dinput\nvibration = merged\nhd_textures = on\n",
        &[],
        false,
    );
    let pack = Path::new("D:\\Jogos d'Ação\\OR2-HD\\textures");
    snapshot(
        "fullscreen-hd-pack-windows.txt",
        &plan(&s, false, Some(pack)).render(),
    );

    // Linux: unlimited frames, vsync off, sharp scaling, the extras off,
    // the original song; a one-run flag over the file.
    let s = run(
        "fps_limit = unlimited\nvsync = off\nscaler = nearest\ncd_switcher = off\nskip_logos = off\nfree_drive = off\nhorn = off\nsoundtrack_fix = off\n",
        &["--draw-distance", "8", "--deadzone", "10"],
        true,
    );
    snapshot(
        "unlimited-extras-off-wine.txt",
        &plan(&s, true, None).render(),
    );
}
