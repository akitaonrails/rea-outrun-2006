//! Wiring the private runtime copy: the bundled OutRun2006Tweaks files,
//! the VC++ runtime the mod needs, DXVK on Linux, the two INI files, the
//! HD texture pack and the soundtrack fix.
//!
//! Every step is idempotent: files are replaced only when they differ, and
//! the INI files keep the user's edits to keys this launcher does not own.

use std::fs;
use std::path::Path;

use crate::config::{HdTextures, Settings};
use crate::runtime;
use crate::soundtrack;

/// Files of the OutRun2006Tweaks build this launcher is wired for, next to
/// the launcher (`tweaks/`). The package pins their source and digest
/// (`packaging/tweaks.txt`).
pub const TWEAKS_DIR: &str = "tweaks";
const VC_DIR: &str = "vc";
const DXVK_MARKER: &str = ".or-dxvk";
const TEXTURES_MARKER: &str = ".or-hd-textures";

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
    let tuned = settings.apply_tweaks(&base);
    fs::write(&ini_path, &tuned)
        .map_err(|e| format!("cannot write {}: {e}", ini_path.display()))?;

    // The game's own settings, written whole (the mod deletes stale copies
    // older than three months; ours is fresh every run).
    let game_ini = runtime.join("outrun2006.ini");
    fs::write(&game_ini, settings.outrun2006_ini())
        .map_err(|e| format!("cannot write {}: {e}", game_ini.display()))?;

    install_dxvk(runtime, launcher_dir, linux, log)?;
    install_textures(runtime, launcher_dir, settings, log)?;

    if settings.soundtrack_fix.unwrap_or(true) {
        match soundtrack::apply(runtime, launcher_dir) {
            soundtrack::Applied::Fixed => log("soundtrack: Rush a Difficulty restored"),
            soundtrack::Applied::AlreadyFixed => {}
            soundtrack::Applied::NoTool(what) => log(&format!(
                "warning: soundtrack fix skipped, {what} not found"
            )),
            soundtrack::Applied::Failed(e) => log(&format!("warning: soundtrack fix failed: {e}")),
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

/// The HD texture pack into `textures/load` (the mod's replacement folder).
/// `Bundled` uses the pack shipped or dropped next to the launcher;
/// `Folder` any folder of replacement textures; off removes a pack the
/// launcher installed (a marker says it is ours to remove).
fn install_textures(
    runtime: &Path,
    launcher_dir: &Path,
    settings: &Settings,
    log: &mut dyn FnMut(&str),
) -> Result<(), String> {
    let load = runtime.join("textures").join("load");
    let marker = runtime.join(TEXTURES_MARKER);
    let source = match &settings.hd_textures {
        Some(HdTextures::Bundled) => Some(launcher_dir.join("textures-pack")),
        Some(HdTextures::Folder(dir)) => Some(dir.clone()),
        None | Some(HdTextures::Off) => None,
    };
    let Some(source) = source else {
        if marker.is_file() {
            fs::remove_dir_all(&load)
                .map_err(|e| format!("cannot remove {}: {e}", load.display()))?;
            let _ = fs::remove_file(&marker);
            log("hd textures: removed");
        }
        return Ok(());
    };
    // The pack may be its `load` folder itself or the folder above it.
    let from = if source.join("load").is_dir() {
        source.join("load")
    } else {
        source.clone()
    };
    if !from.is_dir() {
        return Err(format!(
            "the HD texture folder does not exist: {}",
            source.display()
        ));
    }
    let copied = runtime::mirror(&from, &load).map_err(|e| e.to_string())?;
    fs::write(&marker, b"textures/load is managed by OutRunLauncher\n")
        .map_err(|e| e.to_string())?;
    log(&format!(
        "hd textures: {copied} files (from {})",
        from.display()
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HdTextures;

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
        assert!(!ini.contains("Last_Wave.ogg"));
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
    fn textures_install_and_clean_up() {
        let at = temp("textures");
        let launcher_dir = launcher(&at);
        let pack = launcher_dir.join("textures-pack");
        fs::create_dir_all(pack.join("cs_CS_BEAC_pmt")).unwrap();
        fs::write(pack.join("cs_CS_BEAC_pmt").join("a.dds"), b"dds").unwrap();
        let runtime = at.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        let mut log = |_line: &str| {};

        let mut s = Settings::parse_file("hd_textures = on\n").0;
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(runtime.join("textures/load/cs_CS_BEAC_pmt/a.dds").is_file());

        // A user file in the load folder survives switching off? No: the
        // marker says we own the folder; off removes it wholesale.
        s.hd_textures = Some(HdTextures::Off);
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(!runtime.join("textures/load").exists());

        // A folder of the user's choosing works too, with or without a
        // `load` subfolder.
        let own = at.join("own");
        fs::create_dir_all(&own).unwrap();
        fs::write(own.join("b.dds"), b"dds").unwrap();
        s.hd_textures = Some(HdTextures::Folder(own.clone()));
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(runtime.join("textures/load/b.dds").is_file());
        let own2 = at.join("own2");
        fs::create_dir_all(own2.join("load")).unwrap();
        fs::write(own2.join("load").join("c.dds"), b"dds").unwrap();
        s.hd_textures = Some(HdTextures::Folder(own2));
        install(&runtime, &launcher_dir, false, &s, &mut log).unwrap();
        assert!(runtime.join("textures/load/c.dds").is_file());

        // A missing folder is an error, not silence.
        s.hd_textures = Some(HdTextures::Folder(at.join("nope")));
        assert!(install(&runtime, &launcher_dir, false, &s, &mut log).is_err());
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
        assert!(!tuned.contains("Last_Wave.ogg"));
    }
}
