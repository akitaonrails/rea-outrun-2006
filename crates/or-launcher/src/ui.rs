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
    self, Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Key,
    Mesh, Rect, Sense, Shape, Stroke, StrokeKind, TextureHandle, Vec2, pos2, vec2,
};

use or_launcher::config::{self, Settings};
use or_launcher::config::{Backend, Feedback, Vsync};
use or_launcher::form::{self, Form, Look, PadAction};
use or_launcher::{edition, hdpack, tweaks};

use crate::win32;

// Rosso, Modena yellow and black: the colours of OutRun's car.
const BASE: Color32 = Color32::from_rgb(0x0f, 0x0c, 0x0b);
const PANEL: Color32 = Color32::from_rgb(0x21, 0x1b, 0x18);
const LINE: Color32 = Color32::from_rgb(0x3d, 0x34, 0x30);
/// The action colour: Play, primary buttons, the tab underline. Never text
/// on the dark base (3.5:1); white on it is 5.5:1.
const RED: Color32 = Color32::from_rgb(0xd4, 0x00, 0x00);
const RED_BRIGHT: Color32 = Color32::from_rgb(0xe8, 0x16, 0x0e);
/// Chosen and good: selected chips, switches on, highlighted text.
const YELLOW: Color32 = Color32::from_rgb(0xff, 0xd4, 0x00);
const INK: Color32 = Color32::from_rgb(0xf6, 0xf1, 0xea);
const MUTED: Color32 = Color32::from_rgb(0xab, 0x9f, 0x96);
const ON_RED: Color32 = Color32::WHITE;
const ON_YELLOW: Color32 = Color32::from_rgb(0x1a, 0x13, 0x00);
/// Light coral: errors are text, and red is never text, so the two don't
/// meet.
const ERROR: Color32 = Color32::from_rgb(0xff, 0x95, 0x80);

const SIZE: Vec2 = vec2(form::WINDOW.0, form::WINDOW.1);
const PHOTO_W: f32 = 300.0;

/// How the window was left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Play pressed: the settings are saved.
    Play,
    /// Closed without playing.
    Closed,
}

/// Shows the window over `settings_path` (its text, or the template for a
/// first run). `linux` adds the gamescope scaler choice. Errors when no
/// window can be made (no OpenGL); the caller then goes on without it.
pub fn show(settings_path: &Path, home: &Path, text: &str, linux: bool) -> Result<Outcome, String> {
    let mut outcome = Outcome::Closed;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OutRun 2006")
            .with_inner_size(SIZE)
            .with_min_inner_size(SIZE)
            .with_max_inner_size(SIZE)
            .with_resizable(false)
            .with_maximize_button(false),
        // Under Wine the desktop's window manager places it (on the monitor
        // in use); Wine's idea of the main screen can be another monitor.
        centered: !win32::is_wine(),
        ..Default::default()
    };
    let path = settings_path.to_path_buf();
    let home = home.to_path_buf();
    let text = text.to_string();
    eframe::run_native(
        "OutRun 2006",
        options,
        Box::new(|cc| {
            set_style(&cc.egui_ctx);
            Ok(Box::new(Window::new(
                &cc.egui_ctx,
                path,
                home,
                text,
                linux,
                &mut outcome,
            )))
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(outcome)
}

fn set_style(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let font = |bytes: &'static [u8]| std::sync::Arc::new(FontData::from_static(bytes));
    fonts.font_data.insert(
        "barlow".into(),
        font(include_bytes!("../assets/fonts/Barlow-Regular.ttf")),
    );
    fonts.font_data.insert(
        "barlow-semibold".into(),
        font(include_bytes!("../assets/fonts/Barlow-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        "condensed".into(),
        font(include_bytes!(
            "../assets/fonts/BarlowCondensed-SemiBold.ttf"
        )),
    );
    fonts.font_data.insert(
        "decal".into(),
        font(include_bytes!(
            "../assets/fonts/BarlowCondensed-BlackItalic.ttf"
        )),
    );
    // Barlow everywhere (egui's own fonts are left out to keep the
    // launcher small); the named families are the other weights.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.insert(family, vec!["barlow".into()]);
    }
    for name in ["barlow-semibold", "condensed", "decal"] {
        fonts.families.insert(
            FontFamily::Name(name.into()),
            vec![name.into(), "barlow".into()],
        );
    }
    ctx.set_fonts(fonts);

    // One look whatever the system theme is.
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::dark();
        v.panel_fill = BASE;
        v.extreme_bg_color = PANEL;
        v.text_edit_bg_color = Some(PANEL);
        v.override_text_color = Some(INK);
        v.selection.bg_fill = RED.gamma_multiply(0.6);
        v.selection.stroke = Stroke::new(1.0, YELLOW);
        v.text_cursor.stroke = Stroke::new(2.0, YELLOW);
        for w in [
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.bg_stroke = Stroke::new(1.0, LINE);
            w.corner_radius = CornerRadius::same(6);
        }
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, MUTED);
        v.widgets.active.bg_stroke = Stroke::new(1.0, RED);
        style.spacing.item_spacing = vec2(8.0, 8.0);
    });
}

fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

fn named(name: &str, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(name.into()))
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
    /// The size for this screen is set on the first frame.
    sized: bool,
    /// What the last rumble test found.
    rumble_test: Option<String>,
    /// The gamepad buttons held at the last look, and what pressing them
    /// asks of this frame.
    pad_held: u16,
    pad_actions: Vec<PadAction>,
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
        text: String,
        linux: bool,
        outcome: &'a mut Outcome,
    ) -> Window<'a> {
        let screen = win32::screen_size();
        let wine = win32::is_wine();
        let form = if text == config::TEMPLATE {
            Form::first_run(screen, wine)
        } else {
            Form::new(&Settings::parse_file(&text).0, screen, wine)
        };
        let mut form = form;
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
            pads: win32::xinput_pads().len(),
            linux,
            wine,
            portrait: load_portrait(ctx),
            path,
            text,
            error: None,
            sized: false,
            rumble_test: None,
            pad_held: 0,
            pad_actions: Vec::new(),
            home,
            pack,
            hd_job: None,
            hd_rate: Rate::default(),
            hd_result: None,
            outcome,
        }
    }

    /// Draws everything larger on a large screen (1.5x at 4K), as the
    /// desktop's own scaling often doesn't reach programs under Wine, and
    /// centres the larger window.
    fn fit_screen(&mut self, ctx: &egui::Context) {
        self.sized = true;
        let native = ctx.native_pixels_per_point().unwrap_or(1.0);
        let zoom = form::window_zoom(self.screen, native);
        if zoom == 1.0 {
            return;
        }
        ctx.set_zoom_factor(zoom);
        let size = SIZE * zoom;
        for cmd in [
            egui::ViewportCommand::MinInnerSize(size),
            egui::ViewportCommand::MaxInnerSize(size),
            egui::ViewportCommand::InnerSize(size),
        ] {
            ctx.send_viewport_cmd(cmd);
        }
        if self.wine {
            return;
        }
        if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
            let at = ((monitor - size) / 2.0).max(Vec2::ZERO);
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(at.to_pos2()));
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
        match std::fs::write(&self.path, saved) {
            Ok(()) => {
                *self.outcome = Outcome::Play;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.error = Some(format!("Can't save the settings: {e}")),
        }
    }
}

fn load_portrait(ctx: &egui::Context) -> Option<TextureHandle> {
    let mut decoder = jpeg_decoder::Decoder::new(&include_bytes!("../assets/portrait.jpg")[..]);
    let rgb = decoder.decode().ok()?;
    let info = decoder.info()?;
    let size = [usize::from(info.width), usize::from(info.height)];
    (rgb.len() == size[0] * size[1] * 3).then(|| {
        ctx.load_texture(
            "portrait",
            egui::ColorImage::from_rgb(size, &rgb),
            egui::TextureOptions::LINEAR,
        )
    })
}

impl eframe::App for Window<'_> {
    /// The gamepad, as keys: the d-pad moves the focus (Tab, Shift+Tab), A
    /// presses the control in focus (Space); Start and LB/RB are handled in
    /// `ui`, which also asks for the next look.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        let now = win32::xinput_buttons();
        self.pad_actions = form::pad_actions(self.pad_held, now);
        self.pad_held = now;
        for action in &self.pad_actions {
            let (key, modifiers) = match action {
                PadAction::Next => (Key::Tab, egui::Modifiers::NONE),
                PadAction::Previous => (Key::Tab, egui::Modifiers::SHIFT),
                PadAction::Press => (Key::Space, egui::Modifiers::NONE),
                _ => continue,
            };
            for pressed in [true, false] {
                raw.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.sized {
            self.fit_screen(&ctx);
        }
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, BASE);
        self.draw_photo(ui, full);
        draw_decal(ui, full);

        let content = Rect::from_min_max(
            pos2(full.left() + PHOTO_W + 12.0, full.top() + 24.0),
            pos2(full.right() - 32.0, full.bottom() - 24.0),
        );
        let footer_h = 52.0;
        let body = Rect::from_min_max(
            content.min,
            pos2(content.right(), content.bottom() - footer_h - 16.0),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
            ui.label(
                egui::RichText::new("OutRun 2006")
                    .font(named("decal", 46.0))
                    .color(INK),
            );
            ui.add_space(-6.0);
            ui.label(
                egui::RichText::new("COAST 2 COAST")
                    .font(named("condensed", 17.0))
                    .color(YELLOW),
            );
            ui.add_space(4.0);
            self.status(ui);
            ui.add_space(4.0);
            self.tabs(ui);
            ui.add_space(14.0);
            // Long tabs scroll inside the body instead of running under
            // the footer.
            egui::ScrollArea::vertical()
                .id_salt(self.tab)
                .auto_shrink([false, false])
                .show(ui, |ui| match self.tab {
                    Tab::Play => self.play_tab(ui),
                    Tab::Picture => self.picture_tab(ui),
                    Tab::HdPack => self.hd_tab(ui),
                    Tab::Controls => self.controls_tab(ui),
                    Tab::Extras => self.extras_tab(ui),
                });
        });
        let footer = Rect::from_min_max(
            pos2(content.left(), content.bottom() - footer_h),
            content.max,
        );
        self.footer(ui, footer, &ctx);
        self.poll_download();

        if ctx.input(|i| i.key_pressed(Key::Enter)) && !ctx.egui_wants_keyboard_input() {
            self.play(&ctx);
        }
        // A pad sends no events: look again soon.
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        for action in std::mem::take(&mut self.pad_actions) {
            match action {
                PadAction::Play => self.play(&ctx),
                PadAction::NextTab => self.tab = self.tab.step(1),
                PadAction::PreviousTab => self.tab = self.tab.step(-1),
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
        if let Some(tex) = &self.portrait {
            // Fill the height; keep the middle of the picture (the car).
            let size = tex.size_vec2();
            let (w, h) = (size.x, size.y);
            let shown_w = rect.height() * w / h;
            let over = (shown_w - rect.width()).max(0.0);
            let u0 = over / 2.0 / shown_w;
            let u1 = 1.0 - u0;
            p.image(
                tex.id(),
                rect,
                Rect::from_min_max(pos2(u0, 0.0), pos2(u1, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Without the artwork: a black-to-red-to-yellow sky.
            let mut mesh = Mesh::default();
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
                let _ = &mut mesh;
            }
        }
        // Fades to black join the photo to the page.
        gradient(
            p,
            Rect::from_min_max(
                pos2(rect.left() + rect.width() * 0.55, rect.top()),
                rect.max,
            ),
            true,
        );
        gradient(
            p,
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 130.0), rect.max),
            false,
        );
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

    fn tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 26.0;
            for (tab, name) in [
                (Tab::Play, "Play"),
                (Tab::Picture, "Picture"),
                (Tab::HdPack, "HD Pack"),
                (Tab::Controls, "Controls"),
                (Tab::Extras, "Extras"),
            ] {
                let on = self.tab == tab;
                let galley = ui.painter().layout_no_wrap(
                    name.to_string(),
                    named("condensed", 22.0),
                    if on { INK } else { MUTED },
                );
                let (rect, resp) =
                    ui.allocate_exact_size(galley.size() + vec2(0.0, 8.0), Sense::click());
                let color = if on || resp.hovered() { INK } else { MUTED };
                ui.painter().galley(rect.min, galley, color);
                if on {
                    let bar = Rect::from_min_size(
                        pos2(rect.left(), rect.bottom() - 3.0),
                        vec2(rect.width(), 3.0),
                    );
                    ui.painter().rect_filled(bar, 0.0, RED);
                }
                if resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.tab = tab;
                }
            }
        });
        let y = ui.cursor().top() - 4.0;
        let r = ui.max_rect();
        ui.painter()
            .hline(r.left()..=r.right(), y, Stroke::new(1.0, LINE));
    }

    fn play_tab(&mut self, ui: &mut egui::Ui) {
        heading(ui, "Game folder");
        let game_hint = if self.linux {
            format!(
                "The folder with your {}, e.g. Z:\\home\\you\\Games\\OutRun 2006",
                edition::EXE
            )
        } else {
            format!("The folder with your installed {}", edition::EXE)
        };
        if folder_field(
            ui,
            &mut self.form.game,
            &game_hint,
            &format!("Choose the folder with your {}", edition::EXE),
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
        ui.add_space(10.0);

        heading(ui, "Look");
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
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (look, title, note) in cards {
                if look_card(
                    ui,
                    vec2(w, 104.0),
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

        heading(ui, "Saves");
        folder_field(
            ui,
            &mut self.form.saves,
            "Saved Games\\OutRun 2006 (the default)",
            "Choose where your saves are kept",
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
                field(ui, name);
                ui.vertical(|ui| add(ui));
            });
        };
        row(ui, "Resolution", &mut |ui| {
            chips(ui, resolutions, &mut f.resolution);
        });
        row(ui, "Frame rate", &mut |ui| {
            chips(ui, &FRAME_RATES, &mut f.fps_limit);
            if f.fps_limit > 60 || f.fps_limit == 0 {
                ui.label(
                    egui::RichText::new("Above 60 the picture is interpolated (experimental).")
                        .font(font(13.0))
                        .color(MUTED),
                );
            }
        });
        row(ui, "Vsync", &mut |ui| {
            chips(ui, &VSYNCS, &mut f.vsync);
        });
        row(ui, "Draw distance", &mut |ui| {
            chips(ui, &DISTANCES, &mut f.draw_distance);
        });
        row(ui, "Reflections", &mut |ui| {
            chips(ui, &REFLECTIONS, &mut f.reflections);
        });
        row(ui, "Anisotropy", &mut |ui| {
            chips(ui, &ANISOTROPY, &mut f.anisotropy);
        });
        row(ui, "Anti-aliasing", &mut |ui| {
            chips(ui, &ANTIALIASING, &mut f.antialiasing);
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
                toggle(ui, &mut f.sky_glow, "Sky glow")
                    .on_hover_text("The console versions' glowing sky (bloom)");
                toggle(ui, &mut f.xbox_brightness, "Xbox brightness")
                    .on_hover_text("The Xbox releases' brighter picture");
            });
            toggle(ui, &mut f.hd_characters, "Hi-def characters").on_hover_text(
                "The hi-def driver and passenger models the PC release shipped unused",
            );
        });
        if self.linux {
            row(ui, "Scaler", &mut |ui| {
                let options: Vec<_> = form::SCALERS.iter().map(|s| (s.0, s.1)).collect();
                chips(ui, &options, &mut f.scaler);
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
        heading(ui, "HD texture pack");
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
            win32::open_url(config::HD_PACK_PAGE);
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
        heading(ui, "Already have it?");
        let before = self.form.hd_folder.clone();
        folder_field(
            ui,
            &mut self.form.hd_folder,
            "The folder you unzipped the pack into",
            "Choose the folder you unzipped the HD texture pack into",
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
        ui.horizontal(|ui| {
            if primary_button(ui, label).clicked() {
                self.start_download(ui.ctx());
            }
            ui.add_space(10.0);
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
        });
    }

    /// A pack is in place: switch it on or off, or remove the download.
    fn pack_ready(&mut self, ui: &mut egui::Ui) {
        let downloaded = hdpack::installed(&self.pack, &self.home);
        let ours = downloaded
            .as_ref()
            .is_some_and(|dir| Path::new(self.form.hd_folder.trim()) == dir.as_path());
        toggle(ui, &mut self.form.hd_textures, "Use the HD texture pack");
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
                if outline_button(ui, "Remove").clicked() {
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
        progress_bar(ui, fraction);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(text).font(font(13.5)).color(INK));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if outline_button(ui, "Cancel").clicked() {
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
            let free = win32::free_space(&home);
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
                field(ui, name);
                ui.vertical(|ui| add(ui));
            });
        };
        row(ui, "Deadzone", &mut |ui| {
            chips(ui, &DEADZONES, &mut f.deadzone);
            ui.label(
                egui::RichText::new("Steering deadzone; the game's own is 20%.")
                    .font(font(13.0))
                    .color(MUTED),
            );
        });
        row(ui, "Rumble", &mut |ui| {
            chips(ui, &FEEDBACK, &mut f.vibration);
            ui.add_space(4.0);
            let mut strength = f.vibration_strength;
            chips(ui, &STRENGTHS, &mut strength);
            f.vibration_strength = strength;
            ui.add_space(8.0);
            if f.vibration != Feedback::Off && outline_button(ui, "Test").clicked() {
                let took = win32::test_rumble(f.vibration_strength * 10);
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
                chips(ui, &FEEDBACK, &mut f.impulse_triggers);
                ui.label(
                    egui::RichText::new("Xbox One/Series pads on Windows.")
                        .font(font(13.0))
                        .color(MUTED),
                );
            });
        }
        row(ui, "Input API", &mut |ui| {
            chips(ui, &BACKENDS, &mut f.input_backend);
            ui.label(
                egui::RichText::new(
                    "If a pad isn't seen, try another (wgi is Windows.Gaming.Input).",
                )
                .font(font(13.0))
                .color(MUTED),
            );
        });
        toggle(ui, &mut f.manual_gearbox, "Manual gearbox by default");
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
        toggle(
            ui,
            &mut f.cd_switcher,
            "Change the music during a race (Back or X = next track, Z = previous)",
        );
        if f.cd_switcher {
            toggle(
                ui,
                &mut f.music_shuffle,
                "Shuffle the race music each launch",
            );
        }
        toggle(ui, &mut f.skip_logos, "Skip the boot logos");
        toggle(
            ui,
            &mut f.free_drive,
            "Free drive: no countdown, coast as long as you like",
        );
        toggle(
            ui,
            &mut f.horn,
            "The horn works outside the girlfriend requests",
        );
        toggle(
            ui,
            &mut f.random_anims,
            "Roadside animations from any game mode",
        );
        toggle(
            ui,
            &mut f.show_miles,
            "OutRun miles next to the menu's sign-in text",
        );
        toggle(
            ui,
            &mut f.soundtrack_fix,
            "Restore the first two seconds of “Rush a Difficulty”",
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
        let button = Rect::from_min_size(
            pos2(rect.right() - 170.0, rect.top()),
            vec2(170.0, rect.height()),
        );
        let hint = if self.form.resolution.is_some() {
            "The game runs at this resolution as a borderless window."
        } else {
            "Settings are saved when you press Play."
        };
        let galley = ui.painter().layout(
            hint.to_string(),
            font(13.0),
            MUTED,
            button.left() - rect.left() - 20.0,
        );
        let y = rect.center().y - galley.size().y / 2.0;
        ui.painter().galley(pos2(rect.left(), y), galley, MUTED);

        let resp = ui.interact(button, ui.scope_id().with("play"), Sense::click());
        let ready = self.ready();
        let fill = match (ready, resp.hovered()) {
            (false, _) => LINE,
            (true, true) => RED_BRIGHT,
            (true, false) => RED,
        };
        // A slanted plate, like the stripes.
        let s = 10.0;
        let plate = vec![
            pos2(button.left() + s, button.top()),
            button.right_top(),
            pos2(button.right() - s, button.bottom()),
            button.left_bottom(),
        ];
        ui.painter()
            .add(Shape::convex_polygon(plate, fill, Stroke::NONE));
        ui.painter().text(
            button.center(),
            Align2::CENTER_CENTER,
            "Play",
            named("decal", 30.0),
            if ready { ON_RED } else { MUTED },
        );
        if resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
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

/// Fades `rect` from clear to the black base: to the right when `horizontal`,
/// else downwards.
fn gradient(p: &egui::Painter, rect: Rect, horizontal: bool) {
    let mut mesh = Mesh::default();
    let clear = Color32::from_rgba_unmultiplied(BASE.r(), BASE.g(), BASE.b(), 0);
    let (a, b, c, d) = if horizontal {
        (clear, BASE, BASE, clear)
    } else {
        (clear, clear, BASE, BASE)
    };
    mesh.colored_vertex(rect.left_top(), a);
    mesh.colored_vertex(rect.right_top(), b);
    mesh.colored_vertex(rect.right_bottom(), c);
    mesh.colored_vertex(rect.left_bottom(), d);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

/// Two slanted stripes in the top right corner: rosso red with a thinner
/// yellow one beside it, like racing stripes on a hood.
fn draw_decal(ui: &egui::Ui, full: Rect) {
    let p = ui.painter().with_clip_rect(full);
    for (i, (w, alpha)) in [(30.0, 1.0f32), (16.0, 0.9)].into_iter().enumerate() {
        let x = full.right() - 120.0 + i as f32 * 40.0;
        let slant = 60.0;
        let pts = vec![
            pos2(x, full.top()),
            pos2(x + w, full.top()),
            pos2(x + w - slant, full.top() + 112.0),
            pos2(x - slant, full.top() + 112.0),
        ];
        let color = if i == 0 { RED } else { YELLOW };
        p.add(Shape::convex_polygon(
            pts,
            color.gamma_multiply(alpha),
            Stroke::NONE,
        ));
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(named("barlow-semibold", 14.0))
            .color(MUTED),
    );
}

/// A row's label, as tall as a chip. Rows of labels and chips are
/// top-aligned (`horizontal_top`), so both start on the same line.
fn field(ui: &mut egui::Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(118.0, 30.0), Sense::hover());
    ui.painter().text(
        rect.left_center(),
        Align2::LEFT_CENTER,
        text,
        font(15.0),
        INK,
    );
}

/// A text field with a Browse button; true when it changed.
/// A thin bar on the panel colour, filled from the left.
fn progress_bar(ui: &mut egui::Ui, fraction: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 10.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(5), LINE);
    let fill = Rect::from_min_size(
        rect.min,
        vec2(width * fraction.clamp(0.0, 1.0), rect.height()),
    );
    painter.rect_filled(fill, CornerRadius::same(5), RED);
}

/// The tab's main action: a filled plate, like Play.
fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), named("barlow-semibold", 15.0), ON_RED);
    let size = vec2(galley.size().x + 36.0, 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() { RED_BRIGHT } else { RED };
    ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, ON_RED);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

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

fn folder_field(ui: &mut egui::Ui, value: &mut String, hint: &str, title: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let w = ui.available_width() - 100.0;
        let edit = egui::TextEdit::singleline(value)
            .hint_text(egui::RichText::new(hint).color(MUTED))
            .font(font(15.0))
            .margin(vec2(10.0, 8.0))
            .desired_width(w);
        changed |= ui.add(edit).changed();
        if outline_button(ui, "Browse…").clicked()
            && let Some(dir) = win32::pick_folder(title)
        {
            *value = dir.display().to_string();
            changed = true;
        }
    });
    changed
}

fn outline_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(92.0, 34.0), Sense::click());
    let hover = resp.hovered();
    ui.painter().rect(
        rect,
        CornerRadius::same(6),
        if hover { LINE } else { Color32::TRANSPARENT },
        Stroke::new(1.5, RED),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        named("barlow-semibold", 15.0),
        INK,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A row of choices; the chosen one is on the yellow plate.
fn chips<T: PartialEq + Copy, S: AsRef<str>>(
    ui: &mut egui::Ui,
    options: &[(T, S)],
    current: &mut T,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (value, label) in options {
            let on = *current == *value;
            let galley = ui.painter().layout_no_wrap(
                label.as_ref().to_string(),
                if on {
                    named("barlow-semibold", 14.0)
                } else {
                    font(14.0)
                },
                if on { ON_YELLOW } else { INK },
            );
            let size = vec2(galley.size().x + 24.0, 30.0);
            let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
            let fill = if on {
                YELLOW
            } else if resp.hovered() {
                LINE
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect(
                rect,
                CornerRadius::same(6),
                fill,
                Stroke::new(1.0, if on { YELLOW } else { LINE }),
                StrokeKind::Inside,
            );
            let at = rect.center() - galley.size() / 2.0;
            ui.painter().galley(at, galley, INK);
            if resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                *current = *value;
            }
        }
    });
}

/// A switch: knob left and outlined is off, knob right on the yellow plate
/// is on.
fn toggle(ui: &mut egui::Ui, on: &mut bool, text: &str) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font(15.0), INK);
    let size = vec2(46.0 + 12.0 + galley.size().x, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let t = ui.ctx().animate_bool(resp.id, *on);
    let track = Rect::from_min_size(pos2(rect.left(), rect.center().y - 13.0), vec2(46.0, 26.0));
    let p = ui.painter();
    p.rect(
        track,
        CornerRadius::same(13),
        if *on { YELLOW } else { Color32::TRANSPARENT },
        Stroke::new(1.5, if *on { YELLOW } else { MUTED }),
        StrokeKind::Inside,
    );
    let x = egui::lerp(track.left() + 13.0..=track.right() - 13.0, t);
    p.circle_filled(
        pos2(x, track.center().y),
        9.0,
        if *on { ON_YELLOW } else { MUTED },
    );
    p.galley(
        pos2(
            track.right() + 12.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        INK,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// One of the looks; true when clicked.
fn look_card(
    ui: &mut egui::Ui,
    size: Vec2,
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
        named("condensed", 22.0),
        if on { YELLOW } else { INK },
    );
    if recommended {
        // A small red tab on the card's top edge.
        let galley = p.layout_no_wrap("Recommended".into(), named("barlow-semibold", 12.0), ON_RED);
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
