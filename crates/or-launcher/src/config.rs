//! Command line and the settings file (`launcher.cfg`).
//!
//! One way flow: the launcher's settings become the game's two INI files
//! (`outrun2006.ini` for the video mode, `OutRun2006Tweaks.ini` for the
//! OutRun2006Tweaks mod). The file is plain `key = value` lines with `#`
//! comments, written once from a commented template so users can edit it;
//! the launcher later changes only the keys it owns. The file and the
//! command line share one `apply` (the SDK's `cfg` toolkit); unknown keys
//! and values it cannot read are reported and ignored.

use std::path::{Path, PathBuf};

use rea_launcher::cfg::{self, Rejected};
use rea_launcher::ini::Ini;

pub use rea_launcher::cfg::{clear_key, read_file, set_key};

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
    /// `DX/ANTIALIASING`: 0 off, 2 the game's own MSAA (4x under DXVK).
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
    /// The community HD texture pack (a separate download).
    pub hd_textures: Option<bool>,
    /// Where the unzipped pack is; without it, `textures-pack` next to the
    /// launcher.
    pub hd_textures_folder: Option<PathBuf>,

    // Controls
    /// Steering deadzone in percent (game default 20).
    pub deadzone: Option<u32>,
    pub vibration: Option<Feedback>,
    /// 0 to 10.
    pub vibration_strength: Option<u32>,
    pub impulse_triggers: Option<Feedback>,
    pub input_backend: Option<Backend>,
    /// Start races with the manual gearbox selected.
    pub manual_gearbox: Option<bool>,

    // Extras
    /// Change music mid-race (Back or X = next, Z = previous).
    pub cd_switcher: Option<bool>,
    pub music_shuffle: Option<bool>,
    pub skip_logos: Option<bool>,
    /// No countdown: coast for as long as you like.
    pub free_drive: Option<bool>,
    /// Horn outside the girlfriend missions.
    pub horn: Option<bool>,
    /// Roadside animations from any game mode.
    pub random_anims: Option<bool>,
    /// OutRun miles under the menu's sign-in text.
    pub show_miles: Option<bool>,
    /// Restore the first two seconds of "Rush a Difficulty".
    pub soundtrack_fix: Option<bool>,
    /// Linux only: how `or-play`'s gamescope enlarges the picture.
    pub scaler: Option<Scaler>,
}

/// `vsync`: wait for the screen's refresh or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vsync {
    On,
    Off,
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
    pub fn default_on(wine: bool) -> Backend {
        if wine {
            Backend::XInput
        } else {
            Backend::WindowsGaming
        }
    }

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

    /// `Some(None)` is `auto`: no choice, the platform default.
    fn parse(value: &str) -> Option<Option<Backend>> {
        let value = value.trim().to_ascii_lowercase();
        if value == "auto" {
            return Some(None);
        }
        Backend::ALL
            .into_iter()
            .find(|b| b.key() == value)
            .map(Some)
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

/// The community HD texture pack's download page (a separate download:
/// 326 MB zipped, 3.2 GB unpacked).
pub const HD_PACK_PAGE: &str = "https://github.com/envido32/OR2006Sprites/releases";

/// The pack's folder next to the launcher, used when no folder is chosen.
pub const TEXTURES_PACK: &str = "textures-pack";

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

# Video resolution, e.g. 1920x1080 or 3840x2160. Unset, it is the game's own
# 640x480 (the settings window picks your screen's size on first run). The
# game runs as a borderless window at this size (see fullscreen).
#resolution = 1920x1080

# true switches to exclusive fullscreen (the borderless window is easier on
# multi-monitor desktops and works the same under Wine). In exclusive
# fullscreen the frame-rate cap is held at 60.
#fullscreen = false

# Anti-aliasing: 0 off, 2 the game's own multisampling (4x under DXVK). On
# NVIDIA under Windows, trackside cloth and barriers are supersampled too.
#antialiasing = 0

# Frame-rate cap in frames per second. The game's driving runs at a fixed
# 60 Hz; above 60 the mod interpolates the picture. unlimited (or 0) turns the
# cap off entirely and leaves pacing to vsync or an external limiter.
#fps_limit = 60

# vsync: on waits for the screen's refresh, off doesn't.
#vsync = on

# How much further down the stage the world draws: 0 (the game's own) to 8;
# the window offers 0, 4 and 8. Further out, scenery can pop in.
#draw_distance = 4

# Resolution of the car's reflections: original (128), 512, 1024 or 2048.
#reflections = 2048

# Anisotropic filtering, 1 to 16; 0 keeps the game's own.
#anisotropy = 16

# The console versions' glowing sky effect.
#sky_glow = true

# The Xbox releases' brighter picture.
#xbox_brightness = false

# The hi-def passenger models the PC release shipped unused.
#hd_characters = true

# The community HD texture pack (sharper menus, HUD and sprites). It is a
# separate download: get it from
# https://github.com/envido32/OR2006Sprites/releases, unzip it anywhere and
# give its folder below (the one with textures\load inside). Without a
# folder, a textures-pack folder next to OutRunLauncher.exe is used.
#hd_textures = off
#hd_textures_folder =

# Steering deadzone in percent, 0 to 50 (the game's default is 20).
#deadzone = 20

# Gamepad rumble: off, on, swapped (left/right motors), or merged.
#vibration = on

# Rumble strength, 0 to 10.
#vibration_strength = 7

# Xbox Series impulse triggers: off, on, swapped (recommended), or merged.
#impulse_triggers = swapped

# Input API for the mod's controller system: wgi (Windows.Gaming.Input),
# raw, dinput or xinput. Without a choice: wgi on Windows, xinput under
# Wine and Proton (whose wgi loses the d-pad). Try another if a pad isn't
# seen.
#input_backend = wgi

# Linux (or-play) only: how gamescope enlarges the picture: linear (smooth,
# the default), nearest (sharp), pixel (even pixels) or fsr.
#scaler = linear

# Start races with the manual gearbox selected.
#manual_gearbox = false

# Change the music during a race (Back or X = next track, Z = previous).
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
  --vsync on|off            wait for the screen's refresh
  --draw-distance <0-8>     how much further the stage draws
  --reflections <original|512|1024|2048>  car reflection resolution
  --anisotropy <0-16>       anisotropic filtering
  --sky-glow on|off         the console versions' glowing sky
  --xbox-brightness on|off  the Xbox releases' brighter picture
  --hd-characters on|off    hi-def passenger models
  --hd-textures on|off      the community HD texture pack
  --hd-textures-folder <folder>  where the unzipped pack is
  --deadzone <0-50>         steering deadzone in percent
  --vibration <mode>        rumble: off, on, swapped or merged
  --vibration-strength <0-10>
  --impulse-triggers <mode> Xbox One/Series impulse triggers (Windows): off, on, swapped or merged
  --input-backend <name>    wgi, raw, dinput or xinput
  --manual-gearbox on|off   start races with the manual gearbox
  --cd-switcher on|off      change music mid-race
  --music-shuffle on|off    shuffle the race music
  --skip-logos on|off       skip the boot logos
  --free-drive on|off       no countdown timer
  --horn on|off             usable horn
  --random-anims on|off     roadside animations from any mode
  --show-miles on|off       OutRun miles in the menu
  --soundtrack-fix on|off   restore Rush a Difficulty's first two seconds

These override launcher.cfg for one run (only --game is remembered). Without
options the settings window opens first.

  --no-window               play with the saved settings, no window
  --settings [--linux]      only the settings window (exit codes 0: play,
                           3: closed, 4: no window could be shown;
                           --linux adds the gamescope scaler, for or-play)

OutRunLauncher --check [--game <folder>]
  checks the game folder and tells you whether its OR2006C2C.exe is supported

OutRunLauncher --download-hd-pack
  downloads the community HD texture pack (progress in launcher.log) and
  switches it on in launcher.cfg

Other settings are in launcher.cfg (see the comments in it)."
);

/// `resolution` words: `original` and `off` are the game's own 640x480.
fn parse_resolution(value: &str) -> Option<(u32, u32)> {
    match cfg::parse_resolution(value, (640, 480), (7680, 4320), cfg::ORIGINAL_OR_OFF)? {
        cfg::Resolution::Size(w, h) => Some((w, h)),
        _ => Some(ORIGINAL),
    }
}

/// 0 (unlimited) or 10 to 240 frames per second.
fn valid_fps(n: &u32) -> bool {
    *n == 0 || (10..=240).contains(n)
}

/// `resolution` value meaning the game's own 640x480.
pub const ORIGINAL: (u32, u32) = (640, 480);

/// `original` (0 or 128, the game's own) or a size Tweaks can render.
fn parse_reflections(value: &str) -> Option<u32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("original") || value.eq_ignore_ascii_case("off") {
        return Some(0);
    }
    match value.parse().ok()? {
        0 | 128 => Some(0),
        n @ (512 | 1024 | 2048) => Some(n),
        _ => None,
    }
}

/// `off`/`original` (0) or a number up to `max`.
fn parse_count(value: &str, max: u32) -> Option<u32> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("original") {
        return Some(0);
    }
    cfg::parse_range(value, 0..=max)
}

/// Every key launcher.cfg and the command line take (`--fps-limit` is
/// `fps_limit`). `scaler` is only set by the window and the file.
const FLAG_KEYS: [&str; 30] = [
    "game",
    "runtime",
    "saves",
    "resolution",
    "fullscreen",
    "antialiasing",
    "fps_limit",
    "vsync",
    "draw_distance",
    "reflections",
    "anisotropy",
    "sky_glow",
    "xbox_brightness",
    "hd_characters",
    "hd_textures",
    "hd_textures_folder",
    "deadzone",
    "vibration",
    "vibration_strength",
    "impulse_triggers",
    "input_backend",
    "manual_gearbox",
    "cd_switcher",
    "music_shuffle",
    "skip_logos",
    "free_drive",
    "horn",
    "random_anims",
    "show_miles",
    "soundtrack_fix",
];

/// Where a value comes from: the file is more forgiving (retired values)
/// and words its messages for the file.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    File,
    Args,
}

impl Settings {
    /// Reads a settings file; the second value lists lines it could not use.
    pub fn parse_file(text: &str) -> (Settings, Vec<String>) {
        let mut s = Settings::default();
        let mut warnings = cfg::parse(text, FILE_NAME, |k, v| s.apply(k, v, Source::File));
        for e in cfg::entries(text) {
            if e.key == "vsync" && e.value.eq_ignore_ascii_case("half") {
                warnings.push(format!(
                    "{FILE_NAME} line {}: vsync half is no longer offered; using on",
                    e.line
                ));
            }
        }
        (s, warnings)
    }

    /// One setting, from the file or a `--flag value` pair.
    fn apply(&mut self, key: &str, value: &str, from: Source) -> Result<(), Rejected> {
        let boolean = |slot: &mut Option<bool>| -> Result<(), Rejected> {
            let needs = match from {
                Source::File => "true or false",
                Source::Args => cfg::ON_OFF,
            };
            *slot = Some(cfg::need(cfg::parse_bool(value), needs)?);
            Ok(())
        };
        match key {
            "game" => self.game = Some(value.into()),
            "runtime" => self.runtime = Some(value.into()),
            "saves" => self.saves = Some(value.into()),
            "resolution" => {
                let needs = match from {
                    Source::File => "WIDTHxHEIGHT (640x480 to 7680x4320) or original",
                    Source::Args => "WIDTHxHEIGHT or original",
                };
                self.resolution = Some(cfg::need(parse_resolution(value), needs)?);
            }
            "fullscreen" => return boolean(&mut self.fullscreen),
            "antialiasing" => {
                let aa = cfg::parse_range(value, 0..=2).filter(|n| *n != 1);
                self.antialiasing = Some(cfg::need(aa, "0 or 2")?);
            }
            "fps_limit" => {
                let fps = match value.trim().to_ascii_lowercase().as_str() {
                    "unlimited" | "none" | "off" => Some(0),
                    _ => value.trim().parse().ok().filter(valid_fps),
                };
                self.fps_limit = Some(cfg::need(fps, "10 to 240 or unlimited")?);
            }
            "vsync" => {
                let vsync = match value.trim().to_ascii_lowercase().as_str() {
                    "on" | "true" => Some(Vsync::On),
                    "off" | "false" => Some(Vsync::Off),
                    // Retired: read as on (parse_file says so).
                    "half" if from == Source::File => Some(Vsync::On),
                    _ => None,
                };
                self.vsync = Some(cfg::need(vsync, "on or off")?);
            }
            "draw_distance" => {
                self.draw_distance = Some(cfg::need(cfg::parse_range(value, 0..=8), "0 to 8")?)
            }
            "reflections" => {
                self.reflections = Some(cfg::need(
                    parse_reflections(value),
                    "original (128), 512, 1024 or 2048",
                )?)
            }
            "anisotropy" => self.anisotropy = Some(cfg::need(parse_count(value, 16), "0 to 16")?),
            "sky_glow" => return boolean(&mut self.sky_glow),
            "xbox_brightness" => return boolean(&mut self.xbox_brightness),
            "hd_characters" => return boolean(&mut self.hd_characters),
            "hd_textures" => self.set_hd_textures(value),
            "hd_textures_folder" => {
                self.hd_textures_folder = Some(value.trim())
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            }
            "deadzone" => {
                self.deadzone = Some(cfg::need(cfg::parse_range(value, 0..=50), "0 to 50")?)
            }
            "vibration" => {
                self.vibration = Some(cfg::need(
                    Feedback::parse(value),
                    "off, on, swapped or merged",
                )?)
            }
            "vibration_strength" => {
                self.vibration_strength =
                    Some(cfg::need(cfg::parse_range(value, 0..=10), "0 to 10")?)
            }
            "impulse_triggers" => {
                self.impulse_triggers = Some(cfg::need(
                    Feedback::parse(value),
                    "off, on, swapped or merged",
                )?)
            }
            "input_backend" => {
                self.input_backend = cfg::need(Backend::parse(value), "wgi, raw, dinput or xinput")?
            }
            "manual_gearbox" => return boolean(&mut self.manual_gearbox),
            "cd_switcher" => return boolean(&mut self.cd_switcher),
            "music_shuffle" => return boolean(&mut self.music_shuffle),
            "skip_logos" => return boolean(&mut self.skip_logos),
            "free_drive" => return boolean(&mut self.free_drive),
            "horn" => return boolean(&mut self.horn),
            "random_anims" => return boolean(&mut self.random_anims),
            "show_miles" => return boolean(&mut self.show_miles),
            "soundtrack_fix" => return boolean(&mut self.soundtrack_fix),
            "scaler" if from == Source::File => {
                self.scaler = Some(cfg::need(
                    Scaler::parse(value),
                    "nearest, pixel, fsr or linear",
                )?)
            }
            // Options that were removed are ignored without a word.
            "character_select" | "hot_plug" if from == Source::File => {}
            _ => return Err(Rejected::Unknown),
        }
        Ok(())
    }

    /// `hd_textures`: on or off, or (older files) the pack's folder, which
    /// also turns it on.
    fn set_hd_textures(&mut self, value: &str) {
        match cfg::parse_bool(value) {
            Some(on) => self.hd_textures = Some(on),
            None => {
                self.hd_textures = Some(true);
                self.hd_textures_folder = Some(value.trim().into());
            }
        }
    }

    /// Where the HD textures come from when they are on: the chosen folder,
    /// or `textures-pack` next to the launcher.
    pub fn hd_textures_source(&self, launcher_dir: &Path) -> Option<PathBuf> {
        if !self.hd_textures.unwrap_or(false) {
            return None;
        }
        Some(
            self.hd_textures_folder
                .clone()
                .unwrap_or_else(|| launcher_dir.join(TEXTURES_PACK)),
        )
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
            hd_textures_folder,
            deadzone,
            vibration,
            vibration_strength,
            impulse_triggers,
            input_backend,
            manual_gearbox,
            cd_switcher,
            music_shuffle,
            skip_logos,
            free_drive,
            horn,
            random_anims,
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

    /// The input API when launcher.cfg names none: Windows.Gaming.Input
    /// on Windows (Tweaks' choice); XInput under Wine and Proton, whose
    /// Windows.Gaming.Input loses the d-pad.
    pub fn backend_on(&self, wine: bool) -> Backend {
        self.input_backend.unwrap_or(Backend::default_on(wine))
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
             DX/FOG = 1\n\
             LANGUAGE = 0\n\
             DX/WINDOWED = {windowed}\n"
        )
    }

    /// Writes this launcher's keys into an `OutRun2006Tweaks.ini`'s text,
    /// keeping every other line (comments, keys we do not manage) as it was.
    pub fn apply_tweaks(&self, text: &str) -> String {
        let mut ini = Ini::parse(text);
        self.tweak(&mut ini);
        ini.to_text().expect("parsed from text")
    }

    /// Sets this launcher's keys in an `OutRun2006Tweaks.ini`.
    pub fn tweak(&self, ini: &mut Ini) {
        let mut put = |section: &str, key: &str, value: &str| ini.set(section, key, value);
        // Exclusive fullscreen turns the cap into a refresh-rate request.
        let cap = match self.fps_cap() {
            0 => 0,
            _ if self.fullscreen.unwrap_or(false) => 60,
            n => n,
        };
        let interp = cap == 0 || cap > 60;
        put("Performance", "FramerateLimit", &cap.to_string());
        put(
            "Performance",
            "FramerateInterpolation",
            if interp { "true" } else { "false" },
        );
        put(
            "Performance",
            "VSync",
            &match self.vsync.unwrap_or(Vsync::On) {
                Vsync::On => 1,
                Vsync::Off => 0,
            }
            .to_string(),
        );

        put("Controls", "UseNewInput", "true");
        put(
            "Controls",
            "InputBackend",
            &self.backend().ini().to_string(),
        );
        put(
            "Controls",
            "SteeringDeadZone",
            &format!("{:.2}", f64::from(self.deadzone_percent()) / 100.0),
        );
        put("Controls", "ControllerHotPlug", "false");
        put(
            "Controls",
            "DefaultManualTransmission",
            if self.manual_gearbox.unwrap_or(false) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "Controls",
            "VibrationMode",
            &self.feedback().ini().to_string(),
        );
        put(
            "Controls",
            "VibrationStrength",
            &self.vibration_strength.unwrap_or(7).to_string(),
        );
        put(
            "Controls",
            "ImpulseVibrationMode",
            &self.impulse().ini().to_string(),
        );

        put(
            "Graphics",
            "AnisotropicFiltering",
            &self.anisotropy.unwrap_or(16).to_string(),
        );
        put(
            "Graphics",
            "ReflectionResolution",
            &match self.reflections.unwrap_or(2048) {
                0 => 128,
                n => n,
            }
            .to_string(),
        );
        put(
            "Graphics",
            "DrawDistanceIncrease",
            &self.draw_distance.unwrap_or(4).to_string(),
        );
        put(
            "Graphics",
            "UseHiDefCharacters",
            if self.on(&self.hd_characters) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "Graphics",
            "RestoreXboxBrightness",
            if self.xbox_brightness.unwrap_or(false) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "Graphics",
            "SkyGlowFactor",
            if self.on(&self.sky_glow) { "4" } else { "0" },
        );

        put(
            "Window",
            "WindowedBorderless",
            if self.fullscreen.unwrap_or(false) {
                "false"
            } else {
                "true"
            },
        );
        put("Window", "WindowPositionX", "0");
        put("Window", "WindowPositionY", "0");
        put("Window", "WindowedHideMouseCursor", "true");
        put("Window", "DisableDPIScaling", "true");
        put("Window", "AutoDetectResolution", "false");

        put(
            "CDSwitcher",
            "SwitcherEnable",
            if self.on(&self.cd_switcher) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "CDSwitcher",
            "SwitcherShuffleTracks",
            if self.music_shuffle.unwrap_or(false) {
                "true"
            } else {
                "false"
            },
        );

        put(
            "Misc",
            "SkipIntroLogos",
            if self.on(&self.skip_logos) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "Misc",
            "DisableCountdownTimer",
            if self.on(&self.free_drive) {
                "true"
            } else {
                "false"
            },
        );
        put(
            "Audio",
            "AllowHorn",
            if self.on(&self.horn) { "true" } else { "false" },
        );
        put(
            "Misc",
            "RandomHighwayAnimSets",
            if self.on(&self.random_anims) {
                "true"
            } else {
                "false"
            },
        );
        put("Misc", "AllowCharacterSelection", "false");
        put(
            "Misc",
            "ShowOutRunMilesOnMenu",
            if self.on(&self.show_miles) {
                "true"
            } else {
                "false"
            },
        );
    }

    /// Command-line values (`--fps-limit 120`); the launcher's own switches
    /// (`--no-window`, ...) are taken out before.
    pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Settings, String> {
        let mut s = Settings::default();
        match cfg::parse_args(args, &FLAG_KEYS, &[], |k, v| s.apply(k, v, Source::Args)) {
            Ok(_) => Ok(s),
            Err(cfg::ArgError::Help) => Err(USAGE.into()),
            Err(e) => Err(format!("{e}\n\n{USAGE}")),
        }
    }
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
            "DX/ADAPTER = 0\nDX/RES/WIDTH = 640\nDX/RES/HEIGHT = 480\nDX/ANTIALIASING = 0\nDX/FOG = 1\nLANGUAGE = 0\nDX/WINDOWED = 1\n"
        );
        let s =
            Settings::parse_file("resolution = 3840x2160\nfullscreen = true\nantialiasing = 2\n").0;
        assert_eq!(
            s.outrun2006_ini(),
            "DX/ADAPTER = 0\nDX/RES/WIDTH = 3840\nDX/RES/HEIGHT = 2160\nDX/ANTIALIASING = 2\nDX/FOG = 1\nLANGUAGE = 0\nDX/WINDOWED = 0\n"
        );
    }

    /// `line` sits inside `[section]` (before the next section starts).
    fn in_section(text: &str, section: &str, line: &str) -> bool {
        let Some(start) = text.find(&format!("[{section}]\n")) else {
            return false;
        };
        let body = &text[start + section.len() + 3..];
        let end = body.find("\n[").unwrap_or(body.len());
        body[..end].lines().any(|l| l.trim() == line)
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
        assert!(out.contains("Last_Wave.ogg"));
        assert!(out.contains("ControllerHotPlug = false"));
        assert!(out.contains("DefaultManualTransmission = false"));
        assert!(in_section(&out, "Misc", "AllowCharacterSelection = false"));

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

        // Exclusive fullscreen holds the cap at 60.
        let full = Settings::parse_file("fullscreen = true\nfps_limit = 144\n").0;
        let held = full.apply_tweaks(base);
        assert!(held.contains("FramerateLimit = 60"));
        assert!(held.contains("FramerateInterpolation = false"));
        let borderless = Settings::parse_file("fps_limit = 144\n")
            .0
            .apply_tweaks(base);
        assert!(borderless.contains("FramerateLimit = 144"));

        // The manual gearbox is opt in.
        let manual = Settings::parse_file("manual_gearbox = true\n").0;
        assert!(
            manual
                .apply_tweaks(base)
                .contains("DefaultManualTransmission = true")
        );

        // Everything off: the faithful picture.
        let off = "sky_glow = false\nhd_characters = false\ncd_switcher = false\nskip_logos = false\nfree_drive = false\nhorn = false\nrandom_anims = false\nshow_miles = false\nsoundtrack_fix = false\nvibration = off\ndraw_distance = 0\nreflections = original\n";
        let stock = Settings::parse_file(off).0.apply_tweaks(base);
        assert!(stock.contains("SkyGlowFactor = 0"));
        assert!(stock.contains("UseHiDefCharacters = false"));
        assert!(stock.contains("SwitcherEnable = false"));
        assert!(stock.contains("SkipIntroLogos = false"));
        assert!(stock.contains("DisableCountdownTimer = false"));
        assert!(in_section(&stock, "Audio", "AllowHorn = false"));
        assert!(stock.contains("RandomHighwayAnimSets = false"));
        assert!(stock.contains("ShowOutRunMilesOnMenu = false"));
        assert!(stock.contains("VibrationMode = 0"));
        assert!(stock.contains("DrawDistanceIncrease = 0"));
        assert!(stock.contains("ReflectionResolution = 128"));
    }

    #[test]
    fn tweaks_ini_written_by_older_launchers_is_kept_byte_for_byte() {
        // What the launcher wrote before the SDK (LF, spaced keys, comments):
        // rewriting it with the same settings changes nothing, and a new
        // value changes only its own line.
        let before = Settings::default().apply_tweaks(&format!(
            "# OutRun2006Tweaks\n\n# comment stays\n{}",
            crate::tweaks::BASE_INI
        ));
        assert_eq!(Settings::default().apply_tweaks(&before), before);
        let faster = Settings::parse_file("fps_limit = 120\n")
            .0
            .apply_tweaks(&before);
        assert_eq!(
            faster,
            before
                .replace("FramerateLimit = 60\n", "FramerateLimit = 120\n")
                .replace(
                    "FramerateInterpolation = false\n",
                    "FramerateInterpolation = true\n"
                )
        );
        // An unspaced line the player wrote is rewritten in our spelling,
        // in place.
        let mine = before.replace("VibrationStrength = 7", "vibrationstrength=7");
        assert_eq!(Settings::default().apply_tweaks(&mine), before);
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
        assert!(args(&["--vsync", "half"]).is_err());
        assert!(args(&["--antialiasing", "4"]).is_err());
        assert!(args(&["--reflections", "256"]).is_err());
        assert!(
            args(&["--scaler", "fsr"]).is_err(),
            "the window's setting only"
        );
        assert!(args(&["--character-select", "on"]).is_err());
        // The messages name the flag and end with the usage.
        let bad = args(&["--resolution", "big"]).unwrap_err();
        assert!(bad.starts_with("--resolution needs WIDTHxHEIGHT or original, not big\n\n"));
        assert!(bad.ends_with(USAGE));
        let bad = args(&["--horn", "loud"]).unwrap_err();
        assert!(bad.starts_with("--horn needs on or off, not loud\n\n"));
        assert!(
            args(&["--game"])
                .unwrap_err()
                .starts_with("--game needs a value\n\n")
        );
        assert!(
            args(&["--bogus"])
                .unwrap_err()
                .starts_with("unknown argument --bogus\n\n")
        );
        assert_eq!(
            args(&["--resolution", "off"]).unwrap().resolution,
            Some(ORIGINAL)
        );
    }

    #[test]
    fn retired_and_narrowed_values() {
        // Options that were removed are ignored without a word.
        let (s, warnings) = Settings::parse_file("character_select = true\nhot_plug = false\n");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s, Settings::default());
        let tuned = s.apply_tweaks("");
        assert!(tuned.contains("AllowCharacterSelection = false"));
        assert!(tuned.contains("ControllerHotPlug = false"));

        // Half-rate vsync reads as on, with a warning.
        let (s, warnings) = Settings::parse_file("vsync = half\n");
        assert_eq!(s.vsync, Some(Vsync::On));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("vsync half is no longer offered; using on"));

        // Anti-aliasing is 0 or 2; reflections are 128 (original), 512, 1024, 2048.
        assert_eq!(
            Settings::parse_file("antialiasing = 2\n").0.antialiasing,
            Some(2)
        );
        assert_eq!(Settings::parse_file("antialiasing = 4\n").1.len(), 1);
        assert_eq!(Settings::parse_file("antialiasing = 1\n").1.len(), 1);
        for (text, want) in [
            ("original", 0),
            ("0", 0),
            ("128", 0),
            ("512", 512),
            ("2048", 2048),
        ] {
            let (s, w) = Settings::parse_file(&format!("reflections = {text}\n"));
            assert!(w.is_empty(), "{text}");
            assert_eq!(s.reflections, Some(want));
        }
        assert_eq!(Settings::parse_file("reflections = 256\n").1.len(), 1);
        assert_eq!(Settings::parse_file("reflections = 4096\n").1.len(), 1);

        // Older files with other draw distances still load.
        assert_eq!(
            Settings::parse_file("draw_distance = 6\n").0.draw_distance,
            Some(6)
        );
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
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        // An unknown key is reported and kept in the file.
        assert!(warnings[0].contains("line 2: unknown setting \"future\""));
        assert_eq!(
            warnings[1],
            "launcher.cfg line 6: resolution needs WIDTHxHEIGHT (640x480 to 7680x4320) or original, not \"8000x100\""
        );
        assert_eq!(
            warnings[2],
            "launcher.cfg line 7: deadzone needs 0 to 50, not \"99\""
        );
        let (_, warnings) = Settings::parse_file("fullscreen = maybe\n");
        assert_eq!(
            warnings,
            ["launcher.cfg line 1: fullscreen needs true or false, not \"maybe\""]
        );
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
        // A commented-out example becomes the line, under its comment.
        assert_eq!(
            set_key("# The game\n#game = x\n", "game", "y"),
            "# The game\ngame = y\n"
        );
    }

    #[test]
    fn hd_textures_have_a_switch_and_a_folder() {
        let launcher = Path::new(r"C:\Launcher");
        let on = Settings::parse_file("hd_textures = on\n").0;
        assert_eq!(on.hd_textures, Some(true));
        assert_eq!(
            on.hd_textures_source(launcher),
            Some(launcher.join(TEXTURES_PACK))
        );

        // Off keeps the folder for next time.
        let off = Settings::parse_file("hd_textures = off\nhd_textures_folder = D:\\pack\n").0;
        assert_eq!(off.hd_textures_folder, Some(PathBuf::from(r"D:\pack")));
        assert_eq!(off.hd_textures_source(launcher), None);

        let chosen = Settings::parse_file("hd_textures = on\nhd_textures_folder = D:\\pack\n").0;
        assert_eq!(
            chosen.hd_textures_source(launcher),
            Some(PathBuf::from(r"D:\pack"))
        );

        // Older files named the folder in hd_textures itself.
        let old = Settings::parse_file("hd_textures = D:\\pack\n").0;
        assert_eq!(old.hd_textures, Some(true));
        assert_eq!(old.hd_textures_folder, Some(PathBuf::from(r"D:\pack")));

        // An empty folder line means none.
        let empty = Settings::parse_file("hd_textures_folder =\n").0;
        assert_eq!(empty.hd_textures_folder, None);
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
            "--manual-gearbox",
            "on",
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
        assert_eq!(s.manual_gearbox, Some(true));
        assert_eq!(s.cd_switcher, Some(false));
        assert_eq!(s.music_shuffle, Some(true));
        assert_eq!(s.skip_logos, Some(false));
        assert_eq!(s.free_drive, Some(false));
        assert_eq!(s.horn, Some(false));
        assert_eq!(s.random_anims, Some(false));
        assert_eq!(s.show_miles, Some(false));
        assert_eq!(s.soundtrack_fix, Some(false));
        assert_eq!(s.impulse_triggers, Some(Feedback::Off));
        assert_eq!(s.antialiasing, Some(2));
        assert_eq!(s.reflections, Some(1024));
        assert_eq!(s.anisotropy, Some(8));
        assert_eq!(s.deadzone, Some(10));
        assert_eq!(s.vibration_strength, Some(5));
        assert_eq!(s.hd_textures, Some(false));
    }

    /// The text of `launcher.cfg` after the settings, as `apply_tweaks` sees it.
    fn tweaks_of(cfg: &str) -> String {
        Settings::parse_file(cfg).0.apply_tweaks("")
    }

    #[test]
    fn the_templates_examples_are_the_defaults() {
        // Every `#key = value` example, uncommented. Resolution is the one
        // example that differs from the default (the game's own 640x480).
        let text: String = TEMPLATE
            .lines()
            .filter_map(|line| {
                let rest = line.strip_prefix('#')?;
                let (key, _) = rest.split_once(" = ")?;
                let key_ok =
                    !key.is_empty() && key.bytes().all(|b| b.is_ascii_lowercase() || b == b'_');
                (key_ok && key != "resolution").then(|| format!("{rest}\n"))
            })
            .collect();
        assert!(text.lines().count() > 20, "{text}");
        let (s, warnings) = Settings::parse_file(&text);
        assert!(warnings.is_empty(), "{warnings:?}");
        let d = Settings::default();
        assert_eq!(s.fps_cap(), d.fps_cap());
        assert_eq!(s.backend(), d.backend());
        assert_eq!(s.deadzone_percent(), d.deadzone_percent());
        assert_eq!(s.feedback(), d.feedback());
        assert_eq!(s.impulse(), d.impulse());
        assert_eq!(s.video_resolution(), d.video_resolution());
        assert_eq!(s.scaler.unwrap_or(Scaler::Linear), Scaler::Linear);
        assert_eq!(
            s.hd_textures.unwrap_or(false),
            d.hd_textures.unwrap_or(false)
        );
        assert_eq!(s.outrun2006_ini(), d.outrun2006_ini());
        assert_eq!(s.apply_tweaks(""), d.apply_tweaks(""));
    }

    #[test]
    fn auto_input_backend_means_no_choice() {
        let (s, warnings) = Settings::parse_file("input_backend = auto\n");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.input_backend, None);
        assert_eq!(s.backend_on(true), Backend::XInput);
        let s = args(&["--input-backend", "AUTO"]).unwrap();
        assert_eq!(s.input_backend, None);
        assert_eq!(s.backend_on(false), Backend::WindowsGaming);
        assert!(args(&["--input-backend", "joy"]).is_err());
    }

    #[test]
    fn a_zero_frame_rate_cap_is_unlimited() {
        let (s, warnings) = Settings::parse_file("fps_limit = 0\n");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.fps_cap(), 0);
        assert_eq!(args(&["--fps-limit", "0"]).unwrap().fps_cap(), 0);
        assert!(s.apply_tweaks("").contains("FramerateLimit = 0"));
        assert_eq!(Settings::parse_file("fps_limit = 5\n").1.len(), 1);
    }

    #[test]
    fn settings_reach_the_tweaks_ini() {
        let line = |cfg: &str, section: &str, want: &str| {
            let t = tweaks_of(cfg);
            assert!(in_section(&t, section, want), "{cfg:?}: {want} not in\n{t}");
        };
        line("vsync = off\n", "Performance", "VSync = 0");
        line("", "Performance", "VSync = 1");
        line("anisotropy = 8\n", "Graphics", "AnisotropicFiltering = 8");
        line("", "Graphics", "AnisotropicFiltering = 16");
        line(
            "xbox_brightness = true\n",
            "Graphics",
            "RestoreXboxBrightness = true",
        );
        line(
            "music_shuffle = true\n",
            "CDSwitcher",
            "SwitcherShuffleTracks = true",
        );
        line(
            "fullscreen = true\n",
            "Window",
            "WindowedBorderless = false",
        );
        line("deadzone = 10\n", "Controls", "SteeringDeadZone = 0.10");
        line("vibration = swapped\n", "Controls", "VibrationMode = 2");
        line("vibration = merged\n", "Controls", "VibrationMode = 3");
        line(
            "impulse_triggers = off\n",
            "Controls",
            "ImpulseVibrationMode = 0",
        );
        line(
            "vibration_strength = 3\n",
            "Controls",
            "VibrationStrength = 3",
        );
        line("fps_limit = 90\n", "Performance", "FramerateLimit = 90");
        line(
            "fps_limit = 90\n",
            "Performance",
            "FramerateInterpolation = true",
        );
        line("", "Graphics", "UseHiDefCharacters = true");
        line("", "CDSwitcher", "SwitcherEnable = true");
        line("", "Misc", "SkipIntroLogos = true");
    }

    #[test]
    fn more_flags_are_read() {
        let s = args(&[
            "--runtime",
            "/run/or",
            "--saves",
            "/home/me/saves",
            "--hd-textures-folder",
            "/packs/hd",
            "--vsync",
            "off",
        ])
        .unwrap();
        assert_eq!(s.runtime, Some(PathBuf::from("/run/or")));
        assert_eq!(s.saves, Some(PathBuf::from("/home/me/saves")));
        assert_eq!(s.hd_textures_folder, Some(PathBuf::from("/packs/hd")));
        assert_eq!(s.vsync, Some(Vsync::Off));
    }

    #[test]
    fn only_a_missing_settings_file_reads_as_none() {
        let dir = std::env::temp_dir().join(format!("or-launcher-cfg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(read_file(&dir.join("launcher.cfg")).unwrap(), None);
        // ANSI (Windows-1252) text: read, not taken for a first run.
        std::fs::write(dir.join("ansi.cfg"), b"game = D:\\Jogos\\Corrida \xe9\n").unwrap();
        let text = read_file(&dir.join("ansi.cfg")).unwrap().unwrap();
        assert!(text.starts_with("game = D:\\Jogos\\Corrida "));
        // Anything else is an error, not "no file".
        assert!(read_file(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
