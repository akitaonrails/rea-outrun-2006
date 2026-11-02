//! What the settings window shows and saves, kept apart from the drawing
//! so it can be tested on any host.
//!
//! The window offers one choice first, the look: the original 2006 game,
//! the Coast to Coast preset that fits the player's screen with the tuned
//! settings this launcher is built around, a light Steam Deck set, or
//! custom values. The preset is never stored: the file keeps the individual
//! settings and the look is worked out from them when the window opens.

use std::path::PathBuf;

use crate::config::{self, Backend, Feedback, Settings, Vsync};

pub use crate::config::Scaler;

pub const SCALERS: [(Scaler, &str, &str); 4] = [
    (
        Scaler::Nearest,
        "Sharp",
        "Every game pixel becomes a crisp block.",
    ),
    (
        Scaler::Pixel,
        "Even pixels",
        "Crisp blocks, with only their edges blended so all are the same size.",
    ),
    (
        Scaler::Fsr,
        "FSR",
        "AMD FSR: smooth, then sharpened. Softer than Sharp.",
    ),
    (
        Scaler::Linear,
        "Smooth",
        "Plain smoothing. The softest look (the default).",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Original,
    CoastToCoast,
    SteamDeck,
    Custom,
}

/// Everything the window edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    pub game: String,
    pub saves: String,
    /// Remembered so a look picked by name stays picked even when its
    /// values match another (and Custom stays open while editing).
    pub look: Look,
    /// Running under Wine or Proton: picks the default input API, which is
    /// not written to the file.
    pub wine: bool,

    // Picture
    pub resolution: Option<(u32, u32)>,
    pub fullscreen: bool,
    pub antialiasing: u32,
    pub fps_limit: u32,
    pub vsync: Vsync,
    pub draw_distance: u32,
    pub reflections: u32,
    pub anisotropy: u32,
    pub sky_glow: bool,
    pub xbox_brightness: bool,
    pub hd_characters: bool,
    pub hd_textures: bool,

    // Controls
    pub deadzone: u32,
    pub vibration: Feedback,
    pub vibration_strength: u32,
    pub impulse_triggers: Feedback,
    pub input_backend: Backend,
    pub manual_gearbox: bool,

    // Extras
    pub cd_switcher: bool,
    pub music_shuffle: bool,
    pub skip_logos: bool,
    pub free_drive: bool,
    pub horn: bool,
    pub random_anims: bool,
    pub show_miles: bool,
    pub soundtrack_fix: bool,

    pub scaler: Scaler,
    /// Where the unzipped HD texture pack is (empty: `textures-pack` next to
    /// the launcher). Kept when the pack is switched off.
    pub hd_folder: String,
}

/// The Steam Deck's screen (1280x800, 16:10).
pub const STEAM_DECK_SCREEN: (u32, u32) = (1280, 800);

/// A screen size as a landscape resolution, or `None` when it is too small
/// to be worth it.
fn landscape((a, b): (u32, u32)) -> Option<(u32, u32)> {
    let (w, h) = (a.max(b) & !1, a.min(b) & !1);
    (w > 640 && h > 480 && w <= 7680 && h <= 4320).then_some((w, h))
}

impl Form {
    /// The original 2006 game: stock picture, stock extras, in a borderless
    /// window (exclusive fullscreen is fragile and caps the frame rate).
    pub fn original(wine: bool) -> Form {
        Form {
            look: Look::Original,
            wine,
            resolution: None,
            fullscreen: false,
            antialiasing: 0,
            fps_limit: 60,
            vsync: Vsync::On,
            draw_distance: 0,
            reflections: 0,
            anisotropy: 0,
            sky_glow: false,
            xbox_brightness: false,
            hd_characters: false,
            hd_textures: false,
            deadzone: 20,
            vibration: Feedback::Off,
            vibration_strength: 7,
            impulse_triggers: Feedback::Off,
            input_backend: Backend::default_on(wine),
            manual_gearbox: false,
            cd_switcher: false,
            music_shuffle: false,
            skip_logos: false,
            free_drive: false,
            horn: false,
            random_anims: false,
            show_miles: false,
            soundtrack_fix: false,
            scaler: Scaler::Linear,
            hd_folder: String::new(),
            game: String::new(),
            saves: String::new(),
        }
    }

    /// The tuned set this launcher is built around: the screen's own size
    /// and the distrobox-gaming known-good configuration. Always a
    /// borderless window: exclusive fullscreen is fragile under Wine and on
    /// mixed-monitor desktops.
    pub fn coast_to_coast(screen: Option<(u32, u32)>, wine: bool) -> Form {
        Form {
            look: Look::CoastToCoast,
            wine,
            resolution: screen.and_then(landscape),
            fullscreen: false,
            antialiasing: 0,
            fps_limit: 60,
            vsync: Vsync::On,
            draw_distance: 4,
            reflections: 2048,
            anisotropy: 16,
            sky_glow: true,
            xbox_brightness: false,
            hd_characters: true,
            hd_textures: false,
            deadzone: 20,
            vibration: Feedback::On,
            vibration_strength: 7,
            impulse_triggers: Feedback::Swapped,
            input_backend: Backend::default_on(wine),
            manual_gearbox: false,
            cd_switcher: true,
            music_shuffle: false,
            skip_logos: true,
            free_drive: true,
            horn: true,
            random_anims: true,
            show_miles: true,
            soundtrack_fix: true,
            hd_folder: String::new(),
            scaler: Scaler::Linear,
            game: String::new(),
            saves: String::new(),
        }
    }

    /// The Steam Deck's screen with a light load for its battery.
    pub fn steam_deck() -> Form {
        Form {
            look: Look::SteamDeck,
            resolution: Some(STEAM_DECK_SCREEN),
            fps_limit: 60,
            reflections: 1024,
            // The Deck runs the game under Proton.
            ..Form::coast_to_coast(Some(STEAM_DECK_SCREEN), true)
        }
    }

    /// The window's starting state for saved settings.
    pub fn new(s: &Settings, screen: Option<(u32, u32)>, wine: bool) -> Form {
        let path = |p: &Option<PathBuf>| {
            p.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        };
        Form {
            game: path(&s.game),
            saves: path(&s.saves),
            look: Look::Custom,
            wine,
            resolution: s.video_resolution(),
            fullscreen: s.fullscreen.unwrap_or(false),
            antialiasing: s.antialiasing.unwrap_or(0),
            fps_limit: s.fps_limit.unwrap_or(60),
            vsync: s.vsync.unwrap_or(Vsync::On),
            draw_distance: s.draw_distance.unwrap_or(4),
            reflections: s.reflections.unwrap_or(2048),
            anisotropy: s.anisotropy.unwrap_or(16),
            sky_glow: s.sky_glow.unwrap_or(true),
            xbox_brightness: s.xbox_brightness.unwrap_or(false),
            hd_characters: s.hd_characters.unwrap_or(true),
            hd_textures: s.hd_textures.unwrap_or(false),
            deadzone: s.deadzone.unwrap_or(20),
            vibration: s.vibration.unwrap_or(Feedback::On),
            vibration_strength: s.vibration_strength.unwrap_or(7),
            impulse_triggers: s.impulse_triggers.unwrap_or(Feedback::Swapped),
            input_backend: s.backend_on(wine),
            manual_gearbox: s.manual_gearbox.unwrap_or(false),
            cd_switcher: s.cd_switcher.unwrap_or(true),
            music_shuffle: s.music_shuffle.unwrap_or(false),
            skip_logos: s.skip_logos.unwrap_or(true),
            free_drive: s.free_drive.unwrap_or(true),
            horn: s.horn.unwrap_or(true),
            random_anims: s.random_anims.unwrap_or(true),
            show_miles: s.show_miles.unwrap_or(true),
            soundtrack_fix: s.soundtrack_fix.unwrap_or(true),
            scaler: s.scaler.unwrap_or(Scaler::Linear),
            hd_folder: path(&s.hd_textures_folder),
        }
        .derived_look(screen, wine)
    }

    /// The form for a first run: the preset that suits the screen.
    pub fn first_run(screen: Option<(u32, u32)>, wine: bool) -> Form {
        match recommended(screen) {
            Look::SteamDeck => Form {
                wine,
                ..Form::steam_deck()
            },
            _ => Form::coast_to_coast(screen, wine),
        }
    }

    /// The form for a settings file's text: the recommended look when there
    /// is no file yet (or it is empty), otherwise what the file says. A file
    /// that happens to match the template is the player's, not a first run.
    pub fn from_file(saved: Option<&str>, screen: Option<(u32, u32)>, wine: bool) -> Form {
        match saved {
            Some(text) if !text.trim().is_empty() => {
                Form::new(&Settings::parse_file(text).0, screen, wine)
            }
            _ => Form::first_run(screen, wine),
        }
    }

    /// The look these values make on a screen.
    pub fn look_of(form: &Form, screen: Option<(u32, u32)>, wine: bool) -> Look {
        let mut probe = form.clone();
        probe.look = Look::Original;
        probe.game.clear();
        probe.saves.clear();
        probe.hd_folder.clear();
        // The pack, the scaler and the platform are not part of a look.
        probe.hd_textures = false;
        probe.scaler = Scaler::Linear;
        probe.wine = wine;
        if probe == Form::original(wine) {
            return Look::Original;
        }
        probe.look = Look::CoastToCoast;
        if probe == Form::coast_to_coast(screen, wine) {
            return Look::CoastToCoast;
        }
        probe.look = Look::SteamDeck;
        if probe
            == (Form {
                wine,
                ..Form::steam_deck()
            })
        {
            return Look::SteamDeck;
        }
        Look::Custom
    }

    fn derived_look(mut self, screen: Option<(u32, u32)>, wine: bool) -> Form {
        self.look = Form::look_of(&self, screen, wine);
        self
    }

    /// Picks a look; Custom keeps the current values to edit.
    pub fn set_look(&mut self, look: Look, screen: Option<(u32, u32)>, wine: bool) {
        self.look = look;
        let game = std::mem::take(&mut self.game);
        let saves = std::mem::take(&mut self.saves);
        let hd_folder = std::mem::take(&mut self.hd_folder);
        let scaler = self.scaler;
        let hd_textures = self.hd_textures;
        *self = match look {
            Look::Original => Form::original(wine),
            Look::CoastToCoast => Form::coast_to_coast(screen, wine),
            Look::SteamDeck => Form {
                wine,
                ..Form::steam_deck()
            },
            Look::Custom => return,
        };
        self.game = game;
        self.saves = saves;
        self.hd_folder = hd_folder;
        self.scaler = scaler;
        self.hd_textures = hd_textures;
    }

    /// The settings file with this form's values written in; every other
    /// line (comments, keys the window doesn't show) is kept.
    pub fn save(&self, text: &str) -> String {
        let mut t = text.to_string();
        let mut put = |key: &str, value: Option<String>| {
            t = match &value {
                Some(v) => config::set_key(&t, key, v),
                None => config::clear_key(&t, key),
            };
        };
        let path = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        put("game", path(&self.game));
        put("saves", path(&self.saves));
        put(
            "resolution",
            self.resolution.map(|(w, h)| format!("{w}x{h}")),
        );
        put("fullscreen", Some(self.fullscreen.to_string()));
        put("antialiasing", Some(self.antialiasing.to_string()));
        put(
            "fps_limit",
            Some(match self.fps_limit {
                0 => "unlimited".to_string(),
                n => n.to_string(),
            }),
        );
        put("vsync", Some(self.vsync.key().into()));
        put("draw_distance", Some(self.draw_distance.to_string()));
        put(
            "reflections",
            Some(match self.reflections {
                0 => "original".to_string(),
                n => n.to_string(),
            }),
        );
        put("anisotropy", Some(self.anisotropy.to_string()));
        put("sky_glow", Some(self.sky_glow.to_string()));
        put("xbox_brightness", Some(self.xbox_brightness.to_string()));
        put("hd_characters", Some(self.hd_characters.to_string()));
        put(
            "hd_textures",
            Some(if self.hd_textures { "on" } else { "off" }.into()),
        );
        put("hd_textures_folder", path(&self.hd_folder));
        put("deadzone", Some(self.deadzone.to_string()));
        put("vibration", Some(self.vibration.key().into()));
        put(
            "vibration_strength",
            Some(self.vibration_strength.to_string()),
        );
        put("impulse_triggers", Some(self.impulse_triggers.key().into()));
        // The platform's default API is left unset, not pinned.
        put(
            "input_backend",
            (self.input_backend != Backend::default_on(self.wine))
                .then(|| self.input_backend.key().into()),
        );
        put("manual_gearbox", Some(self.manual_gearbox.to_string()));
        put("cd_switcher", Some(self.cd_switcher.to_string()));
        put("music_shuffle", Some(self.music_shuffle.to_string()));
        put("skip_logos", Some(self.skip_logos.to_string()));
        put("free_drive", Some(self.free_drive.to_string()));
        put("horn", Some(self.horn.to_string()));
        put("random_anims", Some(self.random_anims.to_string()));
        put("show_miles", Some(self.show_miles.to_string()));
        put("soundtrack_fix", Some(self.soundtrack_fix.to_string()));
        put("scaler", Some(self.scaler.key().into()));
        t
    }
}

impl Vsync {
    pub fn key(self) -> &'static str {
        match self {
            Vsync::On => "on",
            Vsync::Off => "off",
        }
    }
}

/// The look to recommend on a screen: Steam Deck on a Deck, else Coast to
/// Coast.
pub fn recommended(screen: Option<(u32, u32)>) -> Look {
    if screen.and_then(landscape) == Some(STEAM_DECK_SCREEN) {
        Look::SteamDeck
    } else {
        Look::CoastToCoast
    }
}

/// What a gamepad does in the settings window, from the XInput buttons
/// pressed since the last look: Start plays, the d-pad moves between
/// controls, A presses the one in focus, LB and RB change tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadAction {
    Play,
    Next,
    Previous,
    Press,
    NextTab,
    PreviousTab,
}

pub const PAD_UP: u16 = 0x0001;
pub const PAD_DOWN: u16 = 0x0002;
pub const PAD_LEFT: u16 = 0x0004;
pub const PAD_RIGHT: u16 = 0x0008;
pub const PAD_START: u16 = 0x0010;
pub const PAD_LB: u16 = 0x0100;
pub const PAD_RB: u16 = 0x0200;
pub const PAD_A: u16 = 0x1000;

pub fn pad_actions(before: u16, now: u16) -> Vec<PadAction> {
    let pressed = now & !before;
    [
        (PAD_START, PadAction::Play),
        (PAD_DOWN, PadAction::Next),
        (PAD_RIGHT, PadAction::Next),
        (PAD_UP, PadAction::Previous),
        (PAD_LEFT, PadAction::Previous),
        (PAD_A, PadAction::Press),
        (PAD_RB, PadAction::NextTab),
        (PAD_LB, PadAction::PreviousTab),
    ]
    .into_iter()
    .filter(|(bit, _)| pressed & bit != 0)
    .map(|(_, action)| action)
    .collect()
}

/// The settings window's design size in points.
pub const WINDOW: (f32, f32) = (880.0, 760.0);

/// How much larger to draw the window on a screen: 1x up to 1440 pixels on
/// the shorter side, then in step with it (1.5x at 4K); never below the
/// system's scale and never larger than 90 % of the screen, in steps of
/// 0.05.
pub fn window_zoom(screen: Option<(u32, u32)>, native: f32) -> f32 {
    let native = if native > 0.0 { native } else { 1.0 };
    let Some((w, h)) = screen else {
        return 1.0;
    };
    let (w, h) = (w as f32, h as f32);
    let fit = 0.9 * (w / WINDOW.0).min(h / WINDOW.1);
    let target = (w.min(h) / 1440.0).max(1.0).min(fit);
    let zoom = (target / native).max(1.0);
    (zoom * 20.0).round() / 20.0
}

/// The zoom the settings window is created with. Under Wine it is the
/// final one (XWayland can lose a resize sent after the window exists),
/// with Wine's own DPI scale (`native`, 1.5 at 144 DPI) divided out, as
/// Wine multiplies the window by it. On Windows the window starts at 1x and
/// the first frame fits it to the DPI-aware scale.
pub fn creation_zoom(wine: bool, screen: Option<(u32, u32)>, native: f32) -> f32 {
    if wine {
        window_zoom(screen, native)
    } else {
        1.0
    }
}

/// Resolutions offered: the game's own, the screen's (when not already
/// listed), then common sizes.
pub fn resolutions(screen: Option<(u32, u32)>) -> Vec<(Option<(u32, u32)>, String)> {
    let common = [(1920, 1080), (2560, 1440), (3840, 2160)];
    let mut out = vec![(None, "640×480".to_string())];
    if let Some(s) = screen.and_then(landscape)
        && !common.contains(&s)
    {
        out.push((Some(s), format!("{}×{}", s.0, s.1)));
    }
    for (w, h) in common {
        let label = match h {
            1080 => "1080p",
            1440 => "1440p",
            _ => "4K",
        };
        out.push((Some((w, h)), label.to_string()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Option<(u32, u32)> = Some((2560, 1440));

    fn form(text: &str) -> Form {
        Form::new(&Settings::parse_file(text).0, SCREEN, false)
    }

    #[test]
    fn the_look_is_worked_out_from_the_saved_values() {
        let mut f = form(config::TEMPLATE);
        assert_eq!(
            f.look,
            Look::Custom,
            "the bare template is custom (no values)"
        );
        f.set_look(Look::CoastToCoast, SCREEN, false);
        let saved = f.save(config::TEMPLATE);
        assert_eq!(form(&saved).look, Look::CoastToCoast);
        // One value off the preset: custom.
        let tweaked = config::set_key(&saved, "draw_distance", "8");
        assert_eq!(form(&tweaked).look, Look::Custom);
        // And back.
        let back = config::set_key(&tweaked, "draw_distance", "4");
        assert_eq!(form(&back).look, Look::CoastToCoast);
    }

    #[test]
    fn only_a_missing_or_empty_file_is_a_first_run() {
        let first = Form::first_run(SCREEN, false);
        assert_eq!(Form::from_file(None, SCREEN, false), first);
        assert_eq!(Form::from_file(Some(""), SCREEN, false), first);
        assert_eq!(Form::from_file(Some(" \r\n"), SCREEN, false), first);
        assert_eq!(first.look, Look::CoastToCoast);
        // A saved file that is the template word for word is the player's.
        let kept = Form::from_file(Some(config::TEMPLATE), SCREEN, false);
        assert_eq!(kept, form(config::TEMPLATE));
        assert_ne!(kept.look, Look::CoastToCoast);
        // A file with just the game folder (written by --game) too.
        let with_game = config::set_key(config::TEMPLATE, "game", r"D:\OutRun");
        assert_eq!(
            Form::from_file(Some(&with_game), SCREEN, false),
            form(&with_game)
        );
    }

    #[test]
    fn presets_round_trip_and_original_means_stock() {
        let mut f = Form::first_run(SCREEN, false);
        assert_eq!(f.look, Look::CoastToCoast);
        assert_eq!(f.resolution, SCREEN);
        assert_eq!(form(&f.save(config::TEMPLATE)).look, Look::CoastToCoast);

        f.set_look(Look::Original, SCREEN, false);
        assert_eq!(f.resolution, None);
        assert_eq!(f.draw_distance, 0);
        assert!(!f.sky_glow && !f.cd_switcher && !f.free_drive);
        assert_eq!(f.vibration, Feedback::Off);
        assert_eq!(form(&f.save(config::TEMPLATE)).look, Look::Original);

        // Original is borderless everywhere.
        assert!(!Form::original(true).fullscreen);
        assert!(!Form::original(false).fullscreen);
    }

    #[test]
    fn the_steam_deck_look_and_its_recommendation() {
        let mut f = form(config::TEMPLATE);
        f.set_look(Look::SteamDeck, SCREEN, false);
        assert_eq!(f.resolution, Some((1280, 800)));
        assert_eq!(form(&f.save(config::TEMPLATE)).look, Look::SteamDeck);
        assert_eq!(recommended(Some((1280, 800))), Look::SteamDeck);
        assert_eq!(recommended(Some((800, 1280))), Look::SteamDeck);
        assert_eq!(recommended(SCREEN), Look::CoastToCoast);
        assert_eq!(recommended(None), Look::CoastToCoast);
        assert_eq!(
            Form::first_run(Some((800, 1280)), false).look,
            Look::SteamDeck
        );
    }

    #[test]
    fn saving_keeps_comments_and_round_trips_every_field() {
        let mut f = form(config::TEMPLATE);
        f.game = r"D:\Games\OutRun 2006".into();
        f.saves = r"Z:\home\me\saves".into();
        f.resolution = Some((1920, 1080));
        f.fullscreen = true;
        f.antialiasing = 2;
        f.fps_limit = 0;
        f.vsync = Vsync::Off;
        f.draw_distance = 8;
        f.reflections = 1024;
        f.anisotropy = 8;
        f.sky_glow = false;
        f.xbox_brightness = true;
        f.hd_characters = false;
        f.hd_textures = true;
        f.deadzone = 10;
        f.vibration = Feedback::Merged;
        f.vibration_strength = 5;
        f.impulse_triggers = Feedback::On;
        f.input_backend = Backend::XInput;
        f.manual_gearbox = true;
        f.cd_switcher = false;
        f.music_shuffle = true;
        f.skip_logos = false;
        f.free_drive = false;
        f.horn = false;
        f.random_anims = false;
        f.show_miles = false;
        f.soundtrack_fix = false;
        f.scaler = Scaler::Fsr;
        let saved = f.save(config::TEMPLATE);
        assert!(saved.contains("# OutRun 2006 launcher settings."));
        let mut back = form(&saved);
        back.look = f.look;
        assert_eq!(back, f);
        let (s, warnings) = Settings::parse_file(&saved);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.fps_cap(), 0);
        assert_eq!(s.vsync, Some(Vsync::Off));
        assert_eq!(s.feedback(), Feedback::Merged);
        assert_eq!(s.impulse(), Feedback::On);
        assert_eq!(s.backend(), Backend::XInput);
        assert_eq!(s.reflections, Some(1024));
        assert_eq!(s.deadzone_percent(), 10);
    }

    #[test]
    fn hd_textures_keep_their_folder() {
        let mut f = form(config::TEMPLATE);
        f.hd_textures = true;
        f.hd_folder = r"D:\pack".into();
        let saved = f.save(config::TEMPLATE);
        assert!(saved.contains("hd_textures = on"));
        assert!(saved.contains("hd_textures_folder = D:\\pack"));
        assert_eq!(form(&saved).hd_folder, r"D:\pack");

        // Switching the pack off keeps where it is.
        f.hd_textures = false;
        let off = f.save(&saved);
        assert!(off.contains("hd_textures = off"));
        assert_eq!(form(&off).hd_folder, r"D:\pack");

        // A look keeps the folder too.
        f.set_look(Look::Original, SCREEN, false);
        assert_eq!(f.hd_folder, r"D:\pack");

        f.hd_folder.clear();
        assert!(!f.save(&off).contains("hd_textures_folder = D"));
    }

    #[test]
    fn the_window_grows_with_the_screen() {
        assert_eq!(window_zoom(Some((1920, 1080)), 1.0), 1.0);
        assert_eq!(window_zoom(Some((3840, 2160)), 1.0), 1.5);
        assert_eq!(window_zoom(Some((3840, 2160)), 1.5), 1.0);
        assert_eq!(window_zoom(Some((1280, 800)), 1.0), 1.0);
        assert_eq!(window_zoom(None, 1.0), 1.0);
    }

    #[test]
    fn under_wine_the_window_is_created_at_its_final_size() {
        // XWayland can lose a later resize: Wine starts zoomed.
        assert_eq!(creation_zoom(true, Some((3840, 2160)), 1.0), 1.5);
        assert_eq!(creation_zoom(true, Some((1920, 1080)), 1.0), 1.0);
        // Wine at 144 or 192 DPI already scales the window: never zoomed
        // on top of that (it was 2.25x and 3x, taller than a 4K screen).
        assert_eq!(creation_zoom(true, Some((3840, 2160)), 1.5), 1.0);
        assert_eq!(creation_zoom(true, Some((3840, 2160)), 2.0), 1.0);
        for native in [1.0, 1.25, 1.5, 2.0] {
            let zoom = creation_zoom(true, Some((3840, 2160)), native);
            assert!(WINDOW.1 * zoom * native <= 2160.0, "{native}");
        }
        // Windows starts at 1x; the first frame fits the DPI-aware scale.
        assert_eq!(creation_zoom(false, Some((3840, 2160)), 1.5), 1.0);
    }

    #[test]
    fn gamepad_buttons_act_once_per_press() {
        assert_eq!(pad_actions(0, PAD_START), [PadAction::Play]);
        assert_eq!(pad_actions(PAD_START, PAD_START), []);
        assert_eq!(
            pad_actions(0, PAD_DOWN | PAD_A),
            [PadAction::Next, PadAction::Press]
        );
        assert_eq!(pad_actions(0, PAD_RB), [PadAction::NextTab]);
    }

    #[test]
    fn every_scaler_round_trips() {
        for (scaler, label, note) in SCALERS {
            assert!(!label.is_empty() && !note.is_empty());
            let mut f = form(config::TEMPLATE);
            f.scaler = scaler;
            let saved = f.save(config::TEMPLATE);
            assert!(saved.contains(&format!("scaler = {}", scaler.key())));
            assert_eq!(Settings::parse_file(&saved).0.scaler, Some(scaler));
        }

        // The scaler is not part of a look.
        for look in [Look::Original, Look::CoastToCoast, Look::SteamDeck] {
            let mut f = form(config::TEMPLATE);
            f.set_look(look, SCREEN, false);
            f.scaler = Scaler::Nearest;
            assert_eq!(Form::look_of(&f, SCREEN, false), look);
            assert_eq!(form(&f.save(config::TEMPLATE)).look, look);
        }
    }

    #[test]
    fn a_look_keeps_the_hd_pack_switch() {
        let mut f = form(config::TEMPLATE);
        f.hd_textures = true;
        f.set_look(Look::CoastToCoast, SCREEN, false);
        assert!(f.hd_textures);
        assert_eq!(Form::look_of(&f, SCREEN, false), Look::CoastToCoast);
        f.set_look(Look::Original, SCREEN, false);
        assert!(f.hd_textures);
        assert_eq!(Form::look_of(&f, SCREEN, false), Look::Original);
    }

    fn input_line(saved: &str) -> Option<&str> {
        saved
            .lines()
            .find(|l| l.trim_start().starts_with("input_backend ="))
    }

    #[test]
    fn the_platform_default_input_api_is_not_pinned() {
        let wine = |text: &str| Form::new(&Settings::parse_file(text).0, SCREEN, true);
        let f = wine(config::TEMPLATE);
        assert_eq!(f.input_backend, Backend::XInput);
        assert_eq!(input_line(&f.save(config::TEMPLATE)), None);

        let mut f = wine(config::TEMPLATE);
        f.input_backend = Backend::DirectInput;
        let saved = f.save(config::TEMPLATE);
        assert_eq!(input_line(&saved), Some("input_backend = dinput"));
        assert_eq!(wine(&saved).input_backend, Backend::DirectInput);

        // Off Wine the default is Windows.Gaming.Input.
        let mut f = form(config::TEMPLATE);
        assert_eq!(f.input_backend, Backend::WindowsGaming);
        assert_eq!(input_line(&f.save(config::TEMPLATE)), None);
        f.input_backend = Backend::XInput;
        assert_eq!(
            input_line(&f.save(config::TEMPLATE)),
            Some("input_backend = xinput")
        );
    }

    #[test]
    fn the_original_resolution_is_the_640x480_chip() {
        for text in ["resolution = original\n", "resolution = 640x480\n"] {
            assert_eq!(form(text).resolution, None);
            let s = Settings::parse_file(&format!(
                "{text}draw_distance = 0\nreflections = original\nanisotropy = 0\nsky_glow = false\nhd_characters = false\nvibration = off\nimpulse_triggers = off\ncd_switcher = false\nskip_logos = false\nfree_drive = false\nhorn = false\nrandom_anims = false\nshow_miles = false\nsoundtrack_fix = false\n"
            ))
            .0;
            assert_eq!(Form::new(&s, SCREEN, false).look, Look::Original);
        }
    }
}
