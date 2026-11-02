//! The settings window: one window for Windows and for Linux (shown
//! through Wine by `or-play`). What it edits and saves is `form::Form`;
//! this module only draws it.
//!
//! Look: the car's colours. A black base with warm charcoal panels, rosso
//! red for actions (Play, the tab underline), Modena yellow for whatever is
//! chosen, a red and a yellow stripe in the corner, and a condensed italic
//! title like the arcade cabinet's. Five tabs
//! keep it short: Play (the game folder and one choice of look), Picture,
//! HD Pack, Controls and Extras.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Key, Mesh, Rect, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, Vec2, pos2, vec2,
};
use rea_launcher_ui::pad::{self, Nav, PadNav};
use rea_launcher_ui::theme::{self, Palette, body as font, condensed, decal, semibold};
use rea_launcher_ui::widgets::{self, Crop};
use rea_launcher_ui::window::{Layout, WindowFit};
use rea_launcher_win32::{dialog, disk, display, wine, xinput};

use or_launcher::config;
use or_launcher::config::{Backend, Feedback, Vsync};
use or_launcher::form::{self, Form, Look};
use or_launcher::{edition, hdpack, tweaks};
use rea_launcher::fsx;

use crate::win32;

// Rosso, Modena yellow and black: the colours of OutRun's car.
const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Red is the action colour (Play, primary buttons, the tab underline),
/// never text on the dark base (3.5:1; white on it is 5.5:1). Yellow is
/// whatever is chosen. Errors are light coral text, so red and error never
/// meet.
const PALETTE: Palette = Palette {
    base: rgb(0x0f0c0b),
    panel: rgb(0x211b18),
    line: rgb(0x3d3430),
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
};
const BASE: Color32 = PALETTE.base;
const PANEL: Color32 = PALETTE.panel;
const LINE: Color32 = PALETTE.line;
const RED: Color32 = PALETTE.action;
const ON_RED: Color32 = PALETTE.on_action;
const YELLOW: Color32 = PALETTE.select;
const INK: Color32 = PALETTE.ink;
const MUTED: Color32 = PALETTE.muted;
const ERROR: Color32 = PALETTE.error;

/// The window's design size in points (taller than the SDK's usual page).
const SIZE: Vec2 = vec2(880.0, 760.0);
const PHOTO_W: f32 = 300.0;

/// How the window was left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Play pressed: the settings are saved.
    Play,
    /// Closed without playing.
    Closed,
}

/// Shows the window over `settings_path`, whose text is `saved` (`None`
/// when there is no file yet: a first run, saved over the template).
/// `linux` adds the gamescope scaler choice. Errors when no window can be
/// made (no OpenGL); the caller then goes on without it.
pub fn show(
    settings_path: &Path,
    home: &Path,
    saved: Option<&str>,
    linux: bool,
) -> Result<Outcome, String> {
    let mut outcome = Outcome::Closed;
    // Under Wine the window is created at its final size (XWayland can
    // lose a later resize), with Wine's own DPI scale divided out; on
    // Windows the first frame fits it to the DPI-aware scale.
    // The scale first: under Wine it makes the process DPI aware, so the
    // screen is measured in pixels.
    let wine_scale = display::wine_dpi_scale();
    let fit = WindowFit::new(SIZE, wine::is_wine(), display::screen_size(), wine_scale);
    let options = fit.native_options("OutRun 2006");
    let path = settings_path.to_path_buf();
    let home = home.to_path_buf();
    let saved = saved.map(str::to_string);
    eframe::run_native(
        "OutRun 2006",
        options,
        Box::new(|cc| {
            set_style(&cc.egui_ctx);
            Ok(Box::new(Window::new(
                &cc.egui_ctx,
                path,
                home,
                saved,
                linux,
                fit,
                &mut outcome,
            )))
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(outcome)
}

/// The SDK's fonts and dark style in OutRun's colours; selected text keeps
/// its red highlight.
fn set_style(ctx: &egui::Context) {
    theme::install(ctx, &PALETTE);
    ctx.all_styles_mut(|style| style.visuals.selection.bg_fill = RED.gamma_multiply(0.6));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tab {
    Play,
    Picture,
    HdPack,
    Controls,
    Extras,
}

impl Tab {
    const ALL: [Tab; 5] = [
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

/// What the game folder holds, checked when its text changes.
enum Check {
    Empty,
    Found(&'static str),
    Problem(String),
}

fn check_game(folder: &str) -> Check {
    let folder = folder.trim();
    if folder.is_empty() {
        return Check::Empty;
    }
    let dir = Path::new(folder);
    if !dir.is_dir() {
        return Check::Problem("No such folder.".into());
    }
    let Some(exe) = std::fs::read_dir(dir).ok().and_then(|entries| {
        entries.flatten().map(|e| e.path()).find(|p| {
            p.file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case(edition::EXE))
        })
    }) else {
        return Check::Problem(format!("There is no {} in this folder.", edition::EXE));
    };
    match std::fs::read(&exe) {
        Ok(bytes) => match edition::identify_with_bundled(&bytes, bundled_exe().as_deref()) {
            Ok(e) => Check::Found(e.name),
            Err(digest) => Check::Problem(format!(
                "This {} is a version the launcher doesn't know.\nSHA-256: {} — please report it.",
                edition::EXE,
                edition::to_hex(&digest)
            )),
        },
        Err(e) => Check::Problem(format!("Can't read {}: {e}", edition::EXE)),
    }
}

/// The replacement EXE shipped in `tweaks/` next to the launcher.
fn bundled_exe() -> Option<Vec<u8>> {
    let dir = std::env::current_exe()
        .ok()?
        .parent()?
        .join(tweaks::TWEAKS_DIR);
    std::fs::read(dir.join(edition::EXE)).ok()
}

struct Window<'a> {
    form: Form,
    tab: Tab,
    check: Check,
    checked: String,
    screen: Option<(u32, u32)>,
    resolutions: Vec<(Option<(u32, u32)>, String)>,
    pads: usize,
    linux: bool,
    wine: bool,
    portrait: Option<TextureHandle>,
    path: PathBuf,
    text: String,
    error: Option<String>,
    /// Sizes the window for this screen on the first frame.
    fit: WindowFit,
    /// What the last rumble test found.
    rumble_test: Option<String>,
    /// The gamepad, and the steps it asked of this frame.
    pad: PadNav,
    pending: Vec<Nav>,
    /// The launcher's home folder, where the HD pack is downloaded.
    home: PathBuf,
    pack: hdpack::Pack,
    /// The HD pack download in progress, its speed, and how the last one
    /// ended.
    hd_job: Option<Arc<hdpack::Job>>,
    hd_rate: Rate,
    hd_result: Option<(String, Color32)>,
    outcome: &'a mut Outcome,
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

impl<'a> Window<'a> {
    fn new(
        ctx: &egui::Context,
        path: PathBuf,
        home: PathBuf,
        saved: Option<String>,
        linux: bool,
        fit: WindowFit,
        outcome: &'a mut Outcome,
    ) -> Window<'a> {
        let screen = display::screen_size();
        let wine = wine::is_wine();
        let mut form = Form::from_file(saved.as_deref(), screen, wine);
        let text = saved.unwrap_or_else(|| config::TEMPLATE.to_string());
        let pack = hdpack::Pack::pinned();
        // A pack downloaded earlier is offered even if its settings weren't
        // saved.
        if form.hd_folder.trim().is_empty()
            && let Some(dir) = hdpack::installed(&pack, &home)
        {
            form.hd_folder = dir.display().to_string();
        }
        let check = check_game(&form.game);
        Window {
            checked: form.game.clone(),
            check,
            form,
            tab: Tab::Play,
            screen,
            resolutions: form::resolutions(screen),
            pads: xinput::connected(),
            linux,
            wine,
            portrait: widgets::load_jpeg(ctx, "portrait", include_bytes!("../assets/portrait.jpg")),
            path,
            text,
            error: None,
            fit,
            rumble_test: None,
            pad: PadNav::default(),
            pending: Vec::new(),
            home,
            pack,
            hd_job: None,
            hd_rate: Rate::default(),
            hd_result: None,
            outcome,
        }
    }

    fn ready(&self) -> bool {
        matches!(self.check, Check::Found(_)) && self.hd_job.is_none()
    }

    fn play(&mut self, ctx: &egui::Context) {
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
                *self.outcome = Outcome::Play;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.error = Some(format!("Can't save the settings: {e}")),
        }
    }
}

impl eframe::App for Window<'_> {
    /// The gamepad, as keys (the SDK's pad navigation): the d-pad moves
    /// the focus and steps a chip row, A presses; Start and LB/RB are
    /// handled in `ui`, which also asks for the next look.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let navs = self.pad.update(xinput::buttons());
        pad::inject(ctx, raw, &navs);
        self.pending.extend(navs);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.fit.first_frame(&ctx);
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
            widgets::tab_body(ui, self.tab, |ui| match self.tab {
                Tab::Play => self.play_tab(ui),
                Tab::Picture => self.picture_tab(ui),
                Tab::HdPack => self.hd_tab(ui),
                Tab::Controls => self.controls_tab(ui),
                Tab::Extras => self.extras_tab(ui),
            });
        });
        self.footer(ui, layout.footer, &ctx);
        self.poll_download();

        if ctx.input(|i| i.key_pressed(Key::Enter)) && !ctx.egui_wants_keyboard_input() {
            self.play(&ctx);
        }
        // A pad sends no events: look again soon.
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
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
            self.check = check_game(&self.form.game);
        }
    }
}

impl Window<'_> {
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
        let x = rect.left() + 20.0;
        let y = rect.bottom() - 20.0;
        p.text(
            pos2(x, y),
            Align2::LEFT_BOTTOM,
            "Music can be changed mid-race",
            font(11.0),
            MUTED,
        );
        p.text(
            pos2(x, y - 16.0),
            Align2::LEFT_BOTTOM,
            "the girl's request is optional",
            font(11.0),
            MUTED,
        );
        p.text(
            pos2(x, y - 34.0),
            Align2::LEFT_BOTTOM,
            "Splash Wave or Magical Sound Shower —",
            font(11.0),
            MUTED,
        );
    }

    fn status(&self, ui: &mut egui::Ui) {
        let (text, color) = match (&self.error, &self.check) {
            (Some(e), _) => (e.clone(), ERROR),
            (None, Check::Found(_)) if self.hd_job.is_some() => (
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
        widgets::heading(ui, "Game folder");
        let game_hint = if self.linux {
            format!(
                "The folder with your {}, e.g. Z:\\home\\you\\Games\\OutRun 2006",
                edition::EXE
            )
        } else {
            format!("The folder with your installed {}", edition::EXE)
        };
        if widgets::folder_field(ui, &mut self.form.game, &game_hint, || {
            dialog::pick_folder(&format!("Choose the folder with your {}", edition::EXE))
        }) {
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
        ui.add_space(10.0);

        widgets::heading(ui, "Look");
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
        let best = match form::recommended(self.screen) {
            form::Look::SteamDeck => Look::SteamDeck,
            _ => Look::CoastToCoast,
        };
        let gap = 10.0;
        let w = (ui.available_width() - 3.0 * gap) / 4.0;
        // One title size for all four cards: the largest at which every
        // title fits.
        let titles: Vec<&str> = cards.iter().map(|(_, title, _)| *title).collect();
        let title_size = widgets::fit_title(ui, &titles, w - 28.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (look, title, note) in cards {
                if look_card(
                    ui,
                    vec2(w, 104.0),
                    title_size,
                    self.form.look == look,
                    title,
                    &note,
                    look == best,
                ) {
                    self.form.set_look(look, self.screen, self.wine);
                    if look == Look::Custom {
                        self.tab = Tab::Picture;
                    }
                }
            }
        });
        ui.add_space(14.0);

        widgets::heading(ui, "Saves");
        widgets::folder_field(
            ui,
            &mut self.form.saves,
            "Saved Games\\OutRun 2006 (the default)",
            || dialog::pick_folder("Choose where your saves are kept"),
        );
        ui.label(
            egui::RichText::new("Leave empty for the default. Your License.dat and records are copied in before the game starts and back when it ends.")
                .font(font(13.0))
                .color(MUTED),
        );
    }

    fn picture_tab(&mut self, ui: &mut egui::Ui) {
        let before = self.form.clone();
        let f = &mut self.form;
        let resolutions = &self.resolutions;
        let row = |ui: &mut egui::Ui, name: &str, add: &mut dyn FnMut(&mut egui::Ui)| {
            ui.horizontal_top(|ui| {
                widgets::field(ui, name);
                ui.vertical(|ui| add(ui));
            });
        };
        row(ui, "Resolution", &mut |ui| {
            widgets::chips(ui, resolutions, &mut f.resolution, None);
        });
        row(ui, "Frame rate", &mut |ui| {
            widgets::chips(ui, &FRAME_RATES, &mut f.fps_limit, None);
            if f.fps_limit > 60 || f.fps_limit == 0 {
                ui.label(
                    egui::RichText::new("Above 60 the picture is interpolated (experimental).")
                        .font(font(13.0))
                        .color(MUTED),
                );
            }
        });
        row(ui, "Vsync", &mut |ui| {
            widgets::chips(ui, &VSYNCS, &mut f.vsync, None);
        });
        row(ui, "Draw distance", &mut |ui| {
            widgets::chips(ui, &DISTANCES, &mut f.draw_distance, None);
        });
        row(ui, "Reflections", &mut |ui| {
            widgets::chips(ui, &REFLECTIONS, &mut f.reflections, None);
        });
        row(ui, "Anisotropy", &mut |ui| {
            widgets::chips(ui, &ANISOTROPY, &mut f.anisotropy, None);
        });
        row(ui, "Anti-aliasing", &mut |ui| {
            widgets::chips(ui, &ANTIALIASING, &mut f.antialiasing, None);
            if f.antialiasing > 0 {
                ui.label(
                    egui::RichText::new("Smooths edges. On NVIDIA under Windows, trackside cloth and barriers are supersampled too.")
                        .font(font(13.0))
                        .color(MUTED),
                );
            }
        });
        row(ui, "", &mut |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 18.0;
                widgets::toggle(ui, &mut f.sky_glow, "Sky glow", None)
                    .on_hover_text("The console versions' glowing sky (bloom)");
                widgets::toggle(ui, &mut f.xbox_brightness, "Xbox brightness", None)
                    .on_hover_text("The Xbox releases' brighter picture");
            });
            widgets::toggle(ui, &mut f.hd_characters, "Hi-def characters", None).on_hover_text(
                "The hi-def driver and passenger models the PC release shipped unused",
            );
        });
        if self.linux {
            row(ui, "Scaler", &mut |ui| {
                let options: Vec<_> = form::SCALERS.iter().map(|s| (s.0, s.1)).collect();
                widgets::chips(ui, &options, &mut f.scaler, None);
                let note = form::SCALERS
                    .iter()
                    .find(|s| s.0 == f.scaler)
                    .map_or("", |s| s.2);
                ui.label(egui::RichText::new(note).font(font(13.0)).color(MUTED));
            });
        }
        if self.form != before {
            self.form.look = Form::look_of(&self.form, self.screen, self.wine);
        }
    }

    fn hd_tab(&mut self, ui: &mut egui::Ui) {
        widgets::heading(ui, "HD texture pack");
        ui.label(
            egui::RichText::new(
                "The community's redrawn menus, HUD, sprites and button prompts at high resolution \
                 (Envido32's compilation of the Sonic-TV OR2006Sprites project). An optional download \
                 from its own GitHub release, never bundled with the launcher.",
            )
            .font(font(13.5))
            .color(MUTED),
        );
        if ui
            .link(
                egui::RichText::new("The pack's page and credits")
                    .font(font(13.0))
                    .color(YELLOW),
            )
            .clicked()
        {
            dialog::open_url(config::HD_PACK_PAGE);
        }
        ui.add_space(12.0);

        if let Some(job) = self.hd_job.clone() {
            self.download_progress(ui, &job);
        } else if tweaks::texture_base(Path::new(self.form.hd_folder.trim())).is_some() {
            self.pack_ready(ui);
        } else {
            self.pack_offer(ui);
        }
        if let Some((text, color)) = &self.hd_result {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(text).font(font(13.5)).color(*color));
        }

        ui.add_space(18.0);
        widgets::heading(ui, "Already have it?");
        let before = self.form.hd_folder.clone();
        widgets::folder_field(
            ui,
            &mut self.form.hd_folder,
            "The folder you unzipped the pack into",
            || dialog::pick_folder("Choose the folder you unzipped the HD texture pack into"),
        );
        if self.form.hd_folder != before {
            self.hd_result = None;
        }
        let (note, color) = hd_pack_note(&self.form.hd_folder);
        ui.label(egui::RichText::new(note).font(font(13.0)).color(color));
    }

    /// No pack yet: the download offer (or its resumption).
    fn pack_offer(&mut self, ui: &mut egui::Ui) {
        let partial = hdpack::partial(&self.pack, &self.home);
        let label = if partial > 0 {
            "Resume download"
        } else {
            "Download"
        };
        if widgets::primary_button(ui, label).clicked() {
            self.start_download(ui.ctx());
        }
        ui.add_space(6.0);
        // On its own line, so it wraps instead of widening the tab.
        let detail = if partial > 0 {
            format!(
                "{:.0} of {:.0} MB already here.",
                hdpack::mb(partial),
                hdpack::mb(self.pack.size)
            )
        } else {
            format!(
                "{:.0} MB, {:.1} GB once unpacked. The launcher downloads, checks and unpacks it for you.",
                hdpack::mb(self.pack.size),
                hdpack::gb(self.pack.unpacked)
            )
        };
        ui.label(egui::RichText::new(detail).font(font(13.0)).color(MUTED));
    }

    /// A pack is in place: switch it on or off, or remove the download.
    fn pack_ready(&mut self, ui: &mut egui::Ui) {
        let downloaded = hdpack::installed(&self.pack, &self.home);
        let ours = downloaded
            .as_ref()
            .is_some_and(|dir| Path::new(self.form.hd_folder.trim()) == dir.as_path());
        widgets::toggle(
            ui,
            &mut self.form.hd_textures,
            "Use the HD texture pack",
            None,
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let what = if ours {
                format!("Downloaded: {}", self.pack.name())
            } else {
                "Your own copy of the pack".to_string()
            };
            ui.label(egui::RichText::new(what).font(font(13.5)).color(YELLOW));
            if ours {
                ui.add_space(10.0);
                if widgets::outline_button(ui, "Remove").clicked() {
                    match hdpack::remove(&self.pack, &self.home) {
                        Ok(()) => {
                            self.form.hd_textures = false;
                            self.form.hd_folder.clear();
                            self.hd_result = Some((
                                format!("Removed; {:.1} GB freed.", hdpack::gb(self.pack.unpacked)),
                                MUTED,
                            ));
                        }
                        Err(e) => self.hd_result = Some((e, ERROR)),
                    }
                }
            }
        });
        ui.label(
            egui::RichText::new(
                "With the pack on, Clarissa looks as in the Japanese release, which its sprites are drawn for.",
            )
            .font(font(13.0))
            .color(MUTED),
        );
    }

    fn download_progress(&mut self, ui: &mut egui::Ui, job: &hdpack::Job) {
        let (fraction, text) = match job.phase() {
            hdpack::Phase::Downloading { done, total } => {
                let rate = self.hd_rate.update(done);
                let speed = if rate > 0.0 {
                    let left = total.saturating_sub(done) as f64 / rate;
                    format!(" · {:.1} MB/s · {}", rate / 1_000_000.0, time_left(left))
                } else {
                    String::new()
                };
                (
                    done as f32 / total.max(1) as f32,
                    format!(
                        "Downloading {:.0} of {:.0} MB{speed}",
                        hdpack::mb(done),
                        hdpack::mb(total)
                    ),
                )
            }
            hdpack::Phase::Verifying => (1.0, "Checking the download…".to_string()),
            hdpack::Phase::Unpacking { done, total } => (
                done as f32 / total.max(1) as f32,
                format!(
                    "Unpacking {:.1} of {:.1} GB",
                    hdpack::gb(done),
                    hdpack::gb(total)
                ),
            ),
            _ => (0.0, "Connecting…".to_string()),
        };
        widgets::progress_bar(ui, fraction);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(text).font(font(13.5)).color(INK));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if widgets::outline_button(ui, "Cancel").clicked() {
                    job.cancel();
                }
            });
        });
    }

    fn start_download(&mut self, ctx: &egui::Context) {
        let job = Arc::new(hdpack::Job::default());
        let worker = job.clone();
        let (pack, home, ctx) = (self.pack.clone(), self.home.clone(), ctx.clone());
        std::thread::spawn(move || {
            let free = disk::free_space(&home);
            hdpack::install(&pack, &home, free, &win32::WinHttp, &worker);
            ctx.request_repaint();
        });
        self.hd_job = Some(job);
        self.hd_rate = Rate::default();
        self.hd_result = None;
    }

    /// Picks up how a finished download ended.
    fn poll_download(&mut self) {
        let Some(job) = &self.hd_job else {
            return;
        };
        let phase = job.phase();
        if phase.running() {
            return;
        }
        self.hd_job = None;
        self.hd_result = match phase {
            hdpack::Phase::Done(dir) => {
                self.form.hd_folder = dir.display().to_string();
                self.form.hd_textures = true;
                Some(("Installed and switched on.".to_string(), YELLOW))
            }
            hdpack::Phase::Cancelled => Some((
                "Stopped. Press Resume download to carry on later.".to_string(),
                MUTED,
            )),
            hdpack::Phase::Failed(e) => Some((e, ERROR)),
            _ => None,
        };
    }

    fn controls_tab(&mut self, ui: &mut egui::Ui) {
        let before = self.form.clone();
        let f = &mut self.form;
        let row = |ui: &mut egui::Ui, name: &str, add: &mut dyn FnMut(&mut egui::Ui)| {
            ui.horizontal_top(|ui| {
                widgets::field(ui, name);
                ui.vertical(|ui| add(ui));
            });
        };
        row(ui, "Deadzone", &mut |ui| {
            widgets::chips(ui, &DEADZONES, &mut f.deadzone, None);
            ui.label(
                egui::RichText::new("Steering deadzone; the game's own is 20%.")
                    .font(font(13.0))
                    .color(MUTED),
            );
        });
        row(ui, "Rumble", &mut |ui| {
            widgets::chips(ui, &FEEDBACK, &mut f.vibration, None);
            ui.add_space(4.0);
            let mut strength = f.vibration_strength;
            widgets::chips(ui, &STRENGTHS, &mut strength, None);
            f.vibration_strength = strength;
            ui.add_space(8.0);
            if f.vibration != Feedback::Off && widgets::outline_button(ui, "Test").clicked() {
                let took = xinput::test_rumble(
                    f.vibration_strength * 10,
                    std::time::Duration::from_millis(500),
                );
                self.rumble_test = Some(match took {
                    0 => "No pad took the rumble. Under Wine the pad must be in X-input mode; on 8BitDo pads use the X-input switch.".to_string(),
                    _ => "Sent: your pad should have shaken for half a second. In a race it shakes on shifts, drifts and crashes.".to_string(),
                });
            }
        });
        if let Some(result) = &self.rumble_test {
            ui.label(egui::RichText::new(result).font(font(13.0)).color(YELLOW));
        }
        if !self.wine && f.vibration != Feedback::Off {
            row(ui, "Impulse triggers", &mut |ui| {
                widgets::chips(ui, &FEEDBACK, &mut f.impulse_triggers, None);
                ui.label(
                    egui::RichText::new("Xbox One/Series pads on Windows.")
                        .font(font(13.0))
                        .color(MUTED),
                );
            });
        }
        row(ui, "Input API", &mut |ui| {
            widgets::chips(ui, &BACKENDS, &mut f.input_backend, None);
            ui.label(
                egui::RichText::new(
                    "If a pad isn't seen, try another (wgi is Windows.Gaming.Input).",
                )
                .font(font(13.0))
                .color(MUTED),
            );
        });
        widgets::toggle(ui, &mut f.manual_gearbox, "Manual gearbox by default", None);
        let pads = match self.pads {
            0 => "The Test button checks Xbox-style (XInput) pads; in the game, PlayStation and Switch pads rumble too.".to_string(),
            1 => "One Xbox-style pad connected.".to_string(),
            n => format!("{n} Xbox-style pads connected."),
        };
        ui.label(egui::RichText::new(pads).font(font(13.0)).color(MUTED));
        if self.form != before {
            self.form.look = Form::look_of(&self.form, self.screen, self.wine);
        }
    }

    fn extras_tab(&mut self, ui: &mut egui::Ui) {
        let before = self.form.clone();
        let f = &mut self.form;
        widgets::toggle(
            ui,
            &mut f.cd_switcher,
            "Change the music during a race (Back or X = next track, Z = previous)",
            None,
        );
        if f.cd_switcher {
            widgets::toggle(
                ui,
                &mut f.music_shuffle,
                "Shuffle the race music each launch",
                None,
            );
        }
        widgets::toggle(ui, &mut f.skip_logos, "Skip the boot logos", None);
        widgets::toggle(
            ui,
            &mut f.free_drive,
            "Free drive: no countdown, coast as long as you like",
            None,
        );
        widgets::toggle(
            ui,
            &mut f.horn,
            "The horn works outside the girlfriend requests",
            None,
        );
        widgets::toggle(
            ui,
            &mut f.random_anims,
            "Roadside animations from any game mode",
            None,
        );
        widgets::toggle(
            ui,
            &mut f.show_miles,
            "OutRun miles next to the menu's sign-in text",
            None,
        );
        widgets::toggle(
            ui,
            &mut f.soundtrack_fix,
            "Restore the first two seconds of “Rush a Difficulty”",
            None,
        );
        ui.label(
            egui::RichText::new("Online play points at the community server (clarissa.port0.org); pick a user name and password in the game's multiplayer menu.")
                .font(font(13.0))
                .color(MUTED),
        );
        if self.form != before {
            self.form.look = Form::look_of(&self.form, self.screen, self.wine);
        }
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

/// The choices offered, with their labels.
pub const FRAME_RATES: [(u32, &str); 5] = [
    (60, "60"),
    (90, "90"),
    (120, "120"),
    (144, "144"),
    (0, "Unlimited"),
];
pub const VSYNCS: [(Vsync, &str); 2] = [(Vsync::On, "On"), (Vsync::Off, "Off")];
pub const DISTANCES: [(u32, &str); 3] = [(0, "Game's own"), (4, "4"), (8, "8")];
pub const REFLECTIONS: [(u32, &str); 4] = [
    (0, "Original"),
    (512, "512"),
    (1024, "1024"),
    (2048, "2048"),
];
pub const ANISOTROPY: [(u32, &str); 4] = [(0, "Game's own"), (4, "4×"), (8, "8×"), (16, "16×")];
pub const ANTIALIASING: [(u32, &str); 2] = [(0, "Off"), (2, "4× MSAA")];
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

/// "about 3 min left".
fn time_left(seconds: f64) -> String {
    match seconds as u64 {
        0..=59 => "under a minute left".to_string(),
        s => format!("about {} min left", s.div_ceil(60)),
    }
}

/// Whether the chosen HD pack folder holds textures, for the note under it.
fn hd_pack_note(folder: &str) -> (String, Color32) {
    let folder = folder.trim();
    if folder.is_empty() {
        return (
            "A separate download (326 MB). Unzip it anywhere and choose that folder.".to_string(),
            MUTED,
        );
    }
    match tweaks::texture_base(std::path::Path::new(folder)) {
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

/// One of the looks; true when clicked. Kept here: the SDK's `look_card`
/// draws its note smaller and closer to the edge (12 pt padding, 13 pt
/// note) than OutRun's cards (14, 13.5).
fn look_card(
    ui: &mut egui::Ui,
    size: Vec2,
    title_size: f32,
    on: bool,
    title: &str,
    note: &str,
    recommended: bool,
) -> bool {
    let w = size.x;
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let p = ui.painter();
    let fill = if on {
        PANEL
    } else if resp.hovered() {
        PANEL.gamma_multiply(0.6)
    } else {
        Color32::TRANSPARENT
    };
    p.rect(
        rect,
        CornerRadius::same(8),
        fill,
        Stroke::new(if on { 2.0 } else { 1.0 }, if on { YELLOW } else { LINE }),
        StrokeKind::Inside,
    );
    let pad = 14.0;
    p.text(
        rect.left_top() + vec2(pad, pad - 2.0),
        Align2::LEFT_TOP,
        title,
        condensed(title_size),
        if on { YELLOW } else { INK },
    );
    if recommended {
        // A small red tab on the card's top edge.
        let galley = p.layout_no_wrap("Recommended".into(), semibold(12.0), ON_RED);
        let tab = Rect::from_min_size(
            pos2(
                rect.right() - pad - galley.size().x - 12.0,
                rect.top() - 9.0,
            ),
            galley.size() + vec2(12.0, 2.0),
        );
        p.rect_filled(tab, CornerRadius::same(4), RED);
        p.galley(tab.min + vec2(6.0, 1.0), galley, ON_RED);
    }
    let galley = p.layout(note.to_string(), font(13.5), MUTED, w - 2.0 * pad);
    p.galley(rect.left_top() + vec2(pad, pad + 32.0), galley, MUTED);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}
