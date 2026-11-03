#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod ui;

#[cfg(windows)]
fn main() {
    let code = match app::run() {
        Ok(code) => code,
        Err(error) => {
            app::log_line(&format!("error: {error}"));
            app::fatal(&error);
            1
        }
    };
    std::process::exit(code as i32);
}

#[cfg(not(windows))]
fn main() {
    eprintln!(
        "OutRunLauncher is a Windows program; on Linux run it with Proton, Wine, or or-play."
    );
    std::process::exit(2);
}

#[cfg(windows)]
mod app {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

    use or_launcher::config::{self, Settings};
    use or_launcher::instance;
    use or_launcher::window::{self, Outcome};
    use or_launcher::{edition, form, hdpack, runtime, tweaks};
    use rea_launcher::fsx;
    use rea_launcher::log::Log;
    use rea_launcher_win32::http::WinHttp;
    use rea_launcher_win32::instance::{self as win32_instance, Window};
    use rea_launcher_win32::report::{self, Mode, Run};
    use rea_launcher_win32::{disk, display, gpu, wine, xinput};

    use crate::ui;

    const EXE: &str = edition::EXE;

    fn home() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("OutRunLauncher")
    }

    /// `launcher.log` in the launcher's home folder (started afresh past
    /// 1 MiB).
    pub fn log_line(line: &str) {
        Log::new(home().join("launcher.log")).line(line);
    }

    /// A `launcher.cfg` next to the exe wins (portable installs); otherwise
    /// the one in the launcher's home folder.
    fn settings_path() -> PathBuf {
        let portable = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|dir| dir.join(config::FILE_NAME)));
        match portable {
            Some(path) if path.is_file() => path,
            _ => home().join(config::FILE_NAME),
        }
    }

    /// `%USERPROFILE%\Saved Games\OutRun 2006`, Windows' usual place for
    /// game saves (inside the prefix under Wine/Proton).
    fn default_saves() -> PathBuf {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(home)
            .join("Saved Games")
            .join("OutRun 2006")
    }

    fn find_game(settings: &Settings) -> Result<PathBuf, String> {
        if let Some(game) = &settings.game {
            return Ok(game.clone());
        }
        // A launcher placed in the game folder needs no argument.
        if let Some(dir) = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            && dir.join(EXE).is_file()
        {
            return Ok(dir);
        }
        Err(format!(
            "Tell the launcher where the game is: run it once with\n\n  OutRunLauncher --game <folder with {EXE}>\n\nThe folder is remembered."
        ))
    }

    /// launcher.cfg's text, `None` when there is none yet. An unreadable
    /// file stops the launcher, which then never writes over it.
    fn read_settings(path: &Path) -> Result<Option<String>, String> {
        config::read_file(path).map_err(|error| {
            format!(
                "Can't read the settings in {}: {error}\n\nFix its permissions, or delete it to start over.",
                path.display()
            )
        })
    }

    /// `--settings` exit codes for or-play: closed without playing, or no
    /// window could be shown (play with the saved settings).
    const WINDOW_CLOSED: u32 = 3;
    const WINDOW_UNAVAILABLE: u32 = 4;

    const TITLE: &str = "OutRun 2006 launcher";

    /// Who may see this run's messages, set once the command line is read
    /// (the SDK's `report::audience`: under Wine or-play's `--no-window`
    /// game run has a player watching; a `--no-window` script on Windows
    /// has none).
    static SAYS: OnceLock<Mode> = OnceLock::new();

    /// The run's message mode; before the command line is read, a play run
    /// a person may be watching.
    fn says() -> Mode {
        SAYS.get()
            .copied()
            .unwrap_or(Mode::Play { interactive: true })
    }

    /// An error the launcher stops on: a dialog when a person may be
    /// watching, and stdout, which or-play, the AppImage and Proton keep
    /// in a log (SDK `report`, so it reaches the player under Wine too).
    pub fn fatal(text: &str) {
        report::fatal(TITLE, text, says());
    }

    /// A message while preparing to play (another launcher, or the game,
    /// is running).
    fn play_note(text: &str) {
        log_lines(text);
        report::note(TITLE, text, says(), true);
    }

    /// Output of a command (`--help`, `--check`, a bad flag): the
    /// terminal, a dialog only when a person started the launcher without
    /// one. Always logged.
    fn command(text: &str, error: bool) {
        log_lines(text);
        if error {
            report::fatal(TITLE, text, says());
        } else {
            report::note(TITLE, text, says(), false);
        }
    }

    fn log_lines(text: &str) {
        for line in text.lines() {
            log_line(line);
        }
    }

    fn launcher_dir() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()?
            .parent()
            .map(Path::to_path_buf)
    }

    /// The DLL overrides for the game, merged with the player's own.
    fn wine_overrides() -> String {
        tweaks::wine_overrides(std::env::var("WINEDLLOVERRIDES").ok().as_deref())
    }

    /// The HTTPS client for downloads the player asks for.
    pub fn http() -> WinHttp {
        WinHttp::new(concat!("OutRunLauncher/", env!("CARGO_PKG_VERSION")))
    }

    /// `--download-hd-pack`: the HD Pack tab's download without the window,
    /// logging progress every 10 percent; switches the pack on when done.
    fn download_hd_pack(settings_path: &Path) -> Result<u32, String> {
        use or_launcher::hdpack;
        use rea_launcher::download::{self, Job, Phase};
        let pin = hdpack::pin();
        let root = hdpack::root(&home());
        let job = std::sync::Arc::new(Job::default());
        let worker = job.clone();
        let handle = std::thread::spawn(move || {
            let free = |dir: &Path| disk::free_space(dir);
            download::install(&pin, &root, &free, &http(), &worker)
        });
        let mut logged = None;
        while !handle.is_finished() {
            let phase = job.phase();
            let what = match phase {
                Phase::Downloading { .. } => Some("downloading"),
                Phase::Unpacking { .. } => Some("unpacking"),
                _ => None,
            };
            let step = what.zip(phase.fraction().map(|f| (f * 10.0) as u32));
            if step.is_some() && step != logged {
                let (what, tenth) = step.unwrap_or_default();
                log_line(&format!("hd pack: {what} {}%", tenth * 10));
                logged = step;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        match handle
            .join()
            .map_err(|_| "the download stopped unexpectedly".to_string())?
        {
            Phase::Done(dir) => {
                let text = read_settings(settings_path)?.unwrap_or_else(config::template);
                let text = config::set_key(&text, "hd_textures", "on");
                let text = config::set_key(&text, "hd_textures_folder", &dir.display().to_string());
                if let Some(parent) = settings_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fsx::write_atomic(settings_path, text).map_err(|e| e.to_string())?;
                log_line(&format!(
                    "hd pack: installed in {} and switched on",
                    dir.display()
                ));
                Ok(0)
            }
            Phase::Failed(e) => Err(e),
            other => Err(format!("hd pack: {other:?}")),
        }
    }

    /// `--check`: everything a launch needs, without launching. Exit code
    /// 0 when it would play, 1 when something stops it.
    fn check(
        game: &Path,
        detected: &edition::Edition,
        runtime_dir: &Path,
        wine: bool,
        warnings: &[String],
    ) -> u32 {
        let mut lines = vec![
            format!("Game folder: {}", game.display()),
            format!("Version: {}", detected.name),
        ];
        let found = launcher_dir()
            .map(|dir| tweaks::check_components(&dir, wine))
            .unwrap_or_default();
        let mut problems = found.errors.clone();
        if let Err(e) = runtime::check_separate(game, runtime_dir) {
            problems.push(e);
        }
        let probe = runtime_dir.join(".or-check");
        match fs::create_dir_all(runtime_dir).and_then(|_| fsx::write_atomic(&probe, "")) {
            Ok(()) => {
                let _ = fs::remove_file(&probe);
                lines.push(format!(
                    "Runtime folder: {} (writable)",
                    runtime_dir.display()
                ));
            }
            Err(e) => problems.push(format!(
                "cannot write to the runtime folder {}: {e}",
                runtime_dir.display()
            )),
        }
        lines.extend(found.warnings.iter().map(|w| format!("warning: {w}")));
        lines.extend(warnings.iter().map(|w| format!("warning: {w}")));
        lines.extend(problems.iter().map(|p| format!("problem: {p}")));
        let ok = problems.is_empty();
        if !ok {
            lines.push(format!(
                "If you can't fix it, report it with your launcher.log: {}",
                window::BUG_URL
            ));
        }
        lines.insert(
            0,
            if ok {
                "Ready to play.".to_string()
            } else {
                "Not ready to play:".to_string()
            },
        );
        command(&lines.join("\n"), !ok);
        u32::from(!ok)
    }

    pub fn run() -> Result<u32, String> {
        log_line(concat!("OutRunLauncher ", env!("CARGO_PKG_VERSION")));
        let settings_path = settings_path();
        let cli: Vec<String> = std::env::args().skip(1).collect();
        // --help and a bad command line answer like a command.
        let as_command = Run {
            command: true,
            ..Run::from_args(&cli, instance::COMMANDS)
        };
        if cli
            .iter()
            .any(|a| matches!(a.as_str(), "--help" | "-h" | "/?"))
        {
            let _ = SAYS.set(report::audience(as_command).mode());
            command(&config::usage(), false);
            return Ok(0);
        }
        let (args, switches) = match Settings::parse_args(cli.clone()) {
            Ok(parsed) => parsed,
            Err(message) => {
                let _ = SAYS.set(report::audience(as_command).mode());
                command(&message, true);
                return Ok(2);
            }
        };
        let _ = SAYS.set(report::audience(instance::run(&switches)).mode());
        let switch = |name: &str| switches.iter().any(|s| s == name);
        let settings_only = switch("--settings");
        let check_only = switch("--check");
        // One launcher at a time, held until the game this run starts has
        // ended: a second start raises this launcher's window (exit 0) or
        // says the game runs (exit 5, which or-play reads as "nothing
        // started"). --check only reads and runs alongside.
        let _instance = if !instance::needs_instance(&switches) {
            None
        } else {
            match win32_instance::single(instance::NAME) {
                Ok(guard) => Some(guard),
                Err(running) => {
                    let action = instance::is_action(&switches);
                    let second = running.second_start(action, Window::Own(window::TITLE));
                    log_line(second.log_line());
                    match instance::message(second) {
                        // An action's outcome goes where its output goes.
                        Some(message) if action => command(message, true),
                        Some(message) => play_note(message),
                        None => {}
                    }
                    return Ok(second.exit_code() as u32);
                }
            }
        };
        if switch("--download-hd-pack") {
            return download_hd_pack(&settings_path);
        }
        if config::opens_window(&args, &switches) {
            if wine::is_wine() {
                match gpu::prefer_glx() {
                    Ok(true) => log_line("Wine settings window: OpenGL through GLX (UseEGL=N)"),
                    Ok(false) => {}
                    Err(error) => log_line(&format!("warning: {error}")),
                }
            }
            // No launcher.cfg yet: the window opens on the recommended look.
            let saved = read_settings(&settings_path)?;
            match ui::show(&settings_path, &home(), saved.as_deref(), switch("--linux")) {
                Ok(Outcome::Play) => {
                    log_line("settings window: play");
                    if settings_only {
                        return Ok(0);
                    }
                }
                Ok(Outcome::Closed) => {
                    log_line("settings window: closed");
                    return Ok(if settings_only { WINDOW_CLOSED } else { 0 });
                }
                Err(error) => {
                    // No window possible (no OpenGL): play with the saved
                    // settings.
                    log_line(&format!("settings window unavailable: {error}"));
                    if settings_only {
                        return Ok(WINDOW_UNAVAILABLE);
                    }
                }
            }
        }

        let existing = read_settings(&settings_path)?;
        let template = config::template();
        let file_text = existing.as_deref().unwrap_or(&template);
        let (saved, warnings) = Settings::parse_file(file_text);
        for warning in &warnings {
            log_line(&format!("warning: {warning}"));
        }
        let wine = wine::is_wine();
        let screen = display::screen_size();
        let preset = |look: &str| form::look_settings(look, screen, wine);
        let mut settings = Settings::combine(saved.clone(), args, &preset);
        // The pack the window would offer: the one downloaded earlier.
        if settings.hd_textures_folder.is_none() {
            settings.hd_textures_folder = hdpack::installed(&home());
        }
        let game = find_game(&settings)?;
        let exe_path =
            rea_launcher::paths::find_ignoring_case(&game, EXE).unwrap_or_else(|| game.join(EXE));
        let exe_bytes = fs::read(&exe_path)
            .map_err(|error| format!("cannot read {}: {error}", exe_path.display()))?;
        let bundled =
            launcher_dir().and_then(|dir| fs::read(dir.join(tweaks::TWEAKS_DIR).join(EXE)).ok());
        let detected = edition::identify_with_bundled(&exe_bytes, bundled.as_deref()).map_err(|digest| {
            format!(
                "This OR2006C2C.exe is a version the launcher doesn't know.\n\nSHA-256: {}\n\nPlease report it at {} with where your copy comes from (Steam, DVD, which repack) and your launcher.log.",
                edition::to_hex(&digest),
                window::REPORT_URL
            )
        })?;
        log_line(&format!(
            "game: {} ({}, SHA-256 {})",
            game.display(),
            detected.name,
            edition::to_hex(&rea_launcher::identify::sha256(&exe_bytes))
        ));
        let runtime_dir = settings
            .runtime
            .clone()
            .unwrap_or_else(|| home().join("OutRun2006"));
        if check_only {
            return Ok(check(&game, &detected, &runtime_dir, wine, &warnings));
        }
        runtime::check_separate(&game, &runtime_dir)?;
        let saves_state = runtime_dir.join(runtime::SAVES_STATE);
        // Temp files left by launchers before the SDK (`name.tmp`).
        runtime::remove_old_temps(&[
            &runtime_dir.join(runtime::MANIFEST),
            &saves_state,
            &settings_path,
        ]);
        let report = runtime::sync(&game, &runtime_dir)
            .map_err(|error| format!("cannot prepare {}: {error}", runtime_dir.display()))?;
        for warning in &report.warnings {
            log_line(&format!("warning: {warning}"));
        }
        log_line(&format!(
            "runtime: {} ({} files copied)",
            runtime_dir.display(),
            report.brought
        ));

        // The player's saves live outside the runtime copy; the game reads
        // and writes its own SaveGame folder, kept in step before and
        // after.
        let saves = settings.saves.clone().unwrap_or_else(default_saves);
        let rt_save = runtime::save_dir(&runtime_dir)
            .map_err(|error| format!("cannot prepare saves: {error}"))?;
        let created = !saves.is_dir();
        let moved = runtime::saves_in(&saves, &rt_save, &saves_state)
            .map_err(|error| format!("cannot sync saves with {}: {error}", saves.display()))?;
        if created {
            log_line(&format!(
                "saves: created {} ({} files)",
                saves.display(),
                moved.to_saves.len()
            ));
        }
        log_line(&format!(
            "saves: {} files updated from {}, {} newer ones copied there",
            moved.to_runtime.len(),
            saves.display(),
            if created { 0 } else { moved.to_saves.len() }
        ));

        tweaks::install(
            &game,
            &runtime_dir,
            &launcher_dir().unwrap_or_default(),
            wine,
            &settings,
            &mut |line| log_line(line),
        )?;
        // An outrun2006.ini naming a display adapter that isn't attached
        // (another PC, a laptop off its dock) stops the game: use the first.
        let attached = display::adapters().iter().filter(|a| a.attached).count();
        tweaks::check_adapter(&runtime_dir, attached, &mut |line| log_line(line))?;
        if wine {
            match wine::seed_winebus(&rea_launcher::wine::SDL_ONLY) {
                Ok(true) => log_line(
                    "wine: controller baseline written (winebus: hidraw and Wine's device mapping off)",
                ),
                Ok(false) => {}
                Err(error) => log_line(&format!("warning: wine controller baseline: {error}")),
            }
        }

        // Remember the game folder; the rest of the file stays as written.
        if existing.is_none() || saved.game.as_ref() != Some(&game) {
            let text = config::set_key(file_text, "game", &game.display().to_string());
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fsx::write_atomic(&settings_path, text)
                .map_err(|error| format!("cannot save {}: {error}", settings_path.display()))?;
        }

        if !wine {
            match gpu::prefer_fast_gpu(&runtime_dir.join(EXE)) {
                Ok(true) => log_line("Windows: high-performance GPU for the game"),
                Ok(false) => {}
                Err(error) => log_line(&format!("warning: {error}")),
            }
        } else {
            log_line(&format!("wine: DLL overrides {}", wine_overrides()));
            // The environment value above only reaches Wine through
            // or-play; the per-program key works under plain Wine/Proton.
            let dlls: Vec<_> = tweaks::app_dll_overrides(true)
                .into_iter()
                .map(|dll| (dll, wine::DllMode::NativeThenBuiltin))
                .collect();
            match wine::app_dll_overrides(EXE, &dlls) {
                Ok(applied) => {
                    if !applied.written.is_empty() {
                        log_line(&format!(
                            "wine: {EXE} loads the bundled {} (AppDefaults)",
                            applied.written.join(", ")
                        ));
                    }
                    for (dll, value) in &applied.player {
                        log_line(&format!("wine: {dll} kept as the player set it ({value})"));
                    }
                }
                Err(error) => log_line(&format!("warning: Wine DLL overrides: {error}")),
            }
        }

        let mut command = Command::new(runtime_dir.join(EXE));
        command.current_dir(&runtime_dir);
        if wine {
            command.env("WINEDLLOVERRIDES", wine_overrides());
            command.env(
                "WINEDEBUG",
                std::env::var("WINEDEBUG").unwrap_or_else(|_| "-all".into()),
            );
        }
        let status = command.status().map_err(|error| {
            format!("cannot start {}: {error}", runtime_dir.join(EXE).display())
        })?;
        xinput::stop_rumble();
        let code = status.code().unwrap_or(1) as u32;
        log_line(&format!("game exited with {code}"));
        // The mod writes crash reports into CrashDumps.
        if let Ok(dir) = fs::read_dir(runtime_dir.join("CrashDumps"))
            && dir.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("dmp"))
            })
        {
            log_line("the game crashed: a report is in the runtime's CrashDumps folder");
        }
        match runtime::saves_out(&rt_save, &saves, &saves_state) {
            Ok(copied) if !copied.is_empty() => {
                log_line(&format!("saves: {} files written back", copied.len()))
            }
            Ok(_) => {}
            Err(error) => log_line(&format!(
                "warning: saves not written back to {}: {error}",
                saves.display()
            )),
        }
        Ok(code)
    }
}
