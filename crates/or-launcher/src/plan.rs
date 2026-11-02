//! Everything a launch produces for the game, worked out without touching
//! a disk: the two INI files, the environment and DLL overrides, the
//! Wine controller values, the components placed in the runtime copy and
//! the Linux helper's gamescope filter. `tweaks::install` and `main.rs`
//! write and pass the same values; the end-to-end option tests and the
//! golden snapshots (`tests/snapshots/`) read them from here.

use std::fmt::Write as _;
use std::path::Path;

use crate::config::{Scaler, Settings};
use crate::tweaks;

/// What the launcher would give the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// `outrun2006.ini` (seeded: no copy of the game's).
    pub outrun2006_ini: String,
    /// `OutRun2006Tweaks.ini` over the launcher's own base.
    pub tweaks_ini: String,
    /// The game's environment (name, value).
    pub env: Vec<(String, String)>,
    /// Values seeded under `HKLM\System\CurrentControlSet\Services\winebus`
    /// (Wine only, where absent).
    pub winebus: Vec<(String, u32)>,
    /// Files placed in the runtime copy, and their source.
    pub components: Vec<String>,
    /// The game's command line after the EXE (OutRun takes none).
    pub args: Vec<String>,
    /// gamescope's `-F` filter (Linux helper).
    pub gamescope_filter: &'static str,
}

/// The plan for `settings` on Windows (`wine` false) or Wine/Proton;
/// `pack` is the HD pack's texture base when it is on and found.
pub fn plan(settings: &Settings, wine: bool, pack: Option<&Path>) -> Plan {
    let mut env = Vec::new();
    let mut winebus = Vec::new();
    let mut components = vec![
        "OR2006C2C.exe <- tweaks/OR2006C2C.exe (OutRun2006Tweaks, DRM-free)".to_string(),
        "dinput8.dll <- tweaks/dinput8.dll (OutRun2006Tweaks)".to_string(),
        "OutRun2006Tweaks.lods.ini <- tweaks/".to_string(),
    ];
    components.extend(tweaks::VC_DLLS.iter().map(|dll| format!("{dll} <- vc/")));
    if wine {
        env.push(("WINEDLLOVERRIDES".into(), tweaks::wine_overrides(None)));
        env.push(("WINEDEBUG".into(), "-all".into()));
        winebus.extend(
            rea_launcher::wine::SDL_ONLY
                .iter()
                .map(|(name, value)| (name.to_string(), *value)),
        );
        components.push("d3d9.dll <- linux/d3d9.dll (DXVK)".into());
    }
    components.push(if settings.soundtrack_fix.unwrap_or(true) {
        "Sound/14_Rush_a_Difficulty_1989.ogg patched <- soundtrack/ (xdelta3)".into()
    } else {
        "Sound/14_Rush_a_Difficulty_1989.ogg: the game's own".into()
    });
    Plan {
        outrun2006_ini: settings.outrun2006_ini(None),
        tweaks_ini: tweaks::tweaks_ini(tweaks::BASE_INI, settings, pack, wine)
            .to_text()
            .expect("parsed from text"),
        env,
        winebus,
        components,
        args: Vec::new(),
        gamescope_filter: settings.scaler.unwrap_or(Scaler::Linear).key(),
    }
}

impl Plan {
    /// The plan as a text file, for snapshots.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "## outrun2006.ini\n{}", self.outrun2006_ini);
        let _ = writeln!(out, "## environment");
        for (name, value) in &self.env {
            let _ = writeln!(out, "{name}={value}");
        }
        let _ = writeln!(out, "\n## registry (winebus, Wine only, where absent)");
        for (name, value) in &self.winebus {
            let _ = writeln!(out, "{name} = dword:{value}");
        }
        let _ = writeln!(out, "\n## runtime components");
        for line in &self.components {
            let _ = writeln!(out, "{line}");
        }
        let _ = writeln!(
            out,
            "\n## command line\nOR2006C2C.exe{}",
            self.args.join(" ")
        );
        let _ = writeln!(
            out,
            "\n## gamescope (Linux helper)\n-F {}",
            self.gamescope_filter
        );
        let _ = writeln!(out, "\n## OutRun2006Tweaks.ini\n{}", self.tweaks_ini);
        out
    }

    /// The value of `key` in `[section]` of the Tweaks INI.
    pub fn tweak(&self, section: &str, key: &str) -> Option<String> {
        rea_launcher::ini::Ini::parse(&self.tweaks_ini)
            .get(section, key)
            .map(str::to_owned)
    }

    /// The value of `key` in `outrun2006.ini`.
    pub fn game_ini(&self, key: &str) -> Option<String> {
        rea_launcher::ini::Ini::parse(&self.outrun2006_ini)
            .get("", key)
            .map(str::to_owned)
    }
}
