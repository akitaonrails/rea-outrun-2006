//! The settings window on Windows (and Wine): the portable window
//! (`or_launcher::window`) with the system's folder picker, browser, pad,
//! rumble and WinHTTP downloads.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use or_launcher::directx;
use or_launcher::window::{METRICS, Outcome, PALETTE, Platform, SIZE, Setup, TITLE, Window};
use rea_launcher::download::{self, Job, Pin};
use rea_launcher_ui::eframe;
use rea_launcher_ui::theme;
use rea_launcher_ui::window::WindowFit;
use rea_launcher_win32::{dialog, disk, display, wine, xinput};

/// Windows' side of the window.
struct Win32;

impl Platform for Win32 {
    fn pick_folder(&self, title: &str) -> Option<PathBuf> {
        dialog::pick_folder(title)
    }

    fn open_url(&self, url: &str) {
        dialog::open_url(url);
    }

    fn test_rumble(&self, percent: u32) -> usize {
        xinput::test_rumble(percent, std::time::Duration::from_millis(500))
    }

    fn pads(&self) -> usize {
        xinput::connected()
    }

    fn buttons(&self) -> u16 {
        xinput::buttons()
    }

    fn download(&self, pin: Pin, root: PathBuf, job: Arc<Job>, done: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(move || {
            let free = |dir: &Path| disk::free_space(dir);
            download::install(&pin, &root, &free, &crate::app::http(), &job);
            done();
        });
    }

    fn download_directx(
        &self,
        pin: Pin,
        root: PathBuf,
        job: Arc<Job>,
        done: Box<dyn FnOnce() + Send>,
    ) {
        std::thread::spawn(move || {
            let free = |dir: &Path| disk::free_space(dir);
            directx::install(&pin, &root, &free, &crate::app::http(), &job);
            done();
        });
    }
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
    // Under Wine the window is created at its final size (XWayland can
    // lose a later resize), with Wine's own DPI scale divided out; on
    // Windows the first frame fits it to the DPI-aware scale.
    // The scale first: under Wine it makes the process DPI aware, so the
    // screen is measured in pixels.
    let wine_scale = display::wine_dpi_scale();
    let wine = wine::is_wine();
    let screen = display::screen_size();
    let fit = WindowFit::new(SIZE, wine, screen, wine_scale).with_native_fit(true);
    let options = fit.native_options(TITLE);
    let setup = Setup {
        path: settings_path.to_path_buf(),
        saved: saved.map(str::to_string),
        home: home.to_path_buf(),
        launcher_dir: std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf)),
        system_dirs: directx::windows_system_dirs(),
        linux,
        wine,
        screen,
    };
    let window = Window::new(setup, Box::new(Win32), Some(fit));
    let outcome = window.outcome();
    eframe::run_native(
        TITLE,
        options,
        Box::new(|cc| {
            theme::install_with(&cc.egui_ctx, &PALETTE, &METRICS);
            Ok(Box::new(window))
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(outcome.get())
}
