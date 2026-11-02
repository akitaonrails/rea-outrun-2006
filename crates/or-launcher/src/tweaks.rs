//! Wiring the private runtime copy: the bundled OutRun2006Tweaks files,
//! the VC++ runtime the mod needs, DXVK on Linux, the two INI files, the
//! HD texture pack and the soundtrack fix.
//!
//! Every step is idempotent: files are replaced only when they differ, and
//! the INI files keep the user's edits to keys this launcher does not own.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Settings;
use crate::ini;
use crate::soundtrack;

/// Files of the OutRun2006Tweaks build this launcher is wired for, next to
/// the launcher (`tweaks/`). The package pins their source and digest
/// (`packaging/tweaks.txt`).
pub const TWEAKS_DIR: &str = "tweaks";
const VC_DIR: &str = "vc";
const USER_INI: &str = "OutRun2006Tweaks.user.ini";
const DXVK_MARKER: &str = ".or-dxvk";

/// The x86 VC++ 2022 runtime DLLs the mod needs, deployed app-local (the
/// officially supported way) so no installer is ever run.
pub const VC_DLLS: [&str; 3] = ["msvcp140.dll", "vcruntime140.dll", "vcomp140.dll"];

/// A minimal base INI of our own writing, used only when the packaged
/// upstream INI is missing. Sections and defaults follow the upstream
/// `OutRun2006Tweaks.ini`; comments are ours.
pub const BASE_INI: &str = concat!(
    "# OutRun2006Tweaks - https://github.com/emoose/OutRun2006Tweaks\n",
    "# Managed by OutRunLauncher; keys it owns are rewritten each run.\n\n",
    "[Performance]\nFramerateLimit = 0\nFramerateLimitMode = 0\nFramerateFastLoad = 3\nFramerateInterpolation = true\nVSync = 1\n\n",
    "[Controls]\nUseNewInput = true\nInputBackend = 0\nSteeringDeadZone = 0.2\nBypassGameSensitivity = false\nControllerHotPlug = true\nDefaultManualTransmission = false\nVibrationMode = 0\nVibrationControllerId = 0\nVibrationStrength = 7\nImpulseVibrationMode = 2\nImpulseVibrationLeftMultiplier = 0.20\nImpulseVibrationRightMultiplier = 0.20\n\n",
    "[Graphics]\nUIScalingMode = 1\nUILetterboxing = 1\nRestoreXboxBrightness = false\nSkyGlowFactor = 4\nSkyGlowTwoStep = true\nAnisotropicFiltering = 16\nReflectionResolution = 1024\nReflectionUpdateRate = 0.5\nUseHiDefCharacters = true\nTransparencySupersampling = true\nScreenEdgeCullFix = true\nDisableVehicleLODs = true\nDisableStageCulling = true\nFixZBufferPrecision = true\nCarBaseShadowOpacity = 1.0\nDrawDistanceIncrease = 0\nDrawDistanceBehind = 0\nTextureBaseFolder = textures\nSceneTextureReplacement = true\nUITextureReplacement = true\nSceneTextureExtract = false\nUITextureExtract = false\nEnableTextureCache = true\nUseNewTextureAllocator = true\n\n",
    "[Audio]\nAllowHorn = true\nAllowWAV = true\nAllowFLAC = true\n\n",
    "[CDSwitcher]\nSwitcherEnable = false\nSwitcherDisplayTitle = true\nSwitcherTitleFont = 2\nSwitcherTitleFontSizeX = 0.3\nSwitcherTitleFontSizeY = 0.5\nSwitcherTitlePositionX = 375\nSwitcherTitlePositionY = 450\nSwitcherShuffleTracks = false\n\n",
    "[Window]\nWindowedBorderless = true\nWindowPositionX = 0\nWindowPositionY = 0\nWindowedHideMouseCursor = true\nDisableDPIScaling = true\nAutoDetectResolution = true\n\n",
    "[Misc]\nEnableHollyCourse2 = 3\nSkipIntroLogos = false\nDisableCountdownTimer = false\nEnableLevelSelect = false\nRestoreJPClarissa = false\nShowOutRunMilesOnMenu = true\nAllowCharacterSelection = false\nRandomHighwayAnimSets = false\nDemonwareServerOverride = clarissa.port0.org\nProtectLoginData = true\n\n",
    "[Overlay]\nEnabled = true\n\n",
    "[Bugfixes]\nFixPegasusClopping = true\nFixRightSideBunkiAnimations = true\nFixC2CRankings = true\nPreventDESTSaveCorruption = true\nFixLensFlarePath = true\nFixIncorrectShading = true\nFixParticleRendering = true\nFixFullPedalChecks = true\nHideOnlineSigninText = false\n\n",
    "[CDTracks]\n",
    include_str!("tracks.ini"),
);

/// Copies `from` to `to` when their bytes differ. Returns true when a copy
/// happened.
fn freshen(from: &Path, to: &Path) -> Result<bool, String> {
    let same = fs::read(from)
        .ok()
        .zip(fs::read(to).ok())
        .is_some_and(|(a, b)| a == b);
    if same {
        return Ok(false);
    }
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    fs::copy(from, to).map_err(|e| format!("cannot copy to {}: {e}", to.display()))?;
    Ok(true)
}

/// A base INI we can build on: the sections our keys live in.
fn usable_base(text: &str) -> bool {
    ["[Performance]", "[Controls]", "[Graphics]", "[CDTracks]"]
        .iter()
        .all(|section| text.contains(section))
}

/// Places everything the game needs into `runtime`. `launcher_dir` is the
/// folder holding OutRunLauncher.exe and its `tweaks/`, `vc/`, `linux/` and
/// `textures-pack/` folders. `linux` selects the DXVK install (the caller
/// decides from Wine detection).
pub fn install(
    runtime: &Path,
    launcher_dir: &Path,
    linux: bool,
    settings: &Settings,
    log: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let tweaks = launcher_dir.join(TWEAKS_DIR);
    for name in ["dinput8.dll", "OR2006C2C.exe"] {
        let source = tweaks.join(name);
        if !source.is_file() {
            return Err(format!(
                "OutRun2006Tweaks is missing: {}. Keep the tweaks folder next to OutRunLauncher.exe.",
                source.display()
            ));
        }
        if freshen(&source, &runtime.join(name))? {
            log(&format!("tweaks: {name} installed"));
        }
    }
    let lods = tweaks.join("OutRun2006Tweaks.lods.ini");
    if lods.is_file() {
        freshen(&lods, &runtime.join("OutRun2006Tweaks.lods.ini"))?;
    } else {
        // The known-bad object list is part of the mod build; running
        // without it only risks some distant geometry glitches.
        log("tweaks: OutRun2006Tweaks.lods.ini not found; draw-distance exclusions off");
    }

    // The VC++ runtime, app-local: the mod needs a 2024+ x86 redist, which
    // few machines have (most have only the x64 one).
    for name in VC_DLLS {
        let source = launcher_dir.join(VC_DIR).join(name);
        if source.is_file() && freshen(&source, &runtime.join(name))? {
            log(&format!("vc++ runtime: {name} installed"));
        }
    }

    // The mod's INI: the packaged upstream file as the base (all its
    // comments and defaults), ours only as the fallback; then our keys.
    // A base that is truncated or from another tool is replaced by ours.
    let ini_path = runtime.join("OutRun2006Tweaks.ini");
    let mut base = fs::read_to_string(&ini_path).unwrap_or_default();
    let usable = ["[Performance]", "[Controls]", "[Graphics]", "[CDTracks]"]
        .iter()
        .all(|section| base.contains(section));
    if !usable {
        base = match fs::read_to_string(tweaks.join("OutRun2006Tweaks.ini")) {
            Ok(packaged) if usable_base(&packaged) => packaged,
            _ => BASE_INI.into(),
        };
    }
    let pack = hd_texture_base(settings, launcher_dir, log);
    let tuned = texture_keys(&settings.apply_tweaks(&base), pack.as_deref());
    // Without a choice in launcher.cfg, the input API follows the platform.
    let backend = settings.backend_on(linux).ini().to_string();
    let tuned = ini::set_section_key(&tuned, "Controls", "InputBackend", &backend);
    fs::write(&ini_path, &tuned)
        .map_err(|e| format!("cannot write {}: {e}", ini_path.display()))?;
    reset_overlay_keys(&runtime.join(USER_INI), log)?;

    // The game's own settings, written whole (the mod deletes stale copies
    // older than three months; ours is fresh every run).
    let game_ini = runtime.join("outrun2006.ini");
    fs::write(&game_ini, settings.outrun2006_ini())
        .map_err(|e| format!("cannot write {}: {e}", game_ini.display()))?;

    install_dxvk(runtime, launcher_dir, linux, log)?;

    if settings.soundtrack_fix.unwrap_or(true) {
        match soundtrack::apply(runtime, launcher_dir) {
            soundtrack::Applied::Fixed => log("soundtrack: Rush a Difficulty restored"),
            soundtrack::Applied::AlreadyFixed => {}
            soundtrack::Applied::NoTool(what) => log(&format!(
                "warning: soundtrack fix skipped, {what} not found"
            )),
            soundtrack::Applied::Failed(e) => log(&format!("warning: soundtrack fix failed: {e}")),
        }
    } else {
        match soundtrack::restore(runtime) {
            Ok(true) => log("soundtrack: original Rush a Difficulty restored"),
            Ok(false) => {}
            Err(e) => log(&format!("warning: soundtrack restore failed: {e}")),
        }
    }
    Ok(())
}

/// DXVK serves the game's Direct3D 9 on Linux; on Windows the real D3D9
/// does (a local d3d9.dll from an earlier Linux run is removed).
fn install_dxvk(
    runtime: &Path,
    launcher_dir: &Path,
    linux: bool,
    log: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let marker = runtime.join(DXVK_MARKER);
    let destination = runtime.join("d3d9.dll");
    if !linux {
        if marker.is_file() {
            let _ = fs::remove_file(&destination);
            let _ = fs::remove_file(&marker);
            log("dxvk: removed (running on Windows)");
        }
        return Ok(());
    }
    let source = launcher_dir.join("linux").join("d3d9.dll");
    if !source.is_file() {
        return Err(format!(
            "DXVK is missing: {}. Keep the linux folder next to OutRunLauncher.exe.",
            source.display()
        ));
    }
    if freshen(&source, &destination)? {
        log("dxvk: installed");
    }
    fs::write(&marker, b"d3d9.dll is managed by OutRunLauncher\n").map_err(|e| e.to_string())
}

/// The folder Tweaks should read HD textures from (`TextureBaseFolder`,
/// which holds `load`), for a pack unzipped anywhere: the folder it was
/// unzipped to (`textures/load` inside), its `textures` folder, or `load`
/// itself. None when it holds no `load` folder.
pub fn texture_base(source: &Path) -> Option<PathBuf> {
    if source.join("textures").join("load").is_dir() {
        Some(source.join("textures"))
    } else if source.join("load").is_dir() {
        Some(source.to_path_buf())
    } else if source.is_dir()
        && source
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("load"))
    {
        source.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

/// The HD pack's base folder when it is on and found; a missing pack is
/// logged and the game starts with its own textures.
fn hd_texture_base(
    settings: &Settings,
    launcher_dir: &Path,
    log: &mut dyn FnMut(&str),
) -> Option<PathBuf> {
    let source = settings.hd_textures_source(launcher_dir)?;
    let base = texture_base(&source);
    match &base {
        Some(base) => log(&format!("hd textures: {}", base.display())),
        None => log(&format!(
            "warning: hd textures are on but {} holds no textures\\load folder; download the pack from {} and choose its folder",
            source.display(),
            crate::config::HD_PACK_PAGE
        )),
    }
    base
}

/// The (section, key) pairs the launcher writes to `OutRun2006Tweaks.ini`.
fn owned_keys() -> Vec<(String, String)> {
    ini::section_keys(&texture_keys(&Settings::default().apply_tweaks(""), None))
}

/// Tweaks reads `OutRun2006Tweaks.user.ini` over the main file (the F11
/// overlay saves there), so the launcher's own keys are dropped from it
/// and its settings win; the overlay's other choices stay.
fn reset_overlay_keys(user_ini: &Path, log: &mut dyn FnMut(&str)) -> Result<(), String> {
    let Ok(text) = fs::read_to_string(user_ini) else {
        return Ok(());
    };
    let mut cleaned = text.clone();
    let mut removed = 0;
    for (section, key) in owned_keys() {
        if ini::section_key(&cleaned, &section, &key).is_some() {
            cleaned = ini::remove_section_key(&cleaned, &section, &key);
            removed += 1;
        }
    }
    if removed > 0 {
        fs::write(user_ini, cleaned)
            .map_err(|e| format!("cannot write {}: {e}", user_ini.display()))?;
        log(&format!(
            "tweaks: {removed} overlay settings the launcher owns were reset"
        ));
    }
    Ok(())
}

/// Points Tweaks at the HD pack where it was unzipped (no copy), or at the
/// game's own `textures` folder. The pack draws Clarissa as in the
/// Japanese release, so she is restored with it.
pub fn texture_keys(ini: &str, pack: Option<&Path>) -> String {
    let folder = pack.map_or_else(|| "textures".to_string(), |p| p.display().to_string());
    let t = ini::set_section_key(ini, "Graphics", "TextureBaseFolder", &folder);
    ini::set_section_key(
        &t,
        "Misc",
        "RestoreJPClarissa",
        if pack.is_some() { "true" } else { "false" },
    )
}

/// The DLL overrides the game needs under Wine: the mod's `dinput8.dll`
/// and DXVK's `d3d9.dll` load as native, and Wine's Mono and Gecko stay
/// away (no download prompts).
pub const WINE_OVERRIDES: &str = "dinput8=n,b;d3d9=n,b;mscoree,mshtml=d";

/// Whether a rule's DLL name is `dll`, in any case, with or without
/// `.dll` (Wine accepts both).
fn same_dll(name: &str, dll: &str) -> bool {
    let len = name.len();
    let bare = if len > 4
        && name.is_char_boundary(len - 4)
        && name[len - 4..].eq_ignore_ascii_case(".dll")
    {
        &name[..len - 4]
    } else {
        name
    };
    bare.eq_ignore_ascii_case(dll)
}

/// `WINEDLLOVERRIDES` for the game: our rules first, then the player's
/// own, minus their rules for the DLLs ours cover (Wine doesn't promise
/// which of two rules for one DLL wins). Everything else they set is kept.
pub fn wine_overrides(existing: Option<&str>) -> String {
    const OURS: [&str; 4] = ["dinput8", "d3d9", "mscoree", "mshtml"];
    let mut result = vec![WINE_OVERRIDES.to_owned()];
    for rule in existing.into_iter().flat_map(|rules| rules.split(';')) {
        if rule.trim().is_empty() {
            continue;
        }
        let Some((names, action)) = rule.split_once('=') else {
            result.push(rule.to_owned());
            continue;
        };
        let kept: Vec<_> = names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty() && !OURS.iter().any(|o| same_dll(name, o)))
            .collect();
        if !kept.is_empty() {
            result.push(format!("{}={action}", kept.join(",")));
        }
    }
    result.join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("or-tweaks-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// A launcher folder with a fake tweaks build and VC runtime.
    fn launcher(at: &Path) -> std::path::PathBuf {
        let dir = at.join("launcher");
        fs::create_dir_all(dir.join(TWEAKS_DIR)).unwrap();
        fs::create_dir_all(dir.join(VC_DIR)).unwrap();
        fs::write(dir.join(TWEAKS_DIR).join("dinput8.dll"), b"dll").unwrap();
        fs::write(dir.join(TWEAKS_DIR).join("OR2006C2C.exe"), b"exe").unwrap();
        fs::write(dir.join(TWEAKS_DIR).join("OutRun2006Tweaks.ini"), "[Performance]\nFramerateLimit = 0\n\n[CDTracks]\n01_Splash_wave.ogg = Splash Wave\nLast_Wave.ogg = Last Wave\n").unwrap();
        fs::write(
            dir.join(TWEAKS_DIR).join("OutRun2006Tweaks.lods.ini"),
            "[Stage 0]\n",
        )
        .unwrap();
        for dll in VC_DLLS {
            fs::write(dir.join(VC_DIR).join(dll), b"vc").unwrap();
        }
        dir
    }

    #[test]
    fn installs_and_is_idempotent() {
        let at = temp("install");
        let launcher_dir = launcher(&at);
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let mut lines = Vec::new();
        let mut log = |line: &str| lines.push(line.to_string());

        install(
            &runtime,
            &launcher_dir,
            false,
            &Settings::default(),
            &mut log,
        )
        .unwrap();
        assert!(runtime.join("OR2006C2C.exe").is_file());
        assert!(runtime.join("dinput8.dll").is_file());
        assert!(runtime.join("msvcp140.dll").is_file());
        assert!(runtime.join("OutRun2006Tweaks.ini").is_file());
        assert!(runtime.join("outrun2006.ini").is_file());
        let ini = fs::read_to_string(runtime.join("OutRun2006Tweaks.ini")).unwrap();
        assert!(ini.contains("FramerateLimit = 60"));
        assert!(ini.contains("01_Splash_wave.ogg"));
        assert!(
            ini.contains("[Bugfixes]"),
            "falls back to sections the base lacked"
        );
        assert_eq!(
            fs::read_to_string(runtime.join("outrun2006.ini")).unwrap(),
            Settings::default().outrun2006_ini()
        );

        // A second run changes nothing and reports nothing.
        drop(log);
        let mut log = |line: &str| println!("{line}");
        let mut second = Vec::new();
        {
            let mut log = |line: &str| second.push(line.to_string());
            install(
                &runtime,
                &launcher_dir,
                false,
                &Settings::default(),
                &mut log,
            )
            .unwrap();
        }
        assert!(second.is_empty(), "{second:?}");

        // Changing a setting rewrites only the INI keys.
        let tuned = Settings::parse_file("fps_limit = unlimited\ndraw_distance = 8\n").0;
        install(&runtime, &launcher_dir, false, &tuned, &mut log).unwrap();
        let ini = fs::read_to_string(runtime.join("OutRun2006Tweaks.ini")).unwrap();
        assert!(ini.contains("FramerateLimit = 0"));
        assert!(ini.contains("FramerateInterpolation = true"));
        assert!(ini.contains("DrawDistanceIncrease = 8"));

        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn launcher_keys_win_over_the_overlay_file() {
        let at = temp("overlay");
        let launcher_dir = launcher(&at);
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let user = runtime.join(USER_INI);
        fs::write(
            &user,
            "# overlay\n[Performance]\nFramerateLimit = 144\n\n[Overlay]\nEnabled = false\n\n[Graphics]\nSkyGlowFactor = 0\nUIScalingMode = 2\n",
        )
        .unwrap();
        let mut lines = Vec::new();
        let mut log = |line: &str| lines.push(line.to_string());
        install(
            &runtime,
            &launcher_dir,
            false,
            &Settings::default(),
            &mut log,
        )
        .unwrap();
        let text = fs::read_to_string(&user).unwrap();
        assert!(!text.contains("FramerateLimit"));
        assert!(!text.contains("SkyGlowFactor"));
        assert!(text.contains("# overlay"));
        assert!(text.contains("Enabled = false"));
        assert!(text.contains("UIScalingMode = 2"));
        assert!(
            lines
                .iter()
                .any(|l| l == "tweaks: 2 overlay settings the launcher owns were reset")
        );
        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn missing_tweaks_is_an_error() {
        let at = temp("missing");
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let err = install(&runtime, &at, false, &Settings::default(), &mut |_| {}).unwrap_err();
        assert!(err.contains("tweaks"), "{err}");
        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn dxvk_follows_the_platform() {
        let at = temp("dxvk");
        let launcher_dir = launcher(&at);
        fs::create_dir_all(launcher_dir.join("linux")).unwrap();
        fs::write(launcher_dir.join("linux").join("d3d9.dll"), b"dxvk").unwrap();
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let mut log = |_line: &str| {};

        install(
            &runtime,
            &launcher_dir,
            true,
            &Settings::default(),
            &mut log,
        )
        .unwrap();
        assert_eq!(fs::read(runtime.join("d3d9.dll")).unwrap(), b"dxvk");
        assert!(runtime.join(DXVK_MARKER).is_file());

        // Moving the same runtime to Windows removes it again.
        install(
            &runtime,
            &launcher_dir,
            false,
            &Settings::default(),
            &mut log,
        )
        .unwrap();
        assert!(!runtime.join("d3d9.dll").exists());
        assert!(!runtime.join(DXVK_MARKER).exists());

        // Without the linux folder it is an error on Linux only.
        fs::remove_file(launcher_dir.join("linux").join("d3d9.dll")).unwrap();
        assert!(
            install(
                &runtime,
                &launcher_dir,
                true,
                &Settings::default(),
                &mut log
            )
            .is_err()
        );
        assert!(
            install(
                &runtime,
                &launcher_dir,
                false,
                &Settings::default(),
                &mut log
            )
            .is_ok()
        );
        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn the_hd_pack_is_read_where_it_was_unzipped() {
        let at = temp("textures");
        let launcher_dir = launcher(&at);
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let ini =
            |runtime: &Path| fs::read_to_string(runtime.join("OutRun2006Tweaks.ini")).unwrap();

        // The zip as published: textures/load inside the folder chosen.
        let unzipped = at.join("OR2-HD-GUI");
        fs::create_dir_all(unzipped.join("textures/load/spr_a")).unwrap();
        assert_eq!(texture_base(&unzipped), Some(unzipped.join("textures")));
        assert_eq!(
            texture_base(&unzipped.join("textures")),
            Some(unzipped.join("textures"))
        );
        assert_eq!(
            texture_base(&unzipped.join("textures/load")),
            Some(unzipped.join("textures"))
        );
        assert_eq!(texture_base(&at.join("nope")), None);

        let mut lines = Vec::new();
        let mut log = |line: &str| lines.push(line.to_string());
        let mut s = Settings::parse_file("hd_textures = on\n").0;
        s.hd_textures_folder = Some(unzipped.clone());
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        let tuned = ini(&runtime);
        assert!(tuned.contains(&format!(
            "TextureBaseFolder = {}",
            unzipped.join("textures").display()
        )));
        assert!(tuned.contains("RestoreJPClarissa = true"));
        // Nothing is copied into the runtime.
        assert!(!runtime.join("textures").exists());

        // Off: the game's own textures folder.
        s.hd_textures = Some(false);
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(ini(&runtime).contains("TextureBaseFolder = textures"));
        assert!(ini(&runtime).contains("RestoreJPClarissa = false"));

        // On without a pack: a warning, and the game still starts.
        s.hd_textures = Some(true);
        s.hd_textures_folder = Some(at.join("nope"));
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(ini(&runtime).contains("TextureBaseFolder = textures"));
        assert!(lines.iter().any(|l| l.starts_with("warning: hd textures")));

        // No folder chosen: textures-pack next to the launcher.
        fs::create_dir_all(launcher_dir.join("textures-pack/load")).unwrap();
        s.hd_textures_folder = None;
        let mut quiet = |_: &str| {};
        install(&runtime, &launcher_dir, false, &s, &mut quiet).unwrap();
        assert!(ini(&runtime).contains(&format!(
            "TextureBaseFolder = {}",
            launcher_dir.join("textures-pack").display()
        )));
        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn base_ini_covers_our_sections() {
        let s = Settings::default();
        let tuned = s.apply_tweaks(BASE_INI);
        for section in [
            "Performance",
            "Controls",
            "Graphics",
            "Audio",
            "CDSwitcher",
            "Window",
            "Misc",
            "Overlay",
            "Bugfixes",
            "CDTracks",
        ] {
            assert!(tuned.contains(&format!("[{section}]")), "{section} missing");
        }
        assert!(tuned.contains("01_Splash_wave.ogg"));
    }

    #[test]
    fn the_input_api_follows_the_platform_unless_chosen() {
        let at = temp("backend");
        let launcher_dir = launcher(&at);
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(launcher_dir.join("linux")).unwrap();
        fs::write(launcher_dir.join("linux").join("d3d9.dll"), b"dxvk").unwrap();
        let backend = |runtime: &Path| {
            ini::section_key(
                &fs::read_to_string(runtime.join("OutRun2006Tweaks.ini")).unwrap(),
                "Controls",
                "InputBackend",
            )
        };
        let mut quiet = |_: &str| {};
        let unset = Settings::default();
        install(&runtime, &launcher_dir, true, &unset, &mut quiet).unwrap();
        assert_eq!(backend(&runtime).as_deref(), Some("3"), "XInput under Wine");
        install(&runtime, &launcher_dir, false, &unset, &mut quiet).unwrap();
        assert_eq!(backend(&runtime).as_deref(), Some("0"), "WGI on Windows");
        let chosen = Settings::parse_file("input_backend = dinput\n").0;
        install(&runtime, &launcher_dir, true, &chosen, &mut quiet).unwrap();
        assert_eq!(backend(&runtime).as_deref(), Some("2"), "a choice wins");
        fs::remove_dir_all(&at).unwrap();
    }

    #[test]
    fn wine_overrides_put_ours_first_and_drop_conflicting_rules() {
        assert_eq!(wine_overrides(None), WINE_OVERRIDES);
        assert_eq!(wine_overrides(Some("")), WINE_OVERRIDES);
        assert_eq!(
            wine_overrides(Some(
                "DINPUT8=b;xinput1_3=n,b;d3d9,d3d11=b;MSCOREE,mshtml=;winemenubuilder.exe=d"
            )),
            format!("{WINE_OVERRIDES};xinput1_3=n,b;d3d11=b;winemenubuilder.exe=d")
        );
        // A rule without '=' is passed on as written.
        assert_eq!(
            wine_overrides(Some("odd;;foo=n")),
            format!("{WINE_OVERRIDES};odd;foo=n")
        );
    }

    #[test]
    fn wine_overrides_recognise_names_spelled_with_dll() {
        assert_eq!(
            wine_overrides(Some("D3D9.DLL=b;dinput8.dll=b;xinput1_3.dll=n;d3d9x.dll=b")),
            format!("{WINE_OVERRIDES};xinput1_3.dll=n;d3d9x.dll=b")
        );
    }
}
