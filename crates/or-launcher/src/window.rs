//! The settings window: one window for Windows and for Linux (shown
//! through Wine by `or-play`). What it edits and saves is `form::Form`;
//! this module draws it. Everything Windows-specific (the folder picker,
//! the pad, rumble, the download's HTTPS) comes through [`Platform`], so
//! the window runs headless in tests on any host.
//!
//! Look: the car's colours. A black base with warm charcoal panels, rosso
//! red for actions (Play, the tab underline), Modena yellow for whatever is
//! chosen, a red and a yellow stripe in the corner, and a condensed italic
//! title like the arcade cabinet's. Five tabs keep it short: Play (the game
//! folder and one choice of look), Picture, HD Pack, Controls and Extras.
//! Every row's label and note come from the option table
//! (`config::OPTIONS`), the same words as `--help` and `launcher.cfg`.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use rea_launcher::download::{self, Job, Phase, Pin};
use rea_launcher::fsx;
use rea_launcher_ui::download::{Download, DownloadAction, DownloadState, download_row};
use rea_launcher_ui::eframe;
use rea_launcher_ui::egui::{
    self, Color32, Key, Mesh, Rect, Shape, TextureHandle, Vec2, pos2, vec2,
};
use rea_launcher_ui::pad::{self, Nav, PadNav};
use rea_launcher_ui::spec::{RowOptions, mark_rendered, spec_row, spec_row_with};
use rea_launcher_ui::theme::{
    ButtonText, CardMetrics, Metrics, Palette, body as font, condensed, decal, tint,
};
use rea_launcher_ui::widgets::{self, Chip, Crop, LookCard};
use rea_launcher_ui::window::{Layout, WindowFit};

use crate::config::{self, Backend, Feedback, Vsync, option};
use crate::form::{self, Form, Look};
use crate::{edition, hdpack, tweaks};

// Rosso, Modena yellow and black: the colours of OutRun's car.
const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Red is the action colour (Play, primary buttons, the tab underline),
/// never text on the dark base (3.5:1; white on it is 5.5:1). Yellow is
/// whatever is chosen. Errors are light coral text, so red and error never
/// meet.
pub const PALETTE: Palette = Palette {
    base: rgb(0x0f0c0b),
    panel: rgb(0x211b18),
    // Control borders: 3:1 against the base and the panels (WCAG non-text).
    line: rgb(0x6d6663),
    ink: rgb(0xf6f1ea),
    muted: rgb(0xab9f96),
    action: rgb(0xd40000),
    action_hover: rgb(0xe8160e),
    on_action: Color32::WHITE,
    select: rgb(0xffd400),
    on_select: rgb(0x1a1300),
    error: rgb(0xff9580),
    disabled_ink: rgb(0x5a504a),
    disabled_line: rgb(0x2a2320),
    focus: rgb(0xf6f1ea),
    // Selected text keeps OutRun's red highlight.
    selection: tint(rgb(0xd40000), 153),
};

/// OutRun's widget sizes: roomier look cards with a larger note, ink text
/// on outline buttons (the red is too dark to be text), one-line toggles.
pub const METRICS: Metrics = Metrics {
    button_text: ButtonText::Ink,
    toggle_wrap: false,
    card: CardMetrics {
        pad: 14.0,
        note: 13.5,
        note_top: 32.0,
        ..Metrics::DEFAULT.card
    },
    ..Metrics::DEFAULT
};

const BASE: Color32 = PALETTE.base;
const RED: Color32 = PALETTE.action;
const YELLOW: Color32 = PALETTE.select;
const INK: Color32 = PALETTE.ink;
const MUTED: Color32 = PALETTE.muted;
const ERROR: Color32 = PALETTE.error;

/// The window's design size in points (taller than the SDK's usual page).
pub const SIZE: Vec2 = vec2(880.0, 760.0);
/// The window's title (a second launcher finds the window by it).
pub const TITLE: &str = "OutRun 2006";
const PHOTO_W: f32 = 300.0;

/// How the window was left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Play pressed: the settings are saved.
    Play,
    /// Closed without playing.
    Closed,
}

/// What the window needs from the system it runs on.
pub trait Platform {
    /// The folder picker; `None` when cancelled.
    fn pick_folder(&self, title: &str) -> Option<PathBuf>;
    fn open_url(&self, url: &str);
    /// Rumbles every connected pad at `percent` for half a second; how
    /// many pads took it.
    fn test_rumble(&self, percent: u32) -> usize;
    /// Xbox-style pads connected.
    fn pads(&self) -> usize;
    /// The first pad's XInput button mask (0 without one).
    fn buttons(&self) -> u16;
    /// Starts `download::install(pin, root, …, job)` on a worker thread;
    /// `done` runs after it.
    fn download(&self, pin: Pin, root: PathBuf, job: Arc<Job>, done: Box<dyn FnOnce() + Send>);
}

/// Where the window is opened and on what.
pub struct Setup {
    /// `launcher.cfg`, written when Play is pressed.
    pub path: PathBuf,
    /// Its text, `None` when there is none yet (a first run, saved over
    /// the template).
    pub saved: Option<String>,
    /// The launcher's home folder, where the HD pack is downloaded.
    pub home: PathBuf,
    /// The folder of OutRunLauncher.exe (its `tweaks/` holds the bundled
    /// replacement EXE).
    pub launcher_dir: Option<PathBuf>,
    /// Shown for `or-play`: adds the gamescope scaler.
    pub linux: bool,
    /// Under Wine or Proton.
    pub wine: bool,
    /// The screen in pixels.
    pub screen: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Play,
    Picture,
    HdPack,
    Controls,
    Extras,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Play,
        Tab::Picture,
        Tab::HdPack,
        Tab::Controls,
        Tab::Extras,
    ];

    /// The tab `by` places along (wrapping), for LB and RB.
    fn step(self, by: isize) -> Tab {
        let at = Tab::ALL.iter().position(|&t| t == self).unwrap_or(0) as isize;
        Tab::ALL[(at + by).rem_euclid(Tab::ALL.len() as isize) as usize]
    }
}

const TABS: [(Tab, &str); 5] = [
    (Tab::Play, "Play"),
    (Tab::Picture, "Picture"),
    (Tab::HdPack, "HD Pack"),
    (Tab::Controls, "Controls"),
    (Tab::Extras, "Extras"),
];

/// Where to report an EXE the launcher doesn't know (the issue template
/// asks for the SHA-256 and `launcher.log`).
pub const REPORT_URL: &str =
    "https://github.com/akitaonrails/rea-outrun-2006/issues/new?template=unknown-edition.yml";
/// The bug report form, for problems the launcher can't fix itself.
pub const BUG_URL: &str =
    "https://github.com/akitaonrails/rea-outrun-2006/issues/new?template=bug-report.yml";

/// What the game folder holds, checked when its text changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    Empty,
    Found(&'static str),
    Problem(String),
}

/// Checks a game folder: its `OR2006C2C.exe` (any spelling) is a build
/// the launcher knows, or the replacement EXE in `bundled`.
pub fn check_game(folder: &str, bundled: Option<&[u8]>) -> Check {
    let folder = folder.trim();
    if folder.is_empty() {
        return Check::Empty;
    }
    let dir = Path::new(folder);
    if !dir.is_dir() {
        return Check::Problem(format!(
            "No such folder. Choose the folder that holds {}.",
            edition::EXE
        ));
    }
    let Some(exe) = rea_launcher::paths::find_ignoring_case(dir, edition::EXE) else {
        return Check::Problem(format!(
            "There is no {} in this folder. Choose the folder the game was installed in.",
            edition::EXE
        ));
    };
    match std::fs::read(&exe) {
        Ok(bytes) => match edition::identify_with_bundled(&bytes, bundled) {
            Ok(e) => Check::Found(e.name),
            Err(digest) => Check::Problem(format!(
                "This {} is a version the launcher doesn't know (SHA-256 {}). Please report it: {REPORT_URL}",
                edition::EXE,
                edition::to_hex(&digest)
            )),
        },
        Err(e) => Check::Problem(format!("Can't read {}: {e}", edition::EXE)),
    }
}

/// Download speed, smoothed over the last seconds.
#[derive(Default)]
struct Rate {
    sample: Option<(std::time::Instant, u64)>,
    bytes_per_second: f64,
}

impl Rate {
    fn update(&mut self, done: u64) -> f64 {
        let now = std::time::Instant::now();
        match self.sample {
            Some((at, before)) if now.duration_since(at).as_secs_f64() >= 1.0 => {
                let rate =
                    done.saturating_sub(before) as f64 / now.duration_since(at).as_secs_f64();
                self.bytes_per_second = if self.bytes_per_second == 0.0 {
                    rate
                } else {
                    0.7 * self.bytes_per_second + 0.3 * rate
                };
                self.sample = Some((now, done));
            }
            None => self.sample = Some((now, done)),
            _ => {}
        }
        self.bytes_per_second
    }
}

pub struct Window {
    pub form: Form,
    pub tab: Tab,
    check: Check,
    checked: String,
    screen: Option<(u32, u32)>,
    resolutions: Vec<(Option<(u32, u32)>, String)>,
    pads: usize,
    linux: bool,
    wine: bool,
    portrait: Option<TextureHandle>,
    portrait_tried: bool,
    path: PathBuf,
    text: String,
    error: Option<String>,
    bundled: Option<Vec<u8>>,
    launcher_dir: PathBuf,
    /// Sizes the window for this screen on the first frame.
    fit: Option<WindowFit>,
    /// What the last rumble test found.
    rumble_test: Option<String>,
    /// The gamepad, and the steps it asked of this frame.
    pad: PadNav,
    pending: Vec<Nav>,
    home: PathBuf,
    pin: Pin,
    /// The HD pack download (running, or failed), its speed, and how the
    /// last one ended.
    hd_job: Option<Arc<Job>>,
    hd_rate: Rate,
    hd_result: Option<(String, Color32)>,
    /// Remove was pressed once; the second press deletes.
    confirm_remove: bool,
    platform: Box<dyn Platform>,
    outcome: Rc<Cell<Outcome>>,
}

impl Window {
    pub fn new(setup: Setup, platform: Box<dyn Platform>, fit: Option<WindowFit>) -> Window {
        let Setup {
            path,
            saved,
            home,
            launcher_dir,
            linux,
            wine,
            screen,
        } = setup;
        let mut form = Form::from_file(saved.as_deref(), screen, wine);
        let text = saved.unwrap_or_else(config::template);
        // A pack downloaded earlier is offered even if its settings weren't
        // saved.
        if form.hd_folder.trim().is_empty()
            && let Some(dir) = hdpack::installed(&home)
        {
            form.hd_folder = dir.display().to_string();
        }
        let launcher_dir = launcher_dir.unwrap_or_default();
        let bundled = std::fs::read(launcher_dir.join(tweaks::TWEAKS_DIR).join(edition::EXE)).ok();
        let check = check_game(&form.game, bundled.as_deref());
        Window {
            checked: form.game.clone(),
            check,
            form,
            tab: Tab::Play,
            screen,
            resolutions: form::resolutions(screen),
            pads: platform.pads(),
            linux,
            wine,
            portrait: None,
            portrait_tried: false,
            path,
            text,
            error: None,
            bundled,
            launcher_dir,
            fit,
            rumble_test: None,
            pad: PadNav::default(),
            pending: Vec::new(),
            home,
            pin: hdpack::pin(),
            hd_job: None,
            hd_rate: Rate::default(),
            hd_result: None,
            confirm_remove: false,
            platform,
            outcome: Rc::new(Cell::new(Outcome::Closed)),
        }
    }

    /// How the window was left, readable after eframe drops it.
    pub fn outcome(&self) -> Rc<Cell<Outcome>> {
        self.outcome.clone()
    }

    /// The game folder's check, as the Play tab shows it.
    pub fn check(&self) -> &Check {
        &self.check
    }

    pub fn ready(&self) -> bool {
        matches!(self.check, Check::Found(_)) && !self.hd_running()
    }

    /// Saves and closes, when the game folder is good and no download runs.
    pub fn play(&mut self, ctx: &egui::Context) {
        if !self.ready() {
            self.tab = Tab::Play;
            return;
        }
        let saved = self.form.save(&self.text);
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match fsx::write_atomic(&self.path, saved) {
            Ok(()) => {
                self.outcome.set(Outcome::Play);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.error = Some(format!("Can't save the settings: {e}")),
        }
    }

    /// The whole window in `ui`'s rect: art, title, tabs, the current tab
    /// and the footer.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if !self.portrait_tried {
            self.portrait_tried = true;
            self.portrait =
                widgets::load_jpeg(&ctx, "portrait", include_bytes!("../assets/portrait.jpg"));
        }
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, BASE);
        self.draw_photo(ui, full);
        widgets::stripes(
            ui.painter(),
            full,
            &[(30.0, RED), (16.0, YELLOW.gamma_multiply(0.9))],
        );

        let layout = Layout::new(full, PHOTO_W);
        ui.scope_builder(egui::UiBuilder::new().max_rect(layout.body), |ui| {
            ui.label(
                egui::RichText::new("OutRun 2006")
                    .font(decal(46.0))
                    .color(INK),
            );
            ui.add_space(-6.0);
            ui.label(
                egui::RichText::new("COAST 2 COAST")
                    .font(condensed(17.0))
                    .color(YELLOW),
            );
            ui.add_space(4.0);
            self.status(ui);
            ui.add_space(4.0);
            widgets::tabs_bar(ui, &TABS, &mut self.tab);
            ui.add_space(14.0);
            // Long tabs scroll inside the body instead of running under
            // the footer.
            widgets::tab_body(ui, self.tab, |ui| self.tab_contents(ui, self.tab));
        });
        self.footer(ui, layout.footer, &ctx);
        self.poll_download();

        if ctx.input(|i| i.key_pressed(Key::Enter)) && !ctx.egui_wants_keyboard_input() {
            self.play(&ctx);
        }
        // Taken, so a second egui pass in one frame can't play twice.
        for nav in std::mem::take(&mut self.pending) {
            match nav {
                Nav::Play => self.play(&ctx),
                Nav::NextTab => self.tab = self.tab.step(1),
                Nav::PreviousTab => self.tab = self.tab.step(-1),
                _ => {}
            }
        }
        if self.form.game != self.checked {
            self.checked = self.form.game.clone();
            self.check = check_game(&self.form.game, self.bundled.as_deref());
        }
    }

    /// One tab's rows (the window draws the current one; tests draw each).
    pub fn tab_contents(&mut self, ui: &mut egui::Ui, tab: Tab) {
        let before = self.form.clone();
        match tab {
            Tab::Play => self.play_tab(ui),
            Tab::Picture => self.picture_tab(ui),
            Tab::HdPack => self.hd_tab(ui),
            Tab::Controls => self.controls_tab(ui),
            Tab::Extras => self.extras_tab(ui),
        }
        if tab != Tab::Play && self.form != before {
            self.form.look = Form::look_of(&self.form, self.screen, self.wine);
        }
    }

    fn draw_photo(&self, ui: &egui::Ui, full: Rect) {
        let rect = Rect::from_min_size(full.min, vec2(PHOTO_W, full.height()));
        let p = ui.painter();
        if self.portrait.is_none() {
            // Without the artwork: a black-to-red-to-yellow sky.
            let colors = [BASE, RED.gamma_multiply(0.45), RED, YELLOW];
            let n = colors.len() as f32;
            for (i, &color) in colors.iter().enumerate() {
                let y0 = rect.top() + rect.height() * i as f32 / n;
                let y1 = rect.top() + rect.height() * (i + 1) as f32 / n;
                let band = Rect::from_min_max(pos2(rect.left(), y0), pos2(rect.right(), y1));
                let mut m = Mesh::default();
                m.colored_vertex(band.left_top(), color);
                m.colored_vertex(band.right_top(), color);
                m.colored_vertex(band.right_bottom(), color);
                m.colored_vertex(band.left_bottom(), color);
                m.add_triangle(0, 1, 2);
                m.add_triangle(0, 2, 3);
                p.add(Shape::mesh(m));
            }
        }
        // The middle of the picture (the car), faded to black into the page.
        widgets::hero_image(p, rect, self.portrait.as_ref(), Crop::Center, BASE);
        // The caption sits on a solid plate: no small text straight on the
        // photo.
        let galley = p.layout(
            "Splash Wave or Magical Sound Shower — the girl's request is optional. Music can be changed mid-race."
                .to_string(),
            font(13.0),
            MUTED,
            rect.width() - 56.0,
        );
        let plate = Rect::from_min_size(
            pos2(rect.left() + 16.0, rect.bottom() - 28.0 - galley.size().y),
            galley.size() + vec2(24.0, 16.0),
        );
        p.rect_filled(plate, 6.0, PALETTE.panel);
        p.galley(plate.min + vec2(12.0, 8.0), galley, MUTED);
    }

    fn status(&self, ui: &mut egui::Ui) {
        let (text, color) = match (&self.error, &self.check) {
            (Some(e), _) => (e.clone(), ERROR),
            (None, Check::Found(_)) if self.hd_running() => (
                "Downloading the HD pack; Play waits until it's done.".to_string(),
                MUTED,
            ),
            (None, Check::Found(_)) => ("Ready. Press Play or Enter.".to_string(), MUTED),
            (None, _) => (
                "First, show the launcher where the game is.".to_string(),
                MUTED,
            ),
        };
        ui.label(egui::RichText::new(text).font(font(13.5)).color(color));
    }

    fn play_tab(&mut self, ui: &mut egui::Ui) {
        widgets::heading(ui, option("game").label);
        mark_rendered(ui.ctx(), "game");
        let game_hint = if self.linux {
            format!(
                "The folder with your {}, e.g. Z:\\home\\you\\Games\\OutRun 2006",
                edition::EXE
            )
        } else {
            format!("The folder with your installed {}", edition::EXE)
        };
        let platform = &self.platform;
        if widgets::folder_field(
            ui,
            &mut self.form.game,
            option("game").label,
            &game_hint,
            || platform.pick_folder(&format!("Choose the folder with your {}", edition::EXE)),
        ) {
            self.error = None;
        }
        let (note, color) = match &self.check {
            Check::Empty => (
                "The game folder is only read, never changed.".to_string(),
                MUTED,
            ),
            Check::Found(name) => (format!("Found: {name}"), YELLOW),
            Check::Problem(p) => (p.clone(), ERROR),
        };
        ui.label(egui::RichText::new(note).font(font(13.0)).color(color));
        if matches!(&self.check, Check::Problem(p) if p.contains(REPORT_URL))
            && widgets::outline_button(ui, "Report this version").clicked()
        {
            self.platform.open_url(REPORT_URL);
        }
        ui.add_space(10.0);

        widgets::heading(ui, option("look").label);
        mark_rendered(ui.ctx(), "look");
        ui.add_space(6.0);
        let note = match Form::coast_to_coast(self.screen, self.wine).resolution {
            Some((w, h)) => format!("Your screen's {w}×{h}, the tuned set."),
            None => "Smooth, further view, glowing skies.".to_string(),
        };
        let cards = [
            (
                Look::Original,
                "Original 2006",
                "640×480, as it shipped.".to_string(),
            ),
            (Look::CoastToCoast, "Coast to Coast", note),
            (
                Look::SteamDeck,
                "Steam Deck",
                "1280×800 at 60 fps, easy on the battery.".to_string(),
            ),
            (
                Look::Custom,
                "Custom",
                "Pick each setting in the tabs.".to_string(),
            ),
        ];
        let best = form::recommended(self.screen);
        let gap = 10.0;
        let w = (ui.available_width() - 3.0 * gap) / 4.0;
        // One title size for all four cards: the largest at which every
        // title fits.
        let titles: Vec<&str> = cards.iter().map(|(_, title, _)| *title).collect();
        let title_size = widgets::fit_title(ui, &titles, w - 2.0 * METRICS.card.pad);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (look, title, note) in cards {
                let card = LookCard {
                    title,
                    note: &note,
                    detail: None,
                    selected: self.form.look == look,
                    badge: (look == best).then_some("Recommended"),
                    title_size,
                    disabled: None,
                };
                if widgets::look_card(ui, vec2(w, 104.0), &card).clicked() {
                    self.form.set_look(look, self.screen, self.wine);
                    if look == Look::Custom {
                        self.tab = Tab::Picture;
                    }
                }
            }
        });
        ui.add_space(14.0);

        widgets::heading(ui, option("saves").label);
        mark_rendered(ui.ctx(), "saves");
        let platform = &self.platform;
        widgets::folder_field(
            ui,
            &mut self.form.saves,
            option("saves").label,
            "Saved Games\\OutRun 2006 (the default)",
            || platform.pick_folder("Choose where your saves are kept"),
        );
        widgets::note(
            ui,
            "Leave empty for the default. Your License.dat and records are copied in before the game starts and back when it ends.",
        );
    }

    fn picture_tab(&mut self, ui: &mut egui::Ui) {
        let f = &mut self.form;
        let mut resolutions = self.resolutions.clone();
        if let Some(size) = f.resolution
            && !resolutions.iter().any(|(r, _)| *r == Some(size))
        {
            // A size from the file or the command line stays shown.
            resolutions.push((Some(size), format!("{}×{}", size.0, size.1)));
        }
        widgets::heading(ui, "Screen");
        chip_row(ui, "resolution", &resolutions, &mut f.resolution, None);
        bool_row(ui, "fullscreen", &mut f.fullscreen, None);
        let rates = with_current(&FRAME_RATES, f.fps_limit, |n| n.to_string());
        step_row(ui, "fps_limit", &rates, &mut f.fps_limit);
        let mut vsync = f.vsync == Vsync::On;
        bool_row(ui, "vsync", &mut vsync, None);
        f.vsync = if vsync { Vsync::On } else { Vsync::Off };
        group(ui, "Detail");
        let distances = with_current(&DISTANCES, f.draw_distance, |n| n.to_string());
        step_row(ui, "draw_distance", &distances, &mut f.draw_distance);
        let mut reflections = match f.reflections {
            0 => "original".to_string(),
            n => n.to_string(),
        };
        if spec_row(ui, option("reflections"), &mut reflections).changed() {
            f.reflections = reflections.parse().unwrap_or(0);
        }
        let anisotropy = with_current(&ANISOTROPY, f.anisotropy, |n| format!("{n}×"));
        step_row(ui, "anisotropy", &anisotropy, &mut f.anisotropy);
        let mut aa = f.antialiasing.to_string();
        if spec_row(ui, option("antialiasing"), &mut aa).changed() {
            f.antialiasing = aa.parse().unwrap_or(0);
        }
        group(ui, "Effects");
        bool_row(ui, "sky_glow", &mut f.sky_glow, None);
        bool_row(ui, "xbox_brightness", &mut f.xbox_brightness, None);
        bool_row(ui, "hd_characters", &mut f.hd_characters, None);
        if self.linux {
            group(ui, "Linux");
            let mut scaler = f.scaler.key().to_string();
            if spec_row(ui, option("scaler"), &mut scaler).changed()
                && let Some(s) = config::Scaler::ALL.into_iter().find(|s| s.key() == scaler)
            {
                f.scaler = s;
            }
        }
    }

    fn hd_tab(&mut self, ui: &mut egui::Ui) {
        widgets::heading(ui, "HD texture pack");
        widgets::note(
            ui,
            "The community's redrawn menus, HUD, sprites and button prompts at high resolution \
             (Envido32's compilation of the Sonic-TV OR2006Sprites project). An optional download \
             from its own GitHub release, never bundled with the launcher.",
        );
        if ui
            .link(
                egui::RichText::new("The pack's page and credits")
                    .font(font(13.0))
                    .color(YELLOW),
            )
            .clicked()
        {
            self.platform.open_url(config::HD_PACK_PAGE);
        }
        ui.add_space(12.0);

        let root = hdpack::root(&self.home);
        let downloaded = download::installed(&self.pin, &root);
        // Without a folder the game is pointed at textures-pack next to the
        // launcher (config::Settings::hd_textures_source).
        let folder = match self.form.hd_folder.trim() {
            "" => self.launcher_dir.join(config::TEXTURES_PACK),
            chosen => PathBuf::from(chosen),
        };
        let has_pack = tweaks::texture_base(&folder).is_some();
        let running = self.hd_running();
        let own_copy = has_pack && downloaded.as_deref() != Some(folder.as_path()) && !running;
        let off = if running {
            Some("Wait for the download to finish.")
        } else if !has_pack {
            Some("Download the pack, or choose the folder of a copy you have.")
        } else {
            None
        };
        bool_row(ui, "hd_textures", &mut self.form.hd_textures, off);
        ui.add_space(6.0);
        if own_copy {
            // A copy the player unzipped: nothing to download or remove.
            ui.label(
                egui::RichText::new("Your own copy of the pack")
                    .font(font(13.5))
                    .color(YELLOW),
            );
        } else {
            self.download_row(ui, &root);
        }
        if has_pack {
            widgets::note(
                ui,
                "With the pack on, Clarissa looks as in the Japanese release, which its sprites are drawn for.",
            );
        }
        if let Some((text, color)) = &self.hd_result {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(text).font(font(13.5)).color(*color));
        }

        ui.add_space(18.0);
        widgets::heading(ui, "Already have it?");
        mark_rendered(ui.ctx(), "hd_textures_folder");
        let before = self.form.hd_folder.clone();
        let platform = &self.platform;
        widgets::folder_field(
            ui,
            &mut self.form.hd_folder,
            option("hd_textures_folder").label,
            "The folder you unzipped the pack into",
            || platform.pick_folder("Choose the folder you unzipped the HD texture pack into"),
        );
        if self.form.hd_folder != before {
            self.hd_result = None;
        }
        let (note, color) = hd_pack_note(&self.form.hd_folder);
        ui.label(egui::RichText::new(note).font(font(13.0)).color(color));
    }

    fn hd_running(&self) -> bool {
        self.hd_job
            .as_ref()
            .is_some_and(|job| job.phase().running())
    }

    /// The SDK's download row for the pinned pack: Download (or resume),
    /// progress with Cancel, or Remove once it is in.
    fn download_row(&mut self, ui: &mut egui::Ui, root: &Path) {
        let phase = self.hd_job.as_ref().map(|job| job.phase());
        let rate = match &phase {
            Some(Phase::Downloading { done, .. }) => Some(self.hd_rate.update(*done)),
            _ => None,
        };
        let state = DownloadState::from_phase(
            phase.as_ref(),
            download::installed(&self.pin, root).is_some(),
            download::partial(&self.pin, root),
        )
        .with_rate(rate);
        let row = Download {
            label: hdpack::LABEL,
            size: self.pin.size,
            state,
        };
        match download_row(ui, &row) {
            Some(DownloadAction::Start) => self.start_download(ui.ctx()),
            Some(DownloadAction::Cancel) => {
                if let Some(job) = &self.hd_job {
                    job.cancel();
                }
            }
            Some(DownloadAction::Remove) if !self.confirm_remove => {
                self.confirm_remove = true;
                self.hd_result = Some((
                    format!(
                        "Press Remove again to delete the pack ({}). You can download it again later.",
                        rea_launcher_ui::download::size_text(self.pin.unpacked_size)
                    ),
                    YELLOW,
                ));
            }
            Some(DownloadAction::Remove) => match download::remove(&self.pin, root) {
                Ok(()) => {
                    self.confirm_remove = false;
                    self.hd_job = None;
                    self.form.hd_textures = false;
                    self.form.hd_folder.clear();
                    self.hd_result = Some((
                        format!(
                            "Removed; {} freed.",
                            rea_launcher_ui::download::size_text(self.pin.unpacked_size)
                        ),
                        MUTED,
                    ));
                }
                Err(e) => self.hd_result = Some((e.to_string(), ERROR)),
            },
            None => {}
        }
        if matches!(state, DownloadState::Missing { partial: 0 }) {
            widgets::note(
                ui,
                &format!(
                    "{} once unpacked. The launcher downloads, checks and unpacks it for you.",
                    rea_launcher_ui::download::size_text(self.pin.unpacked_size)
                ),
            );
        }
        if state.running() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn start_download(&mut self, ctx: &egui::Context) {
        let job = Arc::new(Job::default());
        let ctx = ctx.clone();
        self.platform.download(
            self.pin.clone(),
            hdpack::root(&self.home),
            job.clone(),
            Box::new(move || ctx.request_repaint()),
        );
        self.hd_job = Some(job);
        self.hd_rate = Rate::default();
        self.hd_result = None;
    }

    /// Picks up how a finished download ended: done switches the pack on;
    /// a failure stays in the row until the next attempt.
    fn poll_download(&mut self) {
        let Some(job) = &self.hd_job else {
            return;
        };
        match job.phase() {
            Phase::Done(dir) => {
                self.hd_job = None;
                self.form.hd_folder = dir.display().to_string();
                self.form.hd_textures = true;
                self.hd_result = Some(("Installed and switched on.".to_string(), YELLOW));
            }
            Phase::Cancelled => {
                self.hd_job = None;
                self.hd_result = Some((
                    "Stopped. Press Download to carry on from there later.".to_string(),
                    MUTED,
                ));
            }
            _ => {}
        }
    }

    fn controls_tab(&mut self, ui: &mut egui::Ui) {
        let f = &mut self.form;
        let deadzones = with_current(&DEADZONES, f.deadzone, |n| format!("{n}%"));
        step_row(ui, "deadzone", &deadzones, &mut f.deadzone);
        let mut vibration = f.vibration.key().to_string();
        if spec_row(ui, option("vibration"), &mut vibration).changed()
            && let Some(v) = Feedback::ALL.into_iter().find(|v| v.key() == vibration)
        {
            f.vibration = v;
        }
        let rumble_off = (f.vibration == Feedback::Off).then_some("Turn rumble on first.");
        let strengths = with_current(&STRENGTHS, f.vibration_strength, |n| n.to_string());
        chip_row(
            ui,
            "vibration_strength",
            &strengths,
            &mut f.vibration_strength,
            rumble_off,
        );
        if rumble_off.is_none() {
            ui.horizontal(|ui| {
                ui.add_space(METRICS.field_width + ui.spacing().item_spacing.x);
                if widgets::outline_button(ui, "Test rumble").clicked() {
                    let took = self.platform.test_rumble(f.vibration_strength * 10);
                    self.rumble_test = Some(rumble_result(took, self.wine).to_string());
                }
            });
            if let Some(result) = &self.rumble_test {
                ui.label(egui::RichText::new(result).font(font(13.0)).color(YELLOW));
            }
        }
        let impulse_off = if self.wine {
            Some("Impulse triggers work on Windows only.")
        } else {
            rumble_off
        };
        chip_row(
            ui,
            "impulse_triggers",
            &FEEDBACK,
            &mut f.impulse_triggers,
            impulse_off,
        );
        chip_row(ui, "input_backend", &BACKENDS, &mut f.input_backend, None);
        bool_row(ui, "manual_gearbox", &mut f.manual_gearbox, None);
        let pads = match self.pads {
            0 => "The Test button checks Xbox-style (XInput) pads; in the game, PlayStation and Switch pads rumble too.".to_string(),
            1 => "One Xbox-style pad connected.".to_string(),
            n => format!("{n} Xbox-style pads connected."),
        };
        widgets::note(ui, &pads);
    }

    fn extras_tab(&mut self, ui: &mut egui::Ui) {
        let f = &mut self.form;
        bool_row(ui, "cd_switcher", &mut f.cd_switcher, None);
        let no_switcher = (!f.cd_switcher).then_some("Needs the CD switcher.");
        bool_row(ui, "music_shuffle", &mut f.music_shuffle, no_switcher);
        bool_row(ui, "skip_logos", &mut f.skip_logos, None);
        bool_row(ui, "free_drive", &mut f.free_drive, None);
        bool_row(ui, "horn", &mut f.horn, None);
        bool_row(ui, "random_anims", &mut f.random_anims, None);
        bool_row(ui, "show_miles", &mut f.show_miles, None);
        bool_row(ui, "soundtrack_fix", &mut f.soundtrack_fix, None);
        widgets::note(
            ui,
            "Online play points at the community server (clarissa.port0.org); pick a user name and password in the game's multiplayer menu.",
        );
    }

    fn footer(&mut self, ui: &mut egui::Ui, rect: Rect, ctx: &egui::Context) {
        let hint = if self.form.resolution.is_some() {
            if self.form.fullscreen {
                "The game runs at this resolution in exclusive fullscreen."
            } else {
                "The game runs at this resolution as a borderless window."
            }
        } else {
            "Settings are saved when you press Play."
        };
        if widgets::footer(ui, rect, hint, self.ready()).clicked() {
            self.play(ctx);
        }
    }
}

impl eframe::App for Window {
    /// The gamepad, as keys (the SDK's pad navigation): the d-pad moves
    /// the focus and steps a chip row, A presses; Start and LB/RB are
    /// handled in `draw`.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let navs = self.pad.update(self.platform.buttons());
        pad::inject(ctx, raw, &navs);
        self.pending.extend(navs);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(fit) = &mut self.fit {
            fit.first_frame(ui.ctx());
        }
        self.draw(ui);
        // A pad sends no events: look again soon.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
    }
}

/// A heading that starts a new group of rows.
fn group(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    widgets::heading(ui, title);
}

/// The SDK's row for the `Range` option `key`, as chips with `steps`.
fn step_row(ui: &mut egui::Ui, key: &'static str, steps: &[(u32, String)], value: &mut u32) {
    let steps: Vec<(u32, &str)> = steps.iter().map(|(v, l)| (*v, l.as_str())).collect();
    let mut text = value.to_string();
    let options = RowOptions {
        steps: Some(&steps),
        ..RowOptions::default()
    };
    if spec_row_with(ui, option(key), &mut text, options).changed()
        && let Ok(n) = text.parse()
    {
        *value = n;
    }
    ui.add_space(6.0);
}

/// A row of chips for `key`: its label on the left, its description under
/// it (both also the row's accessible name and description). `disabled`
/// greys every chip out; the SDK's chip row writes why under the chips,
/// where a pad player sees it. Also for the rows the SDK has no chips
/// for: a resolution, and the input API without its "auto" value.
fn chip_row<T: PartialEq + Clone, S: AsRef<str>>(
    ui: &mut egui::Ui,
    key: &'static str,
    options: &[(T, S)],
    value: &mut T,
    disabled: Option<&str>,
) {
    let row = option(key);
    mark_rendered(ui.ctx(), key);
    ui.vertical(|ui| {
        let chips: Vec<Chip<'_, T>> = options
            .iter()
            .map(|(v, label)| Chip::new(v.clone(), label.as_ref()).disabled(disabled))
            .collect();
        let response = ui
            .horizontal_top(|ui| {
                widgets::field(ui, row.label);
                widgets::chips_with(ui, &chips, value)
            })
            .inner;
        widgets::name_control(&response, Some(row.label), Some(row.description));
        widgets::note(ui, row.description);
        ui.add_space(6.0);
    });
}

/// A switch for `key` with its description under it; `disabled` says
/// why it can't be changed now (the SDK's switch writes it under its
/// label).
fn bool_row(ui: &mut egui::Ui, key: &'static str, on: &mut bool, disabled: Option<&str>) {
    let row = option(key);
    mark_rendered(ui.ctx(), key);
    ui.vertical(|ui| {
        let response = widgets::toggle(ui, on, row.label, disabled);
        widgets::name_control(&response, Some(row.label), Some(row.description));
        widgets::note(ui, row.description);
        ui.add_space(6.0);
    });
}

/// `list` plus `current` when it is not among them (a value from the file
/// or the command line stays shown and kept).
fn with_current(
    list: &[(u32, &str)],
    current: u32,
    label: impl Fn(u32) -> String,
) -> Vec<(u32, String)> {
    let mut out: Vec<(u32, String)> = list.iter().map(|(v, l)| (*v, l.to_string())).collect();
    if !out.iter().any(|(v, _)| *v == current) {
        let at = out.iter().position(|(v, _)| *v > current && *v != 0);
        out.insert(at.unwrap_or(out.len()), (current, label(current)));
    }
    out
}

/// The choices offered, with their labels.
pub const FRAME_RATES: [(u32, &str); 5] = [
    (60, "60"),
    (90, "90"),
    (120, "120"),
    (144, "144"),
    (0, "Unlimited"),
];
pub const DISTANCES: [(u32, &str); 3] = [(0, "Game's own"), (4, "4"), (8, "8")];
pub const ANISOTROPY: [(u32, &str); 4] = [(0, "Game's own"), (4, "4×"), (8, "8×"), (16, "16×")];
pub const DEADZONES: [(u32, &str); 5] = [
    (0, "0%"),
    (10, "10%"),
    (20, "20%"),
    (30, "30%"),
    (40, "40%"),
];
pub const STRENGTHS: [(u32, &str); 4] =
    [(3, "Light"), (5, "Medium"), (7, "Normal"), (10, "Strong")];
pub const FEEDBACK: [(Feedback, &str); 4] = [
    (Feedback::Off, "Off"),
    (Feedback::On, "On"),
    (Feedback::Swapped, "Swapped"),
    (Feedback::Merged, "Merged"),
];
pub const BACKENDS: [(Backend, &str); 4] = [
    (Backend::WindowsGaming, "WGI"),
    (Backend::Raw, "RawInput"),
    (Backend::DirectInput, "DInput"),
    (Backend::XInput, "XInput"),
];

/// Whether the chosen HD pack folder holds textures, for the note under it.
fn hd_pack_note(folder: &str) -> (String, Color32) {
    let folder = folder.trim();
    if folder.is_empty() {
        return (
            "Unzip the pack anywhere and choose that folder.".to_string(),
            MUTED,
        );
    }
    match tweaks::texture_base(Path::new(folder)) {
        Some(_) => (
            "Pack found. It's read from there; nothing is copied.".to_string(),
            YELLOW,
        ),
        None => (
            "No textures\\load folder here: choose the folder the zip was unzipped into."
                .to_string(),
            ERROR,
        ),
    }
}

/// What "Test rumble" says after `took` pads took it. Wine is named only
/// under Wine (`wine`): on Windows an Xbox-style pad needs no mode.
fn rumble_result(took: usize, wine: bool) -> &'static str {
    match (took, wine) {
        (0, true) => {
            "No pad took the rumble. Under Wine the pad must be in X-input mode; on 8BitDo pads use the X-input switch."
        }
        (0, false) => {
            "No pad took the rumble. The test needs an Xbox-style (XInput) pad; on 8BitDo pads use the X-input switch."
        }
        _ => {
            "Sent: your pad should have shaken for half a second. In a race it shakes on shifts, drifts and crashes."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher::cfg::spec;
    use rea_launcher_ui::spec::{clear_rendered, rendered_keys};
    use rea_launcher_ui::testing::{self, Harness, Queryable};
    use rea_launcher_ui::theme;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A platform that records what the window asked of it.
    #[derive(Default)]
    struct Fake {
        rumbles: Arc<AtomicUsize>,
        downloads: Arc<AtomicUsize>,
        /// No pad takes the rumble test.
        no_pad: bool,
    }

    impl Platform for Fake {
        fn pick_folder(&self, _title: &str) -> Option<PathBuf> {
            None
        }
        fn open_url(&self, _url: &str) {}
        fn test_rumble(&self, _percent: u32) -> usize {
            self.rumbles.fetch_add(1, Ordering::Relaxed);
            usize::from(!self.no_pad)
        }
        fn pads(&self) -> usize {
            1
        }
        fn buttons(&self) -> u16 {
            0
        }
        fn download(
            &self,
            _pin: Pin,
            _root: PathBuf,
            _job: Arc<Job>,
            _done: Box<dyn FnOnce() + Send>,
        ) {
            self.downloads.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// A launcher folder whose bundled EXE is also the game's, so the
    /// game folder checks out (nothing from the real game).
    struct Dirs {
        root: PathBuf,
    }

    impl Dirs {
        fn new(name: &str) -> Dirs {
            // A hostile path: spaces, an apostrophe and accents.
            let root = std::env::temp_dir().join(format!(
                "or-window-{}-{name}/José/Jogos d'Ação",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            for dir in ["launcher/tweaks", "game", "home"] {
                std::fs::create_dir_all(root.join(dir)).unwrap();
            }
            std::fs::write(root.join("launcher/tweaks/OR2006C2C.exe"), b"fake exe").unwrap();
            std::fs::write(root.join("game/OR2006C2C.exe"), b"fake exe").unwrap();
            Dirs { root }
        }

        fn cfg(&self) -> PathBuf {
            self.root.join("home/launcher.cfg")
        }

        fn setup(&self, saved: Option<&str>, linux: bool, wine: bool) -> Setup {
            Setup {
                path: self.cfg(),
                saved: saved.map(str::to_string),
                home: self.root.join("home"),
                launcher_dir: Some(self.root.join("launcher")),
                linux,
                wine,
                screen: Some((2560, 1440)),
            }
        }
    }

    impl Drop for Dirs {
        fn drop(&mut self) {
            let top = self.root.ancestors().nth(2).unwrap().to_path_buf();
            let _ = std::fs::remove_dir_all(top);
        }
    }

    /// The window at its own width, tall enough to show every row of
    /// every tab (the long tabs scroll at 880×760).
    fn harness(window: Window) -> Harness<'static, Window> {
        testing::harness_sized(
            &PALETTE,
            &METRICS,
            vec2(SIZE.x, 1400.0),
            window,
            |ui, w: &mut Window| w.draw(ui),
        )
    }

    fn open(dirs: &Dirs, saved: Option<&str>, linux: bool, wine: bool) -> Harness<'static, Window> {
        let game = dirs.root.join("game").display().to_string();
        let text = saved.map_or_else(
            || config::set_key(&config::template(), "game", &game),
            str::to_string,
        );
        harness(Window::new(
            dirs.setup(Some(&text), linux, wine),
            Box::new(Fake::default()),
            None,
        ))
    }

    /// The Play plate (the first tab is called Play too).
    fn play<'a>(ui: &'a Harness<'static, Window>) -> testing::Node<'a> {
        ui.get_by_role_and_label(egui::accesskit::Role::Button, "Play")
    }

    fn show(ui: &mut Harness<'static, Window>, tab: Tab) {
        ui.state_mut().tab = tab;
        ui.run();
    }

    #[test]
    fn the_window_and_the_command_line_have_the_same_options() {
        let dirs = Dirs::new("parity");
        let mut ui = open(&dirs, None, true, false);
        clear_rendered(&ui.ctx);
        for tab in Tab::ALL {
            show(&mut ui, tab);
        }
        spec::assert_parity(config::OPTIONS, &rendered_keys(&ui.ctx));
    }

    #[test]
    fn every_option_is_labelled_and_described() {
        let dirs = Dirs::new("labels");
        let mut ui = open(&dirs, None, true, false);
        for (tab, keys) in [
            (
                Tab::Picture,
                &[
                    "resolution",
                    "fullscreen",
                    "fps_limit",
                    "vsync",
                    "draw_distance",
                    "reflections",
                    "anisotropy",
                    "antialiasing",
                    "sky_glow",
                    "xbox_brightness",
                    "hd_characters",
                    "scaler",
                ][..],
            ),
            (Tab::HdPack, &["hd_textures"][..]),
            (
                Tab::Controls,
                &[
                    "deadzone",
                    "vibration",
                    "vibration_strength",
                    "impulse_triggers",
                    "input_backend",
                    "manual_gearbox",
                ][..],
            ),
            (
                Tab::Extras,
                &[
                    "cd_switcher",
                    "music_shuffle",
                    "skip_logos",
                    "free_drive",
                    "horn",
                    "random_anims",
                    "show_miles",
                    "soundtrack_fix",
                ][..],
            ),
        ] {
            show(&mut ui, tab);
            for key in keys {
                let row = option(key);
                // The control (not its row's text label) carries both.
                assert!(
                    ui.get_all_by_label(row.label).any(|node| {
                        testing::description(&node).as_deref() == Some(row.description)
                    }),
                    "{key}: no control named {:?} described {:?}",
                    row.label,
                    row.description
                );
            }
        }
        show(&mut ui, Tab::Play);
        for look in ["Original 2006", "Coast to Coast", "Steam Deck", "Custom"] {
            ui.get_by_label(look);
        }
        // Each folder field is named by its heading; the hint is its
        // description (shown for or-play: it names a Wine path).
        for (label, hint) in [
            (
                "Game folder",
                "The folder with your OR2006C2C.exe, e.g. Z:\\home\\you\\Games\\OutRun 2006",
            ),
            ("Saves", "Saved Games\\OutRun 2006 (the default)"),
        ] {
            let field = ui.get_by_role_and_label(egui::accesskit::Role::TextInput, label);
            assert_eq!(testing::description(&field).as_deref(), Some(hint));
        }
    }

    #[test]
    fn clicking_changes_what_play_saves() {
        let dirs = Dirs::new("play");
        let mut ui = open(&dirs, None, false, false);
        assert!(ui.state().ready(), "{:?}", ui.state().check());
        show(&mut ui, Tab::Picture);
        ui.get_by_label("120").click();
        ui.get_by_label("Exclusive fullscreen").click();
        ui.get_by_label("Vsync").click();
        ui.run();
        assert_eq!(ui.state().form.fps_limit, 120);
        assert_eq!(ui.state().form.look, Look::Custom);
        show(&mut ui, Tab::Controls);
        ui.get_all_by_label("Merged").next().unwrap().click();
        ui.run();
        show(&mut ui, Tab::Extras);
        ui.get_by_label("Free drive").click();
        ui.run();
        play(&ui).click();
        ui.run();
        assert_eq!(ui.state().outcome().get(), Outcome::Play);
        let saved = std::fs::read_to_string(dirs.cfg()).unwrap();
        let (s, warnings) = config::Settings::parse_file(&saved);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(s.fps_limit, Some(120));
        assert_eq!(s.fullscreen, Some(true));
        assert_eq!(s.vsync, Some(Vsync::Off));
        assert_eq!(s.vibration, Some(Feedback::Merged));
        assert_eq!(s.free_drive, Some(false));
        assert_eq!(s.game, Some(dirs.root.join("game")));
    }

    #[test]
    fn a_look_card_sets_every_value() {
        let dirs = Dirs::new("look");
        let mut ui = open(&dirs, None, false, false);
        ui.get_by_label("Original 2006").click();
        ui.run();
        assert_eq!(ui.state().form.look, Look::Original);
        assert!(testing::checked(&ui.get_by_label("Original 2006")));
        assert_eq!(ui.state().form.resolution, None);
        assert!(!ui.state().form.sky_glow);
    }

    #[test]
    fn play_waits_for_a_good_game_folder() {
        let dirs = Dirs::new("nogame");
        let mut ui = open(&dirs, Some(""), false, false);
        assert!(!ui.state().ready());
        assert!(testing::disabled(&play(&ui)));
        ui.state_mut().tab = Tab::Extras;
        play(&ui).click();
        ui.run();
        assert_eq!(ui.state().outcome().get(), Outcome::Closed);
        assert!(!dirs.cfg().exists());
    }

    #[test]
    fn an_unknown_exe_says_where_to_report_it() {
        let dirs = Dirs::new("unknown");
        // Removed first: on Windows both spellings are one file.
        std::fs::remove_file(dirs.root.join("game/OR2006C2C.exe")).unwrap();
        std::fs::write(dirs.root.join("game/or2006c2c.EXE"), b"other").unwrap();
        let mut ui = open(&dirs, None, false, false);
        ui.run();
        match ui.state().check() {
            Check::Problem(p) => assert!(p.contains("SHA-256") && p.contains(REPORT_URL), "{p}"),
            other => panic!("{other:?}"),
        }
        ui.get_by_label("Report this version");
    }

    #[test]
    fn report_links_name_the_issue_forms() {
        let forms = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/ISSUE_TEMPLATE");
        for url in [REPORT_URL, BUG_URL] {
            let form = url.rsplit("template=").next().unwrap();
            assert!(forms.join(form).is_file(), "{url}: no {form}");
        }
    }

    #[test]
    fn disabled_controls_say_why() {
        let dirs = Dirs::new("disabled");
        let mut ui = open(&dirs, None, false, true);
        show(&mut ui, Tab::Controls);
        let swapped = ui.get_all_by_label("Swapped").last().unwrap();
        assert!(testing::disabled(&swapped));
        assert_eq!(
            testing::description(&swapped).as_deref(),
            Some("Impulse triggers work on Windows only.")
        );
        show(&mut ui, Tab::Extras);
        ui.get_by_label("CD switcher").click();
        ui.run();
        let shuffle = ui.get_by_label("Shuffle the music");
        assert!(testing::disabled(&shuffle));
        show(&mut ui, Tab::HdPack);
        assert!(testing::disabled(
            &ui.get_by_label("Use the HD texture pack")
        ));
    }

    /// A pad player reads why a row is disabled on the window, once (the
    /// SDK's chip row writes it under the chips, the switch under its
    /// label), and the row's own description stays under it.
    #[test]
    fn a_disabled_row_says_why_once_and_keeps_its_description() {
        let dirs = Dirs::new("disabled-once");
        let mut ui = open(&dirs, None, false, true);
        show(&mut ui, Tab::Controls);
        let why = "Impulse triggers work on Windows only.";
        assert_eq!(ui.get_all_by_label(why).count(), 1, "{why}");
        let _ = ui.get_by_label(option("impulse_triggers").description);
        show(&mut ui, Tab::Extras);
        ui.get_by_label("CD switcher").click();
        ui.run();
        // The switch paints its reason; no note under it repeats it.
        assert_eq!(ui.query_all_by_label("Needs the CD switcher.").count(), 0);
        let _ = ui.get_by_label(option("music_shuffle").description);
    }

    #[test]
    fn rumble_test_and_download_go_to_the_platform() {
        let dirs = Dirs::new("platform");
        let fake = Fake::default();
        let (rumbles, downloads) = (fake.rumbles.clone(), fake.downloads.clone());
        let game = dirs.root.join("game").display().to_string();
        let text = config::set_key(&config::template(), "game", &game);
        let mut ui = harness(Window::new(
            dirs.setup(Some(&text), false, false),
            Box::new(fake),
            None,
        ));
        show(&mut ui, Tab::Controls);
        ui.get_by_label("Test rumble").click();
        ui.run();
        assert_eq!(rumbles.load(Ordering::Relaxed), 1);
        show(&mut ui, Tab::HdPack);
        ui.get_by_label("Download").click();
        ui.run();
        assert_eq!(downloads.load(Ordering::Relaxed), 1);
        // Play waits while it runs.
        assert!(!ui.state().ready());
    }

    /// Windows 10 test (rea-launcher-sdk#1): with no pad, "Test rumble"
    /// talked about Wine on native Windows. Wine is named only under Wine.
    #[test]
    fn the_rumble_test_names_wine_only_under_wine() {
        for wine in [false, true] {
            let dirs = Dirs::new(&format!("rumble-{wine}"));
            let game = dirs.root.join("game").display().to_string();
            let text = config::set_key(&config::template(), "game", &game);
            let fake = Fake {
                no_pad: true,
                ..Fake::default()
            };
            let mut ui = harness(Window::new(
                dirs.setup(Some(&text), false, wine),
                Box::new(fake),
                None,
            ));
            show(&mut ui, Tab::Controls);
            ui.get_by_label("Test rumble").click();
            ui.run();
            let said = ui.state().rumble_test.clone().unwrap();
            assert!(said.starts_with("No pad took the rumble."), "{said}");
            assert_eq!(said.contains("Wine"), wine, "{said}");
            let _ = ui.get_by_label(&said);
        }
    }

    #[test]
    fn values_from_the_file_stay_shown() {
        let dirs = Dirs::new("odd");
        let game = dirs.root.join("game").display().to_string();
        let text =
            format!("game = {game}\nfps_limit = 75\nanisotropy = 2\nresolution = 1600x900\n");
        let mut ui = open(&dirs, Some(&text), false, false);
        show(&mut ui, Tab::Picture);
        assert!(testing::checked(&ui.get_by_label("75")));
        assert!(testing::checked(&ui.get_by_label("2×")));
        assert!(testing::checked(&ui.get_by_label("1600×900")));
    }

    #[test]
    fn every_switch_reaches_launcher_cfg() {
        let dirs = Dirs::new("switches");
        let mut ui = open(&dirs, None, false, false);
        let before = config::Settings::parse_file(&ui.state().form.save("")).0;
        let switches = [
            (Tab::Picture, "fullscreen"),
            (Tab::Picture, "vsync"),
            (Tab::Picture, "sky_glow"),
            (Tab::Picture, "xbox_brightness"),
            (Tab::Picture, "hd_characters"),
            (Tab::Controls, "manual_gearbox"),
            (Tab::Extras, "music_shuffle"),
            (Tab::Extras, "cd_switcher"),
            (Tab::Extras, "skip_logos"),
            (Tab::Extras, "free_drive"),
            (Tab::Extras, "horn"),
            (Tab::Extras, "random_anims"),
            (Tab::Extras, "show_miles"),
            (Tab::Extras, "soundtrack_fix"),
        ];
        for (tab, key) in switches {
            show(&mut ui, tab);
            ui.get_by_role_and_label(egui::accesskit::Role::Switch, option(key).label)
                .click();
            ui.run();
        }
        play(&ui).click();
        ui.run();
        let mut after =
            config::Settings::parse_file(&std::fs::read_to_string(dirs.cfg()).unwrap()).0;
        let mut before = before;
        for (_, key) in switches {
            let flipped = match key {
                "vsync" => Some(after.vsync != before.vsync),
                _ => after
                    .toggle(key)
                    .map(|v| *v)
                    .zip(before.toggle(key).map(|v| *v))
                    .map(|(a, b)| a != b),
            };
            assert_eq!(flipped, Some(true), "{key}");
        }
    }

    #[test]
    fn chips_reach_launcher_cfg() {
        let dirs = Dirs::new("chips");
        let mut ui = open(&dirs, None, false, false);
        show(&mut ui, Tab::Picture);
        for label in ["4K", "Game's own", "1024", "8×", "4× MSAA"] {
            ui.get_all_by_label(label).next().unwrap().click();
            ui.run();
        }
        show(&mut ui, Tab::Controls);
        for label in ["30%", "Strong", "DInput"] {
            ui.get_by_label(label).click();
            ui.run();
        }
        play(&ui).click();
        ui.run();
        let saved = std::fs::read_to_string(dirs.cfg()).unwrap();
        for line in [
            "resolution = 3840x2160",
            "draw_distance = 0",
            "reflections = 1024",
            "anisotropy = 8",
            "antialiasing = 2",
            "deadzone = 30",
            "vibration_strength = 10",
            "input_backend = dinput",
        ] {
            assert!(saved.lines().any(|l| l == line), "{line} not in\n{saved}");
        }
    }

    #[test]
    fn the_palette_and_text_sizes_pass_the_sdk_checks() {
        assert_eq!(
            theme::contrast_problems(&PALETTE, &METRICS),
            Vec::<String>::new()
        );
        assert_eq!(theme::text_size_problems(&METRICS), Vec::<String>::new());
    }

    #[test]
    fn remove_asks_twice() {
        let dirs = Dirs::new("remove");
        let pin = hdpack::pin();
        let dir = hdpack::root(&dirs.root.join("home")).join(pin.name());
        std::fs::create_dir_all(dir.join("textures/load")).unwrap();
        std::fs::write(
            dir.join(".complete"),
            rea_launcher::identify::hex(&pin.sha256),
        )
        .unwrap();
        let mut ui = open(&dirs, None, false, false);
        show(&mut ui, Tab::HdPack);
        assert_eq!(ui.state().form.hd_folder, dir.display().to_string());
        ui.get_by_label("Remove").click();
        ui.run();
        assert!(dir.exists(), "the first press only asks");
        ui.get_by_label("Remove").click();
        ui.run();
        assert!(!dir.exists());
        assert!(ui.state().form.hd_folder.is_empty());
    }
}
