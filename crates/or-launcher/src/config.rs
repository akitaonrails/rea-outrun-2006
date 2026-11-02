//! Command line and the settings file (`launcher.cfg`).
//!
//! One way flow: the launcher's settings become the game's two INI files
//! (`outrun2006.ini` for the video mode, `OutRun2006Tweaks.ini` for the
//! OutRun2006Tweaks mod). The file is plain `key = value` lines with `#`
//! comments, written once from a commented template so users can edit it;
//! the launcher later changes only the keys it owns. Unknown keys are
//! ignored; values it cannot read are reported and ignored.

use std::path::PathBuf;

use crate::ini;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub game: Option<PathBuf>,
    pub runtime: Option<PathBuf>,
    /// Where the player's saves live (the game's `SaveGame` folder,
    /// mirrored in before and back after play).
    pub saves: Option<PathBuf>,

    // Picture
    /// Video resolution; `None` is the game's own 640x480.
    pub resolution: Option<(u32, u32)>,
    /// Exclusive fullscreen instead of a borderless window.
    pub fullscreen: Option<bool>,
    /// `DX/ANTIALIASING`: 0 off, 2 pairs with Tweaks' transparency
    /// supersampling.
    pub antialiasing: Option<u32>,
    /// Frame-rate cap; `0` is unlimited (with Tweaks' interpolation).
    pub fps_limit: Option<u32>,
    pub vsync: Option<Vsync>,
    /// 0 to 8, how much further the stage draws.
    pub draw_distance: Option<u32>,
    /// Car reflection texture size, 0 keeps the game's 128x128.
    pub reflections: Option<u32>,
    /// Anisotropic filtering, 1-16, 0 leaves the game's own.
    pub anisotropy: Option<u32>,
    /// The console versions' glowing sky.
    pub sky_glow: Option<bool>,
    /// The Xbox releases' brighter picture.
    pub xbox_brightness: Option<bool>,
    /// The hi-def passenger models the PC release left unused.
    pub hd_characters: Option<bool>,
    pub hd_textures: Option<HdTextures>,

    // Controls
    /// Steering deadzone in percent (game default 20).
    pub deadzone: Option<u32>,
    pub vibration: Option<Feedback>,
    /// 0 to 10.
    pub vibration_strength: Option<u32>,
    pub impulse_triggers: Option<Feedback>,
    pub input_backend: Option<Backend>,
    /// Detect controllers plugged in after launch.
    pub hot_plug: Option<bool>,

    // Extras
    /// Change music mid-race (Back / RS+Back).
    pub cd_switcher: Option<bool>,
    pub music_shuffle: Option<bool>,
    pub skip_logos: Option<bool>,
    /// No countdown: coast for as long as you like.
    pub free_drive: Option<bool>,
    /// Horn outside the girlfriend missions.
    pub horn: Option<bool>,
    /// Roadside animations from any game mode.
    pub random_anims: Option<bool>,
    /// Character model choice on the Edit License screen.
    pub character_select: Option<bool>,
    /// OutRun miles under the menu's sign-in text.
    pub show_miles: Option<bool>,
    /// Restore the first two seconds of "Rush a Difficulty".
    pub soundtrack_fix: Option<bool>,
    /// Linux only: how `or-play`'s gamescope enlarges the picture.
    pub scaler: Option<Scaler>,
}

/// `vsync`: on, off, or every second refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vsync {
    On,
    Off,
    Half,
}

/// A rumble mode as the Tweaks INI numbers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback {
    Off,
    On,
    Swapped,
    Merged,
}

impl Feedback {
    pub const ALL: [Feedback; 4] = [
        Feedback::Off,
        Feedback::On,
        Feedback::Swapped,
        Feedback::Merged,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Feedback::Off => "off",
            Feedback::On => "on",
            Feedback::Swapped => "swapped",
            Feedback::Merged => "merged",
        }
    }

    fn parse(value: &str) -> Option<Feedback> {
        Feedback::ALL
            .into_iter()
            .find(|f| f.key() == value.trim().to_ascii_lowercase())
    }

    /// The INI value: 0 disables, 1 enables, 2 swaps, 3 merges.
    pub fn ini(self) -> u32 {
        match self {
            Feedback::Off => 0,
            Feedback::On => 1,
            Feedback::Swapped => 2,
            Feedback::Merged => 3,
        }
    }
}

/// `input_backend`: which API Tweaks' SDL input system reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    WindowsGaming,
    Raw,
    DirectInput,
    XInput,
}

impl Backend {
    pub const ALL: [Backend; 4] = [
        Backend::WindowsGaming,
        Backend::Raw,
        Backend::DirectInput,
        Backend::XInput,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Backend::WindowsGaming => "wgi",
            Backend::Raw => "raw",
            Backend::DirectInput => "dinput",
            Backend::XInput => "xinput",
        }
    }

    fn parse(value: &str) -> Option<Backend> {
        let value = value.trim().to_ascii_lowercase();
        if value == "auto" {
            return Some(Backend::WindowsGaming);
        }
        Backend::ALL.into_iter().find(|b| b.key() == value)
    }

    /// The INI value (0-3).
    pub fn ini(self) -> u32 {
        match self {
            Backend::WindowsGaming => 0,
            Backend::Raw => 1,
            Backend::DirectInput => 2,
            Backend::XInput => 3,
        }
    }
}

/// Where HD textures come from (`hd_textures`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HdTextures {
    Off,
    /// The community texture pack shipped with the launcher (`textures-pack`).
    Bundled,
    /// A folder (or zip) of replacement textures for `textures/load`.
    Folder(PathBuf),
}

/// `scaler` (Linux, gamescope's filter): sharp pixels, sharp pixels of even
/// size, FSR, or plain smoothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scaler {
    Nearest,
    Pixel,
    Fsr,
    Linear,
}

impl Scaler {
    pub const ALL: [Scaler; 4] = [Scaler::Nearest, Scaler::Pixel, Scaler::Fsr, Scaler::Linear];

    /// The value in launcher.cfg (and gamescope's `-F` name).
    pub fn key(self) -> &'static str {
        match self {
            Scaler::Nearest => "nearest",
            Scaler::Pixel => "pixel",
            Scaler::Fsr => "fsr",
            Scaler::Linear => "linear",
        }
    }

    fn parse(value: &str) -> Option<Scaler> {
        let value = value.trim().to_ascii_lowercase();
        Scaler::ALL.into_iter().find(|s| s.key() == value)
    }
}

impl HdTextures {
    pub fn parse(value: &str) -> HdTextures {
        match parse_bool(value) {
            Some(true) => HdTextures::Bundled,
            Some(false) => HdTextures::Off,
            None => HdTextures::Folder(value.into()),
        }
    }
}

pub const FILE_NAME: &str = "launcher.cfg";

/// The file written on first run.
pub const TEMPLATE: &str = r#"# OutRun 2006 launcher settings.
#
# One setting per line as key = value; lines starting with # are comments.
# Command-line options (--game, --resolution) override this file for one
# run. A launcher.cfg next to OutRunLauncher.exe is used instead of this
# one when it exists (for portable installs).
#
# Everything below is applied to the launcher's private copy of the game;
# your game folder is never changed.

# The folder with your installed OutRun 2006 (OR2006C2C.exe). Filled in by
# --game; the game folder is only read, never changed.
game =

# Where the launcher keeps its private, writable copy of the game. Default:
# %LOCALAPPDATA%\OutRunLauncher\OutRun2006
#runtime =

# Where your saves (License.dat, the SaveGame folder's contents) are kept.
# They are copied into the game before it starts and back when it exits.
# Default: %USERPROFILE%\Saved Games\OutRun 2006
#saves =

# Video resolution, e.g. 1920x1080 or 3840x2160; original keeps the game's
# own 640x480. The game runs as a borderless window at this size (see
# fullscreen).
#resolution = 1920x1080

# true switches to exclusive fullscreen (the borderless window is easier on
# multi-monitor desktops and works the same under Wine).
#fullscreen = false

# Anti-aliasing: 0 off, 2 multisampled (pairs with the mod's transparency
# supersampling, which sharpens trackside cloth and barriers).
#antialiasing = 0

# Frame-rate cap in frames per second. The game's driving runs at a fixed
# 60 Hz; above 60 the mod interpolates the picture. unlimited (0) turns the
# cap off entirely and leaves pacing to vsync or an external limiter.
#fps_limit = 60

# vsync: on waits for the screen's refresh, off doesn't, half waits for
# every second refresh.
#vsync = on

# How much further down the stage the world draws, 0 to 8. The mod
# recommends 4-8: further out, scenery can pop in as its low-detail model.
#draw_distance = 4

# Resolution of the car's reflections: original (128), 1024 or 2048.
#reflections = 2048

# Anisotropic filtering, 1 to 16, or off for the game's own.
#anisotropy = 16

# The console versions' glowing sky effect.
#sky_glow = true

# The Xbox releases' brighter picture.
#xbox_brightness = false

# The hi-def passenger models the PC release shipped unused.
#hd_characters = true

# HD texture pack: on uses the community pack shipped with the launcher
# (or give a folder of replacement textures). off keeps the game's own.
#hd_textures = off

# Steering deadzone in percent, 0 to 50 (the game's default is 20).
#deadzone = 20

# Gamepad rumble: off, on, swapped (left/right motors), or merged.
#vibration = on

# Rumble strength, 0 to 10.
#vibration_strength = 7

# Xbox Series impulse triggers: off, on, swapped (recommended), or merged.
#impulse_triggers = swapped

# Input API for the mod's controller system: wgi (Windows.Gaming.Input,
# the default), raw, dinput or xinput. Try another if a pad isn't seen.
#input_backend = wgi

# Detect controllers plugged in after the game starts.
#hot_plug = true

# Change the music during a race (Back = next track, RS+Back = previous).
#cd_switcher = true

# Shuffle the race music each launch.
#music_shuffle = false

# Skip the boot logos (the data still loads; a white screen for a moment).
#skip_logos = true

# Free drive: no countdown timer on the stages.
#free_drive = true

# The horn works outside the "beep the horn!" requests.
#horn = true

# Roadside animations can come from any game mode.
#random_anims = true

# Change character model on the Edit License screen (alternate models have
# quirks; back up your License.dat first).
#character_select = true

# Show OutRun miles next to the menu's sign-in text.
#show_miles = true

# Restore the first two seconds of "Rush a Difficulty" (applies a small
# official patch to the launcher's copy of the song).
#soundtrack_fix = true
"#;

pub const USAGE: &str = concat!(
    "OutRunLauncher ",
    env!("CARGO_PKG_VERSION"),
    "\n\n",
    "\
OutRunLauncher [options]

  --game <folder>           the folder with your installed OutRun 2006 (remembered)
  --saves <folder>          where your saves are kept
  --runtime <folder>        where the launcher keeps its private copy
  --resolution <WxH>        video resolution, e.g. 1920x1080, or original
  --fullscreen on|off       exclusive fullscreen instead of a borderless window
  --antialiasing <0|2>      anti-aliasing
  --fps-limit <n>|unlimited frame-rate cap, e.g. 60 or 120
  --vsync on|off|half       wait for the screen's refresh
  --draw-distance <0-8>     how much further the stage draws
  --reflections <0|1024|2048>  car reflection resolution
  --anisotropy <0-16>       anisotropic filtering
  --sky-glow on|off         the console versions' glowing sky
  --xbox-brightness on|off  the Xbox releases' brighter picture
  --hd-characters on|off    hi-def passenger models
  --hd-textures <folder>    HD texture pack: on, off, or a folder
  --deadzone <0-50>         steering deadzone in percent
  --vibration <mode>        rumble: off, on, swapped or merged
  --vibration-strength <0-10>
  --impulse-triggers <mode> Xbox Series impulse triggers
  --input-backend <name>    wgi, raw, dinput or xinput
  --hot-plug on|off         detect pads plugged in after launch
  --cd-switcher on|off      change music mid-race
  --music-shuffle on|off    shuffle the race music
  --skip-logos on|off       skip the boot logos
  --free-drive on|off       no countdown timer
  --horn on|off             usable horn
  --random-anims on|off     roadside animations from any mode
  --character-select on|off character model on the license screen
  --show-miles on|off       OutRun miles in the menu
  --soundtrack-fix on|off   restore Rush a Difficulty's first two seconds

These override launcher.cfg for one run (only --game is remembered). Without
options the settings window opens first.

  --no-window               play with the saved settings, no window
  --settings [--linux]      only the settings window (exit 0: play, 3: closed;
                           --linux adds the gamescope scaler, for or-play)

OutRunLauncher --check [--game <folder>]
  checks the game folder and tells you whether its OR2006C2C.exe is supported

Other settings are in launcher.cfg (see the comments in it)."
);

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn parse_resolution(value: &str) -> Option<(u32, u32)> {
    let lower = value.trim().to_ascii_lowercase();
    if lower == "original" || lower == "off" {
        return Some(ORIGINAL);
    }
    let (w, h) = lower.split_once('x')?;
    let w: u32 = w.trim().parse().ok()?;
    let h: u32 = h.trim().parse().ok()?;
    ((640..=7680).contains(&w) && (480..=4320).contains(&h)).then_some((w, h))
}

/// `resolution` value meaning the game's own 640x480.
pub const ORIGINAL: (u32, u32) = (640, 480);

/// Splits `key = value`, ignoring comments and blank values.
fn entry(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('#') {
        return None;
    }
    let (key, value) = line.split_once('=')?;
    let value = value.trim();
    (!value.is_empty()).then_some((key.trim(), value))
}

/// `off`/`original` (0) or a number up to `max`.
fn parse_count(value: &str, max: u32) -> Option<u32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("original") {
        return Some(0);
    }
    value.parse().ok().filter(|n| *n <= max)
}

impl Settings {
    /// Reads a settings file; the second value lists lines it could not use.
    pub fn parse_file(text: &str) -> (Settings, Vec<String>) {
        let mut s = Settings::default();
        let mut warnings = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let Some((key, value)) = entry(line) else {
                continue;
            };
            let mut bad = |what: &str| {
                warnings.push(format!("{FILE_NAME} line {}: {what} {value:?}", n + 1));
            };
            let mut boolean = |slot: &mut Option<bool>, name: &str| match parse_bool(value) {
                Some(b) => *slot = Some(b),
                None => bad(&format!("{name} needs true or false, not")),
            };
            match key {
                "game" => s.game = Some(value.into()),
                "runtime" => s.runtime = Some(value.into()),
                "saves" => s.saves = Some(value.into()),
                "resolution" => match parse_resolution(value) {
                    Some(r) => s.resolution = Some(r),
                    None => {
                        bad("resolution needs WIDTHxHEIGHT (640x480 to 7680x4320) or original, not")
                    }
                },
                "fullscreen" => boolean(&mut s.fullscreen, "fullscreen"),
                "antialiasing" => match value.parse().ok().filter(|n| *n <= 4) {
                    Some(n) => s.antialiasing = Some(n),
                    None => bad("antialiasing needs 0 or 2, not"),
                },
                "fps_limit" => match value.trim().to_ascii_lowercase().as_str() {
                    "unlimited" | "none" | "off" => s.fps_limit = Some(0),
                    _ => match value.parse().ok().filter(|n| (10..=240).contains(n)) {
                        Some(f) => s.fps_limit = Some(f),
                        None => bad("fps_limit needs 10 to 240 or unlimited, not"),
                    },
                },
                "vsync" => match value.trim().to_ascii_lowercase().as_str() {
                    "on" | "true" => s.vsync = Some(Vsync::On),
                    "off" | "false" => s.vsync = Some(Vsync::Off),
                    "half" => s.vsync = Some(Vsync::Half),
                    _ => bad("vsync needs on, off or half, not"),
                },
                "draw_distance" => match value.parse().ok().filter(|n| *n <= 8) {
                    Some(n) => s.draw_distance = Some(n),
                    None => bad("draw_distance needs 0 to 8, not"),
                },
                "reflections" => match parse_count(value, 2048) {
                    Some(r) => s.reflections = Some(r),
                    None => bad("reflections needs original, 1024 or 2048, not"),
                },
                "anisotropy" => match parse_count(value, 16) {
                    Some(a) => s.anisotropy = Some(a),
                    None => bad("anisotropy needs 0 to 16, not"),
                },
                "sky_glow" => boolean(&mut s.sky_glow, "sky_glow"),
                "xbox_brightness" => boolean(&mut s.xbox_brightness, "xbox_brightness"),
                "hd_characters" => boolean(&mut s.hd_characters, "hd_characters"),
                "hd_textures" => s.hd_textures = Some(HdTextures::parse(value)),
                "deadzone" => match value.parse().ok().filter(|n| *n <= 50) {
                    Some(d) => s.deadzone = Some(d),
                    None => bad("deadzone needs 0 to 50, not"),
                },
                "vibration" => match Feedback::parse(value) {
                    Some(m) => s.vibration = Some(m),
                    None => bad("vibration needs off, on, swapped or merged, not"),
                },
                "vibration_strength" => match value.parse().ok().filter(|n| *n <= 10) {
                    Some(v) => s.vibration_strength = Some(v),
                    None => bad("vibration_strength needs 0 to 10, not"),
                },
                "impulse_triggers" => match Feedback::parse(value) {
                    Some(m) => s.impulse_triggers = Some(m),
                    None => bad("impulse_triggers needs off, on, swapped or merged, not"),
                },
                "input_backend" => match Backend::parse(value) {
                    Some(b) => s.input_backend = Some(b),
                    None => bad("input_backend needs wgi, raw, dinput or xinput, not"),
                },
                "hot_plug" => boolean(&mut s.hot_plug, "hot_plug"),
                "cd_switcher" => boolean(&mut s.cd_switcher, "cd_switcher"),
                "music_shuffle" => boolean(&mut s.music_shuffle, "music_shuffle"),
                "skip_logos" => boolean(&mut s.skip_logos, "skip_logos"),
                "free_drive" => boolean(&mut s.free_drive, "free_drive"),
                "horn" => boolean(&mut s.horn, "horn"),
                "random_anims" => boolean(&mut s.random_anims, "random_anims"),
                "character_select" => boolean(&mut s.character_select, "character_select"),
                "show_miles" => boolean(&mut s.show_miles, "show_miles"),
                "soundtrack_fix" => boolean(&mut s.soundtrack_fix, "soundtrack_fix"),
                "scaler" => match Scaler::parse(value) {
                    Some(f) => s.scaler = Some(f),
                    None => bad("scaler needs nearest, pixel, fsr or linear, not"),
                },
                _ => {}
            }
        }
        (s, warnings)
    }

    /// Command-line values override the file's.
    pub fn merged(self, over: Settings) -> Settings {
        macro_rules! merge {
            ($($field:ident),* $(,)?) => {
                Settings { $($field: over.$field.or(self.$field),)* }
            }
        }
        merge!(
            game,
            runtime,
            saves,
            resolution,
            fullscreen,
            antialiasing,
            fps_limit,
            vsync,
            draw_distance,
            reflections,
            anisotropy,
            sky_glow,
            xbox_brightness,
            hd_characters,
            hd_textures,
            deadzone,
            vibration,
            vibration_strength,
            impulse_triggers,
            input_backend,
            hot_plug,
            cd_switcher,
            music_shuffle,
            skip_logos,
            free_drive,
            horn,
            random_anims,
            character_select,
            show_miles,
            soundtrack_fix,
            scaler,
        )
    }

    // Effective values: the setting, or the launcher's default.

    pub fn on(&self, setting: &Option<bool>) -> bool {
        setting.unwrap_or(true)
    }

    /// The video resolution to apply, if any beyond the game's own.
    pub fn video_resolution(&self) -> Option<(u32, u32)> {
        self.resolution.filter(|&r| r != ORIGINAL)
    }

    pub fn fps_cap(&self) -> u32 {
        self.fps_limit.unwrap_or(60)
    }

    pub fn deadzone_percent(&self) -> u32 {
        self.deadzone.unwrap_or(20).min(50)
    }

    pub fn feedback(&self) -> Feedback {
        self.vibration.unwrap_or(Feedback::On)
    }

    pub fn impulse(&self) -> Feedback {
        self.impulse_triggers.unwrap_or(Feedback::Swapped)
    }

    pub fn backend(&self) -> Backend {
        self.input_backend.unwrap_or(Backend::WindowsGaming)
    }

    /// The game's `outrun2006.ini`, written whole every run: the game's
    /// own tool only ever writes these keys, and the mod deletes stale
    /// copies older than three months.
    pub fn outrun2006_ini(&self) -> String {
        let (w, h) = self.video_resolution().unwrap_or(ORIGINAL);
        let aa = self.antialiasing.unwrap_or(0);
        let windowed = u32::from(!self.fullscreen.unwrap_or(false));
        format!(
            "DX/ADAPTER = 0\n\
             DX/RES/WIDTH = {w}\n\
             DX/RES/HEIGHT = {h}\n\
             DX/ANTIALIASING = {aa}\n\
             DX/FOG = 0\n\
             LANGUAGE = 0\n\
             DX/WINDOWED = {windowed}\n"
        )
    }

    /// Writes this launcher's keys into an `OutRun2006Tweaks.ini`, keeping
    /// every other line (comments, keys we do not manage) as it was.
    pub fn apply_tweaks(&self, ini: &str) -> String {
        let put = |t: &str, section: &str, key: &str, value: &str| {
            ini::set_section_key(t, section, key, value)
        };
        let mut t = ini.to_string();
        let cap = self.fps_cap();
        let interp = cap == 0 || cap > 60;
        t = put(&t, "Performance", "FramerateLimit", &cap.to_string());
        t = put(
            &t,
            "Performance",
            "FramerateInterpolation",
            if interp { "true" } else { "false" },
        );
        t = put(
            &t,
            "Performance",
            "VSync",
            &match self.vsync.unwrap_or(Vsync::On) {
                Vsync::On => 1,
                Vsync::Off => 0,
                Vsync::Half => 2,
            }
            .to_string(),
        );

        t = put(&t, "Controls", "UseNewInput", "true");
        t = put(
            &t,
            "Controls",
            "InputBackend",
            &self.backend().ini().to_string(),
        );
        t = put(
            &t,
            "Controls",
            "SteeringDeadZone",
            &format!("{:.2}", f64::from(self.deadzone_percent()) / 100.0),
        );
        t = put(
            &t,
            "Controls",
            "ControllerHotPlug",
            if self.on(&self.hot_plug) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Controls",
            "VibrationMode",
            &self.feedback().ini().to_string(),
        );
        t = put(
            &t,
            "Controls",
            "VibrationStrength",
            &self.vibration_strength.unwrap_or(7).to_string(),
        );
        t = put(
            &t,
            "Controls",
            "ImpulseVibrationMode",
            &self.impulse().ini().to_string(),
        );

        t = put(
            &t,
            "Graphics",
            "AnisotropicFiltering",
            &self.anisotropy.unwrap_or(16).to_string(),
        );
        t = put(
            &t,
            "Graphics",
            "ReflectionResolution",
            &match self.reflections.unwrap_or(2048) {
                0 => 128,
                n => n,
            }
            .to_string(),
        );
        t = put(
            &t,
            "Graphics",
            "DrawDistanceIncrease",
            &self.draw_distance.unwrap_or(4).to_string(),
        );
        t = put(
            &t,
            "Graphics",
            "UseHiDefCharacters",
            if self.on(&self.hd_characters) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Graphics",
            "RestoreXboxBrightness",
            if self.xbox_brightness.unwrap_or(false) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Graphics",
            "SkyGlowFactor",
            if self.on(&self.sky_glow) { "4" } else { "0" },
        );

        t = put(
            &t,
            "Window",
            "WindowedBorderless",
            if self.fullscreen.unwrap_or(false) {
                "false"
            } else {
                "true"
            },
        );
        t = put(&t, "Window", "WindowPositionX", "0");
        t = put(&t, "Window", "WindowPositionY", "0");
        t = put(&t, "Window", "WindowedHideMouseCursor", "true");
        t = put(&t, "Window", "DisableDPIScaling", "true");
        t = put(&t, "Window", "AutoDetectResolution", "false");

        t = put(
            &t,
            "CDSwitcher",
            "SwitcherEnable",
            if self.on(&self.cd_switcher) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "CDSwitcher",
            "SwitcherShuffleTracks",
            if self.music_shuffle.unwrap_or(false) {
                "true"
            } else {
                "false"
            },
        );

        t = put(
            &t,
            "Misc",
            "SkipIntroLogos",
            if self.on(&self.skip_logos) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Misc",
            "DisableCountdownTimer",
            if self.on(&self.free_drive) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Misc",
            "AllowHorn",
            if self.on(&self.horn) { "true" } else { "false" },
        );
        t = put(
            &t,
            "Misc",
            "RandomHighwayAnimSets",
            if self.on(&self.random_anims) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Misc",
            "AllowCharacterSelection",
            if self.on(&self.character_select) {
                "true"
            } else {
                "false"
            },
        );
        t = put(
            &t,
            "Misc",
            "ShowOutRunMilesOnMenu",
            if self.on(&self.show_miles) {
                "true"
            } else {
                "false"
            },
        );

        // The track list ships stingers and jingles that stop the music
        // mid-cycle; keep the 28 racing tracks only.
        for name in NON_RACING_TRACKS {
            t = ini::remove_section_key(&t, "CDTracks", name);
        }
        t
    }

    pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Settings, String> {
        let mut s = Settings::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let mut value = |name: &str| {
                args.next()
                    .ok_or_else(|| format!("{name} needs a value\n\n{USAGE}"))
            };
            match arg.as_str() {
                "--game" => s.game = Some(value("--game")?.into()),
                "--runtime" => s.runtime = Some(value("--runtime")?.into()),
                "--saves" => s.saves = Some(value("--saves")?.into()),
                "--resolution" => {
                    let v = value("--resolution")?;
                    s.resolution = Some(parse_resolution(&v).ok_or_else(|| {
                        format!("--resolution needs WIDTHxHEIGHT or original, not {v}\n\n{USAGE}")
                    })?);
                }
                "--antialiasing" => {
                    let v = value("--antialiasing")?;
                    s.antialiasing = Some(v.parse().ok().filter(|n| *n <= 4).ok_or_else(|| {
                        format!("--antialiasing needs 0 or 2, not {v}\n\n{USAGE}")
                    })?);
                }
                "--fps-limit" => {
                    let v = value("--fps-limit")?;
                    let lower = v.trim().to_ascii_lowercase();
                    s.fps_limit =
                        Some(if lower == "unlimited" || lower == "off" {
                            0
                        } else {
                            v.parse().ok().filter(|n| (10..=240).contains(n)).ok_or_else(|| {
                            format!("--fps-limit needs 10 to 240 or unlimited, not {v}\n\n{USAGE}")
                        })?
                        });
                }
                "--draw-distance" => {
                    let v = value("--draw-distance")?;
                    s.draw_distance =
                        Some(v.parse().ok().filter(|n| *n <= 8).ok_or_else(|| {
                            format!("--draw-distance needs 0 to 8, not {v}\n\n{USAGE}")
                        })?);
                }
                "--reflections" => {
                    let v = value("--reflections")?;
                    s.reflections = Some(parse_count(&v, 2048).ok_or_else(|| {
                        format!("--reflections needs original, 1024 or 2048, not {v}\n\n{USAGE}")
                    })?);
                }
                "--anisotropy" => {
                    let v = value("--anisotropy")?;
                    s.anisotropy = Some(parse_count(&v, 16).ok_or_else(|| {
                        format!("--anisotropy needs 0 to 16, not {v}\n\n{USAGE}")
                    })?);
                }
                "--deadzone" => {
                    let v = value("--deadzone")?;
                    s.deadzone =
                        Some(v.parse().ok().filter(|n| *n <= 50).ok_or_else(|| {
                            format!("--deadzone needs 0 to 50, not {v}\n\n{USAGE}")
                        })?);
                }
                "--vibration-strength" => {
                    let v = value("--vibration-strength")?;
                    s.vibration_strength =
                        Some(v.parse().ok().filter(|n| *n <= 10).ok_or_else(|| {
                            format!("--vibration-strength needs 0 to 10, not {v}\n\n{USAGE}")
                        })?);
                }
                "--vibration" | "--impulse-triggers" => {
                    let v = value(&arg)?;
                    let mode = Feedback::parse(&v).ok_or_else(|| {
                        format!(
                            "{} needs off, on, swapped or merged, not {v}\n\n{USAGE}",
                            arg
                        )
                    })?;
                    if arg == "--vibration" {
                        s.vibration = Some(mode);
                    } else {
                        s.impulse_triggers = Some(mode);
                    }
                }
                "--input-backend" => {
                    let v = value("--input-backend")?;
                    s.input_backend = Some(Backend::parse(&v).ok_or_else(|| {
                        format!(
                            "--input-backend needs wgi, raw, dinput or xinput, not {v}\n\n{USAGE}"
                        )
                    })?);
                }
                "--hd-textures" => {
                    s.hd_textures = Some(HdTextures::parse(&value("--hd-textures")?))
                }
                "--vsync" => {
                    let v = value("--vsync")?;
                    s.vsync = Some(match v.trim().to_ascii_lowercase().as_str() {
                        "on" | "true" => Vsync::On,
                        "off" | "false" => Vsync::Off,
                        "half" => Vsync::Half,
                        _ => {
                            return Err(format!(
                                "--vsync needs on, off or half, not {v}\n\n{USAGE}"
                            ));
                        }
                    });
                }
                flag @ ("--fullscreen" | "--sky-glow" | "--xbox-brightness" | "--hd-characters"
                | "--hot-plug" | "--cd-switcher" | "--music-shuffle" | "--skip-logos"
                | "--free-drive" | "--horn" | "--random-anims" | "--character-select"
                | "--show-miles" | "--soundtrack-fix") => {
                    let v = value(flag)?;
                    let b = parse_bool(&v)
                        .ok_or_else(|| format!("{flag} needs on or off, not {v}\n\n{USAGE}"))?;
                    match flag {
                        "--fullscreen" => s.fullscreen = Some(b),
                        "--sky-glow" => s.sky_glow = Some(b),
                        "--xbox-brightness" => s.xbox_brightness = Some(b),
                        "--hd-characters" => s.hd_characters = Some(b),
                        "--hot-plug" => s.hot_plug = Some(b),
                        "--cd-switcher" => s.cd_switcher = Some(b),
                        "--music-shuffle" => s.music_shuffle = Some(b),
                        "--skip-logos" => s.skip_logos = Some(b),
                        "--free-drive" => s.free_drive = Some(b),
                        "--horn" => s.horn = Some(b),
                        "--random-anims" => s.random_anims = Some(b),
                        "--character-select" => s.character_select = Some(b),
                        "--show-miles" => s.show_miles = Some(b),
                        _ => s.soundtrack_fix = Some(b),
                    }
                }
                "--help" | "-h" | "/?" => return Err(USAGE.into()),
                other => return Err(format!("unknown argument {other}\n\n{USAGE}")),
            }
        }
        Ok(s)
    }
}

/// Entries of the shipped `[CDTracks]` list that are not racing music
/// (jingles, stingers and ambience), removed so the mid-race CD switcher
/// only lands on songs.
pub const NON_RACING_TRACKS: [&str; 11] = [
    "Last_Wave.ogg",
    "SEGA441.ogg",
    "Beach_wave.ogg",
    "title_pattern1.ogg",
    "title_01.ogg",
    "or2ed1.ogg",
    "or2ed2.ogg",
    "or2ed3a.ogg",
    "or2ed3b.ogg",
    "or2ed4.ogg",
    "or2ed5.ogg",
];

/// Sets `key` in a settings file, keeping every other line (and comments)
/// as it was: replaces the first active or empty `key =` line, else appends.
pub fn set_key(text: &str, key: &str, value: &str) -> String {
    let is_key = |line: &str| {
        let line = line.trim();
        !line.starts_with('#') && line.split_once('=').is_some_and(|(k, _)| k.trim() == key)
    };
    let mut out = String::with_capacity(text.len() + key.len() + value.len() + 4);
    let mut done = false;
    for line in text.lines() {
        if !done && is_key(line) {
            out.push_str(&format!("{key} = {value}"));
            done = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !done {
        out.push_str(&format!("{key} = {value}\n"));
    }
    out
}

/// Turns `key` off in a settings file: its active line becomes a comment,
/// so the default applies; other lines are kept.
pub fn clear_key(text: &str, key: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with('#') && t.split_once('=').is_some_and(|(k, _)| k.trim() == key) {
            out.push_str(&format!("#{key} ="));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Settings, String> {
        Settings::parse_args(list.iter().map(|s| s.to_string()))
    }

    #[test]
    fn template_parses_to_the_defaults() {
        let (s, warnings) = Settings::parse_file(TEMPLATE);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s, Settings::default());
        assert_eq!(s.video_resolution(), None);
        assert_eq!(s.fps_cap(), 60);
        assert_eq!(s.deadzone_percent(), 20);
    }

    #[test]
    fn outrun2006_ini_matches_the_games_format() {
        let s = Settings::default();
        assert_eq!(
            s.outrun2006_ini(),
            "DX/ADAPTER = 0\nDX/RES/WIDTH = 640\nDX/RES/HEIGHT = 480\nDX/ANTIALIASING = 0\nDX/FOG = 0\nLANGUAGE = 0\nDX/WINDOWED = 1\n"
        );
        let s =
            Settings::parse_file("resolution = 3840x2160\nfullscreen = true\nantialiasing = 2\n").0;
        assert_eq!(
            s.outrun2006_ini(),
            "DX/ADAPTER = 0\nDX/RES/WIDTH = 3840\nDX/RES/HEIGHT = 2160\nDX/ANTIALIASING = 2\nDX/FOG = 0\nLANGUAGE = 0\nDX/WINDOWED = 0\n"
        );
    }

    #[test]
    fn tweaks_ini_gets_our_keys_and_keeps_the_rest() {
        let base = "[Performance]\nFramerateLimit = 0\n# comment stays\n\n[Controls]\nSteeringDeadZone = 0.2\n\n[Window]\nWindowedBorderless=true\n\n[CDTracks]\n01_Splash_wave.ogg = Splash Wave\nLast_Wave.ogg = Last Wave\nSEGA441.ogg = SEGA Intro\n\n[Misc]\n";
        let out = Settings::default().apply_tweaks(base);
        assert!(out.contains("FramerateLimit = 60"));
        assert!(out.contains("# comment stays"));
        assert!(out.contains("FramerateInterpolation = false"));
        assert!(out.contains("SteeringDeadZone = 0.20"));
        assert!(out.contains("UseNewInput = true"));
        assert!(out.contains("InputBackend = 0"));
        assert!(out.contains("VibrationMode = 1"));
        assert!(out.contains("ImpulseVibrationMode = 2"));
        assert!(out.contains("ReflectionResolution = 2048"));
        assert!(out.contains("DrawDistanceIncrease = 4"));
        assert!(out.contains("SkyGlowFactor = 4"));
        assert!(out.contains("WindowedBorderless = true"));
        assert!(out.contains("WindowPositionX = 0"));
        assert!(out.contains("AutoDetectResolution = false"));
        assert!(out.contains("DisableCountdownTimer = true"));
        assert!(out.contains("01_Splash_wave.ogg = Splash Wave"));
        assert!(!out.contains("Last_Wave.ogg"));
        assert!(!out.contains("SEGA441.ogg"));

        // Keys the base file lacked are added inside their sections.
        assert!(out.find("[Controls]\n").is_some_and(|at| {
            out[at..out.find("[Window]").unwrap()].contains("VibrationStrength = 7\n")
        }));

        // The unlimited cap turns interpolation on.
        let fast = Settings::parse_file("fps_limit = unlimited\n").0;
        assert!(
            fast.apply_tweaks(&out)
                .contains("FramerateInterpolation = true")
        );

        // Everything off: the faithful picture.
        let off = "sky_glow = false\nhd_characters = false\ncd_switcher = false\nskip_logos = false\nfree_drive = false\nhorn = false\nrandom_anims = false\ncharacter_select = false\nshow_miles = false\nsoundtrack_fix = false\nvibration = off\ndraw_distance = 0\nreflections = original\n";
        let stock = Settings::parse_file(off).0.apply_tweaks(base);
        assert!(stock.contains("SkyGlowFactor = 0"));
        assert!(stock.contains("UseHiDefCharacters = false"));
        assert!(stock.contains("SwitcherEnable = false"));
        assert!(stock.contains("SkipIntroLogos = false"));
        assert!(stock.contains("DisableCountdownTimer = false"));
        assert!(stock.contains("AllowHorn = false"));
        assert!(stock.contains("RandomHighwayAnimSets = false"));
        assert!(stock.contains("AllowCharacterSelection = false"));
        assert!(stock.contains("ShowOutRunMilesOnMenu = false"));
        assert!(stock.contains("VibrationMode = 0"));
        assert!(stock.contains("DrawDistanceIncrease = 0"));
        assert!(stock.contains("ReflectionResolution = 128"));
    }

    #[test]
    fn parses_arguments() {
        let s = args(&[
            "--game",
            r"D:\Games\OutRun 2006",
            "--resolution",
            "2560x1440",
            "--fps-limit",
            "unlimited",
            "--vibration",
            "merged",
            "--input-backend",
            "xinput",
        ])
        .unwrap();
        assert_eq!(s.game, Some(PathBuf::from(r"D:\Games\OutRun 2006")));
        assert_eq!(s.video_resolution(), Some((2560, 1440)));
        assert_eq!(s.fps_cap(), 0);
        assert_eq!(s.feedback(), Feedback::Merged);
        assert_eq!(s.backend(), Backend::XInput);
        assert!(args(&["--game"]).is_err());
        assert!(args(&["--bogus"]).is_err());
        assert!(args(&["--resolution", "8000x100"]).is_err());
        assert!(args(&["--vsync", "maybe"]).is_err());
        assert!(args(&["--fps-limit", "5"]).is_err());
        assert!(args(&["--draw-distance", "9"]).is_err());
        assert!(args(&["--deadzone", "60"]).is_err());
        assert!(args(&["--vibration", "loud"]).is_err());
    }

    #[test]
    fn file_values_and_args_win() {
        let text = "game = C:\\old\nresolution = 1920X1440\nvibration = swapped\n";
        let (saved, warnings) = Settings::parse_file(text);
        assert!(warnings.is_empty());
        assert_eq!(saved.resolution, Some((1920, 1440)));
        assert_eq!(saved.feedback(), Feedback::Swapped);
        let m = saved.clone().merged(args(&["--game", r"C:\new"]).unwrap());
        assert_eq!(m.game, Some(PathBuf::from(r"C:\new")));
        assert_eq!(m.resolution, Some((1920, 1440)));

        // `original` on the command line overrides a file's resolution.
        let back = saved.merged(args(&["--resolution", "original"]).unwrap());
        assert_eq!(back.video_resolution(), None);
    }

    #[test]
    fn file_ignores_junk_and_reports_bad_values() {
        let (s, warnings) = Settings::parse_file(
            "# game = C:\\commented\nfuture=1\ngame = C:\\G \n\nruntime=\nresolution=8000x100\ndeadzone=99\n",
        );
        assert_eq!(s.game, Some(PathBuf::from(r"C:\G")));
        assert_eq!(s.runtime, None);
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[0].contains("line 6"));
    }

    #[test]
    fn set_key_keeps_the_rest_of_the_file() {
        let filled = set_key(TEMPLATE, "game", r"D:\OutRun 2006");
        assert_eq!(filled.lines().count(), TEMPLATE.lines().count());
        assert!(filled.contains("\ngame = D:\\OutRun 2006\n"));
        let (s, _) = Settings::parse_file(&filled);
        assert_eq!(s.game, Some(PathBuf::from(r"D:\OutRun 2006")));
        let again = set_key(&filled, "game", r"E:\OutRun 2006");
        assert_eq!(again.matches("game =").count(), 1);
        assert!(set_key("#game = x\n", "game", "y").ends_with("#game = x\ngame = y\n"));
    }

    #[test]
    fn hd_textures_parse_their_three_forms() {
        assert_eq!(HdTextures::parse("on"), HdTextures::Bundled);
        assert_eq!(HdTextures::parse("off"), HdTextures::Off);
        assert_eq!(
            HdTextures::parse(r"D:\pack"),
            HdTextures::Folder(PathBuf::from(r"D:\pack"))
        );
        assert_eq!(
            Settings::parse_file("hd_textures = on\n").0.hd_textures,
            Some(HdTextures::Bundled)
        );
    }

    #[test]
    fn every_setting_has_a_flag() {
        let s = args(&[
            "--fullscreen",
            "on",
            "--sky-glow",
            "off",
            "--xbox-brightness",
            "on",
            "--hd-characters",
            "off",
            "--hot-plug",
            "off",
            "--cd-switcher",
            "off",
            "--music-shuffle",
            "on",
            "--skip-logos",
            "off",
            "--free-drive",
            "off",
            "--horn",
            "off",
            "--random-anims",
            "off",
            "--character-select",
            "off",
            "--show-miles",
            "off",
            "--soundtrack-fix",
            "off",
            "--impulse-triggers",
            "off",
            "--antialiasing",
            "2",
            "--reflections",
            "1024",
            "--anisotropy",
            "8",
            "--deadzone",
            "10",
            "--vibration-strength",
            "5",
            "--hd-textures",
            "off",
        ])
        .unwrap();
        assert_eq!(s.fullscreen, Some(true));
        assert_eq!(s.sky_glow, Some(false));
        assert_eq!(s.xbox_brightness, Some(true));
        assert_eq!(s.hd_characters, Some(false));
        assert_eq!(s.hot_plug, Some(false));
        assert_eq!(s.cd_switcher, Some(false));
        assert_eq!(s.music_shuffle, Some(true));
        assert_eq!(s.skip_logos, Some(false));
        assert_eq!(s.free_drive, Some(false));
        assert_eq!(s.horn, Some(false));
        assert_eq!(s.random_anims, Some(false));
        assert_eq!(s.character_select, Some(false));
        assert_eq!(s.show_miles, Some(false));
        assert_eq!(s.soundtrack_fix, Some(false));
        assert_eq!(s.impulse_triggers, Some(Feedback::Off));
        assert_eq!(s.antialiasing, Some(2));
        assert_eq!(s.reflections, Some(1024));
        assert_eq!(s.anisotropy, Some(8));
        assert_eq!(s.deadzone, Some(10));
        assert_eq!(s.vibration_strength, Some(5));
        assert_eq!(s.hd_textures, Some(HdTextures::Off));
    }
}
