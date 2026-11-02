//! Every option end to end: its flag and its `launcher.cfg` key reach the
//! same setting, and that setting reaches the game file, environment or
//! helper value it changes (through `plan`, what the launcher writes).
//! Plus the command line itself: `--help`, every flag, bad values.

use std::path::Path;

use or_launcher::config::{self, OPTIONS, Settings};
use or_launcher::form;
use or_launcher::plan::{Plan, plan};
use rea_launcher::cfg::{KeyStyle, spec};

fn args(list: &[&str]) -> Result<Settings, String> {
    Settings::parse_args(list.iter().map(|s| s.to_string())).map(|(s, _)| s)
}

fn file(text: &str) -> Settings {
    let (s, warnings) = Settings::parse_file(text);
    assert!(warnings.is_empty(), "{text:?}: {warnings:?}");
    s
}

/// Where an option shows up in what the launcher writes.
enum Lands {
    /// `[section] key = value` in OutRun2006Tweaks.ini.
    Tweak(&'static str, &'static str, &'static str),
    /// `key = value` in outrun2006.ini.
    GameIni(&'static str, &'static str),
    /// A check on the plan or the settings.
    Plan(fn(&Plan) -> bool),
    Settings(fn(&Settings) -> bool),
}

/// key, value (not the default), where it lands.
const CASES: &[(&str, &str, Lands)] = &[
    (
        "game",
        "D:\\Jogos d'Ação\\OutRun",
        Lands::Settings(|s| s.game.as_deref() == Some(Path::new("D:\\Jogos d'Ação\\OutRun"))),
    ),
    (
        "runtime",
        "D:\\José\\rt",
        Lands::Settings(|s| s.runtime.as_deref() == Some(Path::new("D:\\José\\rt"))),
    ),
    (
        "saves",
        "D:\\José\\saves",
        Lands::Settings(|s| s.saves.as_deref() == Some(Path::new("D:\\José\\saves"))),
    ),
    (
        "look",
        "original",
        Lands::Settings(|s| s.look == Some("original")),
    ),
    (
        "resolution",
        "2560x1440",
        Lands::GameIni("DX/RES/WIDTH", "2560"),
    ),
    ("fullscreen", "on", Lands::GameIni("DX/WINDOWED", "0")),
    (
        "fps_limit",
        "144",
        Lands::Tweak("Performance", "FramerateLimit", "144"),
    ),
    ("vsync", "off", Lands::Tweak("Performance", "VSync", "0")),
    (
        "draw_distance",
        "8",
        Lands::Tweak("Graphics", "DrawDistanceIncrease", "8"),
    ),
    (
        "reflections",
        "512",
        Lands::Tweak("Graphics", "ReflectionResolution", "512"),
    ),
    (
        "anisotropy",
        "4",
        Lands::Tweak("Graphics", "AnisotropicFiltering", "4"),
    ),
    ("antialiasing", "2", Lands::GameIni("DX/ANTIALIASING", "2")),
    (
        "sky_glow",
        "off",
        Lands::Tweak("Graphics", "SkyGlowFactor", "0"),
    ),
    (
        "xbox_brightness",
        "on",
        Lands::Tweak("Graphics", "RestoreXboxBrightness", "true"),
    ),
    (
        "hd_characters",
        "off",
        Lands::Tweak("Graphics", "UseHiDefCharacters", "false"),
    ),
    (
        "scaler",
        "fsr",
        Lands::Plan(|p| p.gamescope_filter == "fsr"),
    ),
    (
        "hd_textures",
        "on",
        Lands::Settings(|s| s.hd_textures_source(Path::new("L")).is_some()),
    ),
    (
        "hd_textures_folder",
        "D:\\packs\\OR2-HD",
        Lands::Settings(|s| {
            s.hd_textures_folder.as_deref() == Some(Path::new("D:\\packs\\OR2-HD"))
        }),
    ),
    (
        "deadzone",
        "10",
        Lands::Tweak("Controls", "SteeringDeadZone", "0.10"),
    ),
    (
        "vibration",
        "merged",
        Lands::Tweak("Controls", "VibrationMode", "3"),
    ),
    (
        "vibration_strength",
        "3",
        Lands::Tweak("Controls", "VibrationStrength", "3"),
    ),
    (
        "impulse_triggers",
        "off",
        Lands::Tweak("Controls", "ImpulseVibrationMode", "0"),
    ),
    (
        "input_backend",
        "dinput",
        Lands::Tweak("Controls", "InputBackend", "2"),
    ),
    (
        "manual_gearbox",
        "on",
        Lands::Tweak("Controls", "DefaultManualTransmission", "true"),
    ),
    (
        "cd_switcher",
        "off",
        Lands::Tweak("CDSwitcher", "SwitcherEnable", "false"),
    ),
    (
        "music_shuffle",
        "on",
        Lands::Tweak("CDSwitcher", "SwitcherShuffleTracks", "true"),
    ),
    (
        "skip_logos",
        "off",
        Lands::Tweak("Misc", "SkipIntroLogos", "false"),
    ),
    (
        "free_drive",
        "off",
        Lands::Tweak("Misc", "DisableCountdownTimer", "false"),
    ),
    ("horn", "off", Lands::Tweak("Audio", "AllowHorn", "false")),
    (
        "random_anims",
        "off",
        Lands::Tweak("Misc", "RandomHighwayAnimSets", "false"),
    ),
    (
        "show_miles",
        "off",
        Lands::Tweak("Misc", "ShowOutRunMilesOnMenu", "false"),
    ),
    (
        "soundtrack_fix",
        "off",
        Lands::Plan(|p| p.components.iter().any(|c| c.ends_with("the game's own"))),
    ),
];

fn lands(s: &Settings, key: &str, value: &str, lands: &Lands, how: &str) {
    let p = plan(s, false, None);
    let ok = match lands {
        Lands::Tweak(section, k, want) => p.tweak(section, k).as_deref() == Some(*want),
        Lands::GameIni(k, want) => p.game_ini(k).as_deref() == Some(*want),
        Lands::Plan(check) => check(&p),
        Lands::Settings(check) => check(s),
    };
    assert!(ok, "{how} {key} = {value} did not land:\n{}", p.render());
}

#[test]
fn every_option_reaches_the_game_by_flag_and_by_file() {
    for spec in spec::active(OPTIONS) {
        assert!(
            CASES.iter().any(|(key, ..)| *key == spec.key),
            "{} has no end-to-end case",
            spec.key
        );
    }
    for (key, value, where_) in CASES {
        let flag = config::option(key).flag_name(KeyStyle::Underscore);
        let by_flag = args(&[&flag, value]).unwrap_or_else(|e| panic!("{flag} {value}: {e}"));
        lands(&by_flag, key, value, where_, "flag");
        let by_file = file(&format!("{key} = {value}\n"));
        lands(&by_file, key, value, where_, "file");
        assert_eq!(by_flag, by_file, "{key}: flag and file differ");
    }
}

#[test]
fn the_defaults_are_the_coast_to_coast_set() {
    let p = plan(&Settings::default(), false, None);
    assert_eq!(
        p.tweak("Performance", "FramerateLimit").as_deref(),
        Some("60")
    );
    assert_eq!(
        p.tweak("Graphics", "DrawDistanceIncrease").as_deref(),
        Some("4")
    );
    assert_eq!(
        p.tweak("Graphics", "ReflectionResolution").as_deref(),
        Some("2048")
    );
    assert_eq!(p.tweak("Controls", "InputBackend").as_deref(), Some("0"));
    assert!(p.env.is_empty());
    // Under Wine: XInput, the overrides and the controller values.
    let w = plan(&Settings::default(), true, None);
    assert_eq!(w.tweak("Controls", "InputBackend").as_deref(), Some("3"));
    assert!(
        w.env
            .iter()
            .any(|(k, v)| k == "WINEDLLOVERRIDES" && v.starts_with("dinput8=n,b;d3d9=n,b"))
    );
    assert_eq!(w.winebus.len(), 4);
}

#[test]
fn every_bundled_dll_wine_also_ships_loads_native_under_wine() {
    // Wine ignores WINEDLLOVERRIDES in the game's environment block (SDK
    // v0.3.3): the per-program AppDefaults key is what reaches the loader.
    let w = plan(&Settings::default(), true, None);
    let mut names: Vec<&str> = w.app_dll_overrides.iter().map(|(d, _)| *d).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["d3d9", "dinput8", "msvcp140", "vcomp140", "vcruntime140"]
    );
    assert!(
        w.app_dll_overrides
            .iter()
            .all(|(_, mode)| *mode == "native,builtin")
    );
    // Every VC++ DLL the runtime copy gets is covered.
    for dll in or_launcher::tweaks::VC_DLLS {
        assert!(names.contains(&dll.trim_end_matches(".dll")), "{dll}");
    }
    // Windows loads the program folder's copies anyway: nothing to write.
    assert!(
        plan(&Settings::default(), false, None)
            .app_dll_overrides
            .is_empty()
    );
}

#[test]
fn a_look_flag_sets_every_value_and_single_flags_win() {
    let preset = |look: &str| form::look_settings(look, Some((2560, 1440)), false);
    let saved = file("fps_limit = 120\ndraw_distance = 8\n");
    let s = Settings::combine(
        saved.clone(),
        args(&["--look", "original"]).unwrap(),
        &preset,
    );
    let p = plan(&s, false, None);
    assert_eq!(
        p.tweak("Graphics", "DrawDistanceIncrease").as_deref(),
        Some("0")
    );
    assert_eq!(
        p.tweak("Performance", "FramerateLimit").as_deref(),
        Some("60")
    );
    assert_eq!(p.game_ini("DX/RES/WIDTH").as_deref(), Some("640"));
    let s = Settings::combine(
        saved.clone(),
        args(&["--fps-limit", "90", "--look", "steam_deck"]).unwrap(),
        &preset,
    );
    assert_eq!(s.fps_limit, Some(90), "a single flag wins over --look");
    assert_eq!(s.resolution, Some((1280, 800)));
    // look = in the file is a base for what the file doesn't set.
    let s = Settings::combine(
        file("look = original\nfps_limit = 144\n"),
        Settings::default(),
        &preset,
    );
    assert_eq!((s.fps_limit, s.draw_distance), (Some(144), Some(0)));
    assert_eq!(
        Settings::combine(saved, Settings::default(), &preset).fps_limit,
        Some(120)
    );
    // The look's settings name no folders, pack, scaler or input API.
    for look in ["original", "coast_to_coast", "steam_deck"] {
        let s = preset(look);
        assert_eq!(
            (s.game, s.saves, s.hd_textures, s.scaler, s.input_backend),
            (None, None, None, None, None)
        );
    }
}

#[test]
fn help_lists_every_flag_from_the_table() {
    let help = args(&["--help"]).unwrap_err();
    assert_eq!(help, config::usage());
    for spec in spec::active(OPTIONS) {
        assert!(
            help.contains(&spec.flag_name(KeyStyle::Underscore)),
            "{}",
            spec.key
        );
    }
    for switch in config::SWITCHES {
        assert!(help.contains(switch), "{switch}");
    }
    assert!(help.contains("--scaler nearest|pixel|fsr|linear") && help.contains("(Linux helper)"));
    assert_eq!(args(&["-h"]).unwrap_err(), help);
    assert_eq!(args(&["/?"]).unwrap_err(), help);
}

#[test]
fn bad_values_say_what_they_need() {
    for (argv, message) in [
        (
            &["--fps-limit", "5"][..],
            "--fps-limit needs 10 to 240 or unlimited (0), not 5",
        ),
        (
            &["--fps-limit", "300"],
            "--fps-limit needs 10 to 240 or unlimited (0), not 300",
        ),
        (&["--vsync", "half"], "--vsync needs on or off, not half"),
        (
            &["--reflections", "256"],
            "--reflections needs original, 512, 1024 or 2048, not 256",
        ),
        (
            &["--antialiasing", "4"],
            "--antialiasing needs 0 or 2, not 4",
        ),
        (
            &["--look", "custom"],
            "--look needs original, coast_to_coast or steam_deck, not custom",
        ),
        (
            &["--input-backend", "joy"],
            "--input-backend needs auto, wgi, raw, dinput or xinput, not joy",
        ),
        (&["--deadzone", "60"], "--deadzone needs 0 to 50, not 60"),
        (
            &["--scaler", "bicubic"],
            "--scaler needs nearest, pixel, fsr or linear, not bicubic",
        ),
        (
            &["--resolution", "8000x100"],
            "--resolution needs WIDTHxHEIGHT, original or off, not 8000x100",
        ),
        (&["--game"], "--game needs a value"),
        (
            &["--no-such-option", "on"],
            "unknown argument --no-such-option",
        ),
    ] {
        let error = args(argv).unwrap_err();
        assert!(
            error.starts_with(&format!("{message}\n\n")),
            "{argv:?}: {error}"
        );
        assert!(error.ends_with(&config::usage()));
    }
}

#[test]
fn old_spellings_are_still_read() {
    assert_eq!(
        args(&["--fps-limit", "unlimited"]).unwrap().fps_limit,
        Some(0)
    );
    assert_eq!(args(&["--anisotropy", "off"]).unwrap().anisotropy, Some(0));
    assert_eq!(
        args(&["--reflections", "128"]).unwrap().reflections,
        Some(0)
    );
    assert_eq!(file("fullscreen = true\n").fullscreen, Some(true));
    assert_eq!(file("input_backend = AUTO\n").input_backend, None);
}

#[test]
fn switches_come_back_in_order() {
    let (s, switches) =
        Settings::parse_args(["--no-window", "--game", "--check", "--linux"].map(String::from))
            .unwrap();
    // A switch name in a value position is a value.
    assert_eq!(s.game.as_deref(), Some(Path::new("--check")));
    assert_eq!(switches, ["--no-window", "--linux"]);
}

/// launcher.cfg as v0.1.0 wrote it: read without a warning, and the window
/// saves over it keeping every comment.
#[test]
fn a_launcher_cfg_from_v0_1_0_still_works() {
    let old = include_str!("fixtures/launcher-v0.1.0.cfg");
    let (s, warnings) = Settings::parse_file(old);
    // Only the retired half-rate vsync is mentioned; removed options
    // (character_select, hot_plug) are dropped silently.
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("vsync half is no longer offered"));
    assert_eq!(
        s.game.as_deref(),
        Some(Path::new("D:\\Jogos d'Ação\\OutRun 2006"))
    );
    assert_eq!(s.fps_limit, Some(0));
    assert_eq!(s.reflections, Some(0));
    assert_eq!(s.hd_textures_folder.as_deref(), Some(Path::new("D:\\pack")));
    let f = form::Form::new(&s, Some((1920, 1080)), false);
    let saved = f.save(old);
    assert!(saved.contains("# Free drive: no countdown timer on the stages."));
    assert_eq!(Settings::parse_file(&saved).0.fps_limit, Some(0));
}
