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

use rea_launcher::cfg::spec::{self, Kind, OptionSpec, Value};
use rea_launcher::cfg::{self, KeyStyle, Rejected, Resolution};
use rea_launcher::ini::Ini;

pub use rea_launcher::cfg::{clear_key, read_file, set_key};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub game: Option<PathBuf>,
    pub runtime: Option<PathBuf>,
    /// Where the player's saves live (the game's `SaveGame` folder,
    /// mirrored in before and back after play).
    pub saves: Option<PathBuf>,
    /// A look's name (`LOOKS`): its values for every setting not given.
    pub look: Option<&'static str>,

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

    fn from_key(value: &str) -> Option<Feedback> {
        Feedback::ALL.into_iter().find(|f| f.key() == value)
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

    fn from_key(value: &str) -> Option<Scaler> {
        Scaler::ALL.into_iter().find(|s| s.key() == value)
    }
}

pub const FILE_NAME: &str = "launcher.cfg";

/// A complete `outrun2006.ini` with the game's own defaults (the format
/// the game's setup tool writes), for a copy of the game without one.
pub const OUTRUN2006_SEED: &str = "DX/ADAPTER = 0\nDX/RES/WIDTH = 640\nDX/RES/HEIGHT = 480\nDX/ANTIALIASING = 0\nDX/FOG = 1\nLANGUAGE = 0\nDX/WINDOWED = 1\n";

/// The game's keys launcher 0.1.0 reset on every start, which later
/// versions keep.
pub const OUTRUN2006_GAME_KEYS: [&str; 3] = ["LANGUAGE", "DX/ADAPTER", "DX/FOG"];

/// True for an `outrun2006.ini` exactly as launcher 0.1.0 wrote it whole
/// on every start (adapter 0, fog on, language 0): its language, adapter
/// and fog were the launcher's, not the player's.
pub fn outrun2006_ini_from_v010(text: &str) -> bool {
    let number = |v: &str| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit());
    let Some(body) = text.strip_suffix('\n') else {
        return false;
    };
    let lines: Vec<&str> = body.split('\n').collect();
    let keys = [
        "DX/ADAPTER = ",
        "DX/RES/WIDTH = ",
        "DX/RES/HEIGHT = ",
        "DX/ANTIALIASING = ",
        "DX/FOG = ",
        "LANGUAGE = ",
        "DX/WINDOWED = ",
    ];
    lines.len() == keys.len()
        && lines.iter().zip(keys).enumerate().all(|(i, (line, key))| {
            line.strip_prefix(key).is_some_and(|value| match i {
                0 | 5 => value == "0",
                4 => value == "1",
                6 => value == "0" || value == "1",
                _ => number(value),
            })
        })
}

/// `runtime` (an `outrun2006.ini` from launcher 0.1.0) with the language,
/// adapter and fog of the game folder's copy `games`.
pub fn outrun2006_ini_upgrade(runtime: &str, games: &str) -> String {
    let games = Ini::parse(games);
    let mut ini = Ini::parse(runtime);
    for key in OUTRUN2006_GAME_KEYS {
        if let Some(value) = games.get("", key) {
            ini.set("", key, value.trim());
        }
    }
    ini.to_text().unwrap_or_else(|| runtime.to_owned())
}

/// `resolution` value meaning the game's own 640x480.
pub const ORIGINAL: (u32, u32) = (640, 480);

/// The flags that take no value: the launcher's modes.
pub const SWITCHES: [&str; 6] = [
    "--no-window",
    "--settings",
    "--linux",
    "--check",
    "--download-hd-pack",
    "--download-directx",
];

/// Whether a run opens the settings window: `--settings`, or no option at
/// all. `--linux` only adds the Linux scaler to the window, so it alone
/// still opens it; `--no-window`, `--check`, `--download-hd-pack` and any
/// option (`--game`, `--fov`…) play or act without it. Decided on the
/// parsed flags, so `--linux` is never mistaken for an option.
pub fn opens_window(args: &Settings, switches: &[String]) -> bool {
    let has = |name: &str| switches.iter().any(|s| s == name);
    has("--settings") || (*args == Settings::default() && switches.iter().all(|s| s == "--linux"))
}

/// The looks, as `--look` and `look =` name them (the window's cards).
pub const LOOKS: &[(&str, &str)] = &[
    ("original", "Original 2006"),
    ("coast_to_coast", "Coast to Coast"),
    ("steam_deck", "Steam Deck"),
];

/// One row per setting: the `launcher.cfg` key, its `--flag`, the values
/// it takes and the words the window, `--help` and the template show.
/// The examples are the launcher's defaults (Coast to Coast), except the
/// resolution's (unset, it is the game's own 640x480).
pub const OPTIONS: &[OptionSpec] = &[
    OptionSpec::new(
        "game",
        Kind::Path,
        "Game folder",
        "The folder with your installed OutRun 2006 (OR2006C2C.exe). It is only read, never changed.",
    ),
    OptionSpec::new(
        "runtime",
        Kind::Path,
        "Runtime folder",
        "Where the launcher keeps its private, writable copy of the game. Default: %LOCALAPPDATA%\\OutRunLauncher\\OutRun2006.",
    )
    .launcher_only(),
    OptionSpec::new(
        "saves",
        Kind::Path,
        "Saves",
        "Where your saves (License.dat, your records) are kept: copied into the game before it starts and back when it ends. Default: Saved Games\\OutRun 2006.",
    ),
    OptionSpec::new(
        "look",
        Kind::Choice(LOOKS),
        "Look",
        "A preset for every picture, control and extra setting at once: the game as it shipped, the tuned Coast to Coast set for your screen, or the Steam Deck's. Settings given on their own win over it.",
    ),
    OptionSpec::new(
        "resolution",
        Kind::Size {
            min: (640, 480),
            max: (7680, 4320),
            aliases: cfg::ORIGINAL_OR_OFF,
        },
        "Resolution",
        "The size the game draws at. Unset or original: the game's own 640x480.",
    )
    .with_example("1920x1080"),
    OptionSpec::new(
        "fullscreen",
        Kind::Toggle,
        "Exclusive fullscreen",
        "Takes over the screen instead of a borderless window. Holds the frame rate at 60.",
    )
    .with_example("off"),
    OptionSpec::new(
        "fps_limit",
        Kind::Range { min: 10, max: 240 },
        "Frame rate",
        "Frames per second: 10 to 240, or 0 (unlimited). Above 60 the picture is interpolated between the game's 60 ticks a second (experimental).",
    )
    .with_value_aliases(&[("unlimited", "0"), ("none", "0"), ("off", "0")])
    .with_needs(FPS_NEEDS)
    .with_placeholder("<10-240|unlimited>")
    .with_example("60"),
    OptionSpec::new(
        "vsync",
        Kind::Toggle,
        "Vsync",
        "Waits for the screen's refresh: no tearing, a little more delay.",
    )
    .with_example("on"),
    OptionSpec::new(
        "draw_distance",
        Kind::Range { min: 0, max: 8 },
        "Draw distance",
        "How much further down the road the scenery is drawn (0 is the game's own). Far out, scenery can pop in.",
    )
    .with_example("4"),
    OptionSpec::new(
        "reflections",
        Kind::Choice(&[
            ("original", "Original"),
            ("512", "512"),
            ("1024", "1024"),
            ("2048", "2048"),
        ]),
        "Reflections",
        "How sharp the reflections on the car are, in pixels (original is 128).",
    )
    .with_value_aliases(&[("0", "original"), ("128", "original"), ("off", "original")])
    .with_example("2048"),
    OptionSpec::new(
        "anisotropy",
        Kind::Range { min: 0, max: 16 },
        "Anisotropy",
        "Keeps the road's texture sharp into the distance (0 is the game's own filtering).",
    )
    .with_value_aliases(&[("off", "0"), ("original", "0")])
    .with_example("16"),
    OptionSpec::new(
        "antialiasing",
        Kind::Choice(&[("0", "Off"), ("2", "4× MSAA")]),
        "Anti-aliasing",
        "Smooths jagged edges (the game's own multisampling; on NVIDIA under Windows, trackside cloth and barriers too).",
    )
    .with_example("0"),
    OptionSpec::new(
        "sky_glow",
        Kind::Toggle,
        "Sky glow",
        "The console versions' glowing sky.",
    )
    .with_example("on"),
    OptionSpec::new(
        "xbox_brightness",
        Kind::Toggle,
        "Xbox brightness",
        "The brighter picture of the Xbox release.",
    )
    .with_example("off"),
    OptionSpec::new(
        "hd_characters",
        Kind::Toggle,
        "Hi-def characters",
        "The detailed driver and passenger models the PC release shipped but never used.",
    )
    .with_example("on"),
    OptionSpec::new(
        "scaler",
        Kind::Choice(&[
            ("nearest", "Sharp"),
            ("pixel", "Even pixels"),
            ("fsr", "FSR"),
            ("linear", "Smooth"),
        ]),
        "Scaler",
        "Linux: how the picture is enlarged to your screen: sharp blocks, even blocks, AMD FSR, or smooth.",
    )
    .with_example("linear")
    .helper_owned(),
    OptionSpec::new(
        "hd_textures",
        Kind::Toggle,
        "Use the HD texture pack",
        "The community's sharper menus, HUD, sprites and button prompts. Get it in the HD Pack tab or with --download-hd-pack.",
    )
    .with_example("off"),
    OptionSpec::new(
        "hd_textures_folder",
        Kind::Path,
        "Pack folder",
        "Where the HD pack is (the folder it was unzipped into). Empty: the one the launcher downloaded, or textures-pack next to OutRunLauncher.exe.",
    ),
    OptionSpec::new(
        "deadzone",
        Kind::Range { min: 0, max: 50 },
        "Deadzone",
        "How far the stick moves before the car steers, in percent (the game's own is 20).",
    )
    .with_example("20"),
    OptionSpec::new(
        "vibration",
        Kind::Choice(&[
            ("off", "Off"),
            ("on", "On"),
            ("swapped", "Swapped"),
            ("merged", "Merged"),
        ]),
        "Rumble",
        "The pad shakes on shifts, drifts and crashes. Swapped trades the left and right motors; merged drives both together.",
    )
    .with_example("on"),
    OptionSpec::new(
        "vibration_strength",
        Kind::Range { min: 0, max: 10 },
        "Strength",
        "How strongly the pad shakes, 0 to 10.",
    )
    .with_example("7"),
    OptionSpec::new(
        "impulse_triggers",
        Kind::Choice(&[
            ("off", "Off"),
            ("on", "On"),
            ("swapped", "Swapped"),
            ("merged", "Merged"),
        ]),
        "Impulse triggers",
        "Rumble in the triggers of Xbox One and Series pads, on Windows.",
    )
    .with_example("swapped"),
    OptionSpec::new(
        "input_backend",
        Kind::Choice(&[
            ("auto", "Automatic"),
            ("wgi", "WGI"),
            ("raw", "RawInput"),
            ("dinput", "DInput"),
            ("xinput", "XInput"),
        ]),
        "Input API",
        "How the game reads pads; change it only if a pad isn't seen. Automatic: Windows.Gaming.Input (WGI) on Windows, XInput under Wine and Proton.",
    )
    .with_example("auto"),
    OptionSpec::new(
        "manual_gearbox",
        Kind::Toggle,
        "Manual gearbox",
        "Races start with the manual gearbox selected.",
    )
    .with_example("off"),
    OptionSpec::new(
        "cd_switcher",
        Kind::Toggle,
        "CD switcher",
        "Change the music during a race: Back or X for the next track, Z for the previous one.",
    )
    .with_example("on"),
    OptionSpec::new(
        "music_shuffle",
        Kind::Toggle,
        "Shuffle the music",
        "The race music comes in a new order each time the game starts.",
    )
    .with_example("off"),
    OptionSpec::new(
        "skip_logos",
        Kind::Toggle,
        "Skip the boot logos",
        "Straight to the title screen (a white screen shows for a moment while the game loads).",
    )
    .with_example("on"),
    OptionSpec::new(
        "free_drive",
        Kind::Toggle,
        "Free drive",
        "No countdown timer: drive as long as you like.",
    )
    .with_example("on"),
    OptionSpec::new(
        "horn",
        Kind::Toggle,
        "Horn any time",
        "The horn works all the time, not only when your passenger asks for it.",
    )
    .with_example("on"),
    OptionSpec::new(
        "random_anims",
        Kind::Toggle,
        "Roadside animations",
        "Roadside animations from every game mode can appear on any stage.",
    )
    .with_example("on"),
    OptionSpec::new(
        "show_miles",
        Kind::Toggle,
        "OutRun miles",
        "Shows your OutRun miles next to the menu's sign-in text.",
    )
    .with_example("on"),
    OptionSpec::new(
        "soundtrack_fix",
        Kind::Toggle,
        "Soundtrack fix",
        "Restores the first two seconds of \"Rush a Difficulty\" (an official patch to the launcher's copy of the song).",
    )
    .with_example("on"),
    OptionSpec::removed(
        "character_select",
        "Removed by the options audit: the mod's character select can crash the game.",
    ),
    OptionSpec::removed(
        "hot_plug",
        "Removed by the options audit: the mod's new input ignores it; hot-plug is always off.",
    ),
];

/// The row for `key`.
pub fn option(key: &str) -> &'static OptionSpec {
    spec::find(OPTIONS, key).unwrap_or_else(|| panic!("no option row for {key}"))
}

/// The comment heading of `launcher.cfg`.
const HEADING: &str = "OutRun 2006 launcher settings.

One setting per line as key = value; lines starting with # are comments. Command-line options (--resolution 1920x1080) override this file for one run. A launcher.cfg next to OutRunLauncher.exe is used instead of this one when it exists (portable installs).

Unset settings get the launcher's defaults (the Coast to Coast look), shown in each #key = line. Everything is applied to the launcher's private copy of the game; your game folder is never changed.";

/// The file written on first run: every option commented out with its
/// description, values and default.
pub fn template() -> String {
    spec::template(HEADING, OPTIONS)
}

/// `--help`.
pub fn usage() -> String {
    format!(
        "OutRunLauncher {version}

OutRunLauncher [options]
  Without options the settings window opens first, then the game.

{options}
These override launcher.cfg for one run; only --game is remembered.

  --no-window          play with the saved settings (and the options given)
  --settings [--linux] only the settings window (exit codes 0: play,
                       3: closed, 4: no window could be shown; --linux adds
                       the gamescope scaler, for or-play)
  --check              checks the game folder and the launcher's files
                       without playing (report in launcher.log)
  --download-hd-pack   downloads the community HD texture pack (progress in
                       launcher.log) and switches it on in launcher.cfg
  --download-directx   downloads Microsoft's June 2010 DirectX 9 DLLs
                       (d3dx9_29.dll, …) for clean Windows 10/11 installs
  --help               this text

Every option is described in launcher.cfg.
",
        version = env!("CARGO_PKG_VERSION"),
        options = spec::usage(OPTIONS, KeyStyle::Underscore)
    )
}

/// What a frame-rate cap may be, for messages.
const FPS_NEEDS: &str = "10 to 240 or unlimited (0)";

impl Settings {
    /// Reads a settings file; the second value lists lines it could not use.
    pub fn parse_file(text: &str) -> (Settings, Vec<String>) {
        let mut s = Settings::default();
        let mut warnings = cfg::parse(text, FILE_NAME, |key, value| match key {
            // Older files named the pack's folder in hd_textures itself.
            "hd_textures" if cfg::parse_bool(value).is_none() => {
                s.hd_textures = Some(true);
                s.hd_textures_folder = Some(value.trim().into());
                Ok(())
            }
            // Retired half-rate vsync: read as on (warned below).
            "vsync" if value.trim().eq_ignore_ascii_case("half") => {
                s.vsync = Some(Vsync::On);
                Ok(())
            }
            _ => spec::dispatch(OPTIONS, key, value, &mut |k, v| s.apply(k, v)),
        });
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

    /// One checked value from the file or a flag.
    fn apply(&mut self, key: &str, value: Value) -> Result<(), Rejected> {
        let path = |text: &str| Some(text).filter(|t| !t.is_empty()).map(PathBuf::from);
        match (key, value) {
            ("game", Value::Text(t)) => self.game = path(t),
            ("runtime", Value::Text(t)) => self.runtime = path(t),
            ("saves", Value::Text(t)) => self.saves = path(t),
            ("hd_textures_folder", Value::Text(t)) => self.hd_textures_folder = path(t),
            ("look", Value::Choice(look)) => self.look = Some(look),
            ("resolution", Value::Size(size)) => {
                self.resolution = Some(match size {
                    Resolution::Size(w, h) => (w, h),
                    _ => ORIGINAL,
                })
            }
            ("fps_limit", Value::Number(n)) => self.fps_limit = Some(n),
            ("vsync", Value::Toggle(on)) => {
                self.vsync = Some(if on { Vsync::On } else { Vsync::Off })
            }
            ("draw_distance", Value::Number(n)) => self.draw_distance = Some(n),
            ("reflections", Value::Choice(r)) => self.reflections = Some(r.parse().unwrap_or(0)),
            ("anisotropy", Value::Number(n)) => self.anisotropy = Some(n),
            ("antialiasing", Value::Choice(aa)) => self.antialiasing = aa.parse().ok(),
            ("scaler", Value::Choice(s)) => self.scaler = Scaler::from_key(s),
            ("deadzone", Value::Number(n)) => self.deadzone = Some(n),
            ("vibration", Value::Choice(f)) => self.vibration = Feedback::from_key(f),
            ("vibration_strength", Value::Number(n)) => self.vibration_strength = Some(n),
            ("impulse_triggers", Value::Choice(f)) => self.impulse_triggers = Feedback::from_key(f),
            ("input_backend", Value::Choice(b)) => {
                self.input_backend = Backend::ALL.into_iter().find(|x| x.key() == b)
            }
            (key, Value::Toggle(on)) => *self.toggle(key).ok_or(Rejected::Unknown)? = Some(on),
            _ => return Err(Rejected::Unknown),
        }
        Ok(())
    }

    /// The on/off setting called `key`.
    pub fn toggle(&mut self, key: &str) -> Option<&mut Option<bool>> {
        Some(match key {
            "fullscreen" => &mut self.fullscreen,
            "sky_glow" => &mut self.sky_glow,
            "xbox_brightness" => &mut self.xbox_brightness,
            "hd_characters" => &mut self.hd_characters,
            "hd_textures" => &mut self.hd_textures,
            "manual_gearbox" => &mut self.manual_gearbox,
            "cd_switcher" => &mut self.cd_switcher,
            "music_shuffle" => &mut self.music_shuffle,
            "skip_logos" => &mut self.skip_logos,
            "free_drive" => &mut self.free_drive,
            "horn" => &mut self.horn,
            "random_anims" => &mut self.random_anims,
            "show_miles" => &mut self.show_miles,
            "soundtrack_fix" => &mut self.soundtrack_fix,
            _ => return None,
        })
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
            look,
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

    /// The game's `outrun2006.ini`: `existing` (the runtime's copy, else
    /// the game folder's) with the launcher's keys set, the video mode, the
    /// anti-aliasing and windowed or not; every other line (the language,
    /// the adapter, fog) stays the game's. Without a file, a complete one
    /// with the game's own defaults is seeded.
    pub fn outrun2006_ini(&self, existing: Option<&str>) -> String {
        let mut ini = Ini::parse(existing.unwrap_or(OUTRUN2006_SEED));
        if !ini.is_text() {
            ini = Ini::parse(OUTRUN2006_SEED);
        }
        let (w, h) = self.video_resolution().unwrap_or(ORIGINAL);
        let windowed = u32::from(!self.fullscreen.unwrap_or(false));
        ini.set("", "DX/RES/WIDTH", &w.to_string());
        ini.set("", "DX/RES/HEIGHT", &h.to_string());
        ini.set(
            "",
            "DX/ANTIALIASING",
            &self.antialiasing.unwrap_or(0).to_string(),
        );
        ini.set("", "DX/WINDOWED", &windowed.to_string());
        ini.to_text().expect("parsed from text")
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

    /// Command-line values (`--fps-limit 120`), and the switches given
    /// (`--no-window`, ...) in order. `Err` is the message to show: the
    /// usage for `--help`, else what was wrong and the usage.
    pub fn parse_args<I: IntoIterator<Item = String>>(
        args: I,
    ) -> Result<(Settings, Vec<String>), String> {
        let mut s = Settings::default();
        match spec::parse_args(args, KeyStyle::Underscore, OPTIONS, &SWITCHES, |k, v| {
            s.apply(k, v)
        }) {
            Ok(switches) => Ok((s, switches)),
            Err(cfg::ArgError::Help(_)) => Err(usage()),
            Err(e) => Err(format!("{e}\n\n{}", usage())),
        }
    }

    /// The settings a run uses: the file's, over the look it names; then
    /// a `--look` from the command line over those; then the other flags.
    /// `preset` gives a look's settings (`form::look_settings`).
    pub fn combine(file: Settings, args: Settings, preset: &dyn Fn(&str) -> Settings) -> Settings {
        let mut base = match file.look {
            Some(look) => preset(look).merged(file),
            None => file,
        };
        if let Some(look) = args.look {
            base = base.merged(preset(look));
        }
        base.merged(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Settings, String> {
        Settings::parse_args(list.iter().map(|s| s.to_string())).map(|(s, _)| s)
    }

    #[test]
    fn template_parses_to_the_defaults() {
        let (s, warnings) = Settings::parse_file(template().as_str());
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s, Settings::default());
        assert_eq!(s.video_resolution(), None);
        assert_eq!(s.fps_cap(), 60);
        assert_eq!(s.deadzone_percent(), 20);
    }

    #[test]
    fn outrun2006_ini_matches_the_games_format() {
        let s = Settings::default();
        assert_eq!(s.outrun2006_ini(None), OUTRUN2006_SEED);
        let s =
            Settings::parse_file("resolution = 3840x2160\nfullscreen = true\nantialiasing = 2\n").0;
        assert_eq!(
            s.outrun2006_ini(None),
            "DX/ADAPTER = 0\nDX/RES/WIDTH = 3840\nDX/RES/HEIGHT = 2160\nDX/ANTIALIASING = 2\nDX/FOG = 1\nLANGUAGE = 0\nDX/WINDOWED = 0\n"
        );
        // The game's language, adapter and fog (and the line endings) stay.
        let games = "DX/ADAPTER = 1\r\nDX/RES/WIDTH = 800\r\nDX/RES/HEIGHT = 600\r\nDX/ANTIALIASING = 0\r\nDX/FOG = 0\r\nLANGUAGE = 3\r\nDX/WINDOWED = 0\r\n";
        assert_eq!(
            s.outrun2006_ini(Some(games)),
            "DX/ADAPTER = 1\r\nDX/RES/WIDTH = 3840\r\nDX/RES/HEIGHT = 2160\r\nDX/ANTIALIASING = 2\r\nDX/FOG = 0\r\nLANGUAGE = 3\r\nDX/WINDOWED = 0\r\n"
        );
    }

    #[test]
    fn outrun2006_ini_from_v010_is_recognised_and_upgraded() {
        let old = "DX/ADAPTER = 0\nDX/RES/WIDTH = 1920\nDX/RES/HEIGHT = 1080\nDX/ANTIALIASING = 2\nDX/FOG = 1\nLANGUAGE = 0\nDX/WINDOWED = 1\n";
        assert!(outrun2006_ini_from_v010(old));
        assert!(outrun2006_ini_from_v010(OUTRUN2006_SEED));
        for other in [
            old.replace("LANGUAGE = 0", "LANGUAGE = 2"),
            old.replace("DX/ADAPTER = 0", "DX/ADAPTER = 1"),
            old.replace("DX/FOG = 1", "DX/FOG = 0"),
            old.replace('\n', "\r\n"),
            format!("{old}EXTRA = 1\n"),
            old.trim_end().to_string(),
        ] {
            assert!(!outrun2006_ini_from_v010(&other), "{other:?}");
        }
        let games = "DX/ADAPTER = 1\r\nDX/RES/WIDTH = 800\r\nDX/RES/HEIGHT = 600\r\nDX/ANTIALIASING = 0\r\nDX/FOG = 0\r\nLANGUAGE = 3\r\nDX/WINDOWED = 0\r\n";
        assert_eq!(
            outrun2006_ini_upgrade(old, games),
            "DX/ADAPTER = 1\nDX/RES/WIDTH = 1920\nDX/RES/HEIGHT = 1080\nDX/ANTIALIASING = 2\nDX/FOG = 0\nLANGUAGE = 3\nDX/WINDOWED = 1\n"
        );
    }

    #[test]
    fn the_window_opens_without_options_and_with_linux_alone() {
        let opens = |list: &[&str]| {
            let (args, switches) =
                Settings::parse_args(list.iter().map(|s| s.to_string())).unwrap();
            opens_window(&args, &switches)
        };
        assert!(opens(&[]));
        assert!(opens(&["--linux"]), "or-play's window run");
        assert!(opens(&["--settings"]) && opens(&["--settings", "--linux"]));
        assert!(!opens(&["--no-window"]));
        assert!(!opens(&["--no-window", "--linux"]));
        assert!(!opens(&["--check"]) && !opens(&["--linux", "--check"]));
        assert!(!opens(&["--download-hd-pack"]));
        assert!(!opens(&["--game", "C:\\Games\\OutRun"]));
        assert!(!opens(&["--linux", "--deadzone", "10"]));
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
        // The Linux helper's scaler is a flag too (or-play reads it).
        assert_eq!(
            args(&["--scaler", "fsr"]).unwrap().scaler,
            Some(Scaler::Fsr)
        );
        assert!(args(&["--scaler", "bicubic"]).is_err());
        // A removed option is taken and ignored, as in launcher.cfg.
        assert_eq!(
            args(&["--character-select", "on"]).unwrap(),
            Settings::default()
        );
        assert!(!usage().contains("--character-select"));
        // The messages name the flag and end with the usage.
        let bad = args(&["--resolution", "big"]).unwrap_err();
        assert!(bad.starts_with("--resolution needs WIDTHxHEIGHT, original or off, not big\n\n"));
        assert!(bad.ends_with(&usage()));
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
            "launcher.cfg line 6: resolution needs WIDTHxHEIGHT, original or off, not \"8000x100\""
        );
        assert_eq!(
            warnings[2],
            "launcher.cfg line 7: deadzone needs 0 to 50, not \"99\""
        );
        let (_, warnings) = Settings::parse_file("fullscreen = maybe\n");
        assert_eq!(
            warnings,
            ["launcher.cfg line 1: fullscreen needs on or off, not \"maybe\""]
        );
    }

    #[test]
    fn set_key_keeps_the_rest_of_the_file() {
        let filled = set_key(template().as_str(), "game", r"D:\OutRun 2006");
        assert_eq!(filled.lines().count(), template().as_str().lines().count());
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
        let text: String = template()
            .as_str()
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
        assert_eq!(s.outrun2006_ini(None), d.outrun2006_ini(None));
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
