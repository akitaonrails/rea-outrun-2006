#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod ui;
#[cfg(windows)]
mod win32;

#[cfg(windows)]
fn main() {
    let code = match app::run() {
        Ok(code) => code,
        Err(error) => {
            app::log_line(&format!("error: {error}"));
            win32::message("OutRun 2006 launcher", &error, true);
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
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use or_launcher::config::{self, Settings};
    use or_launcher::{edition, runtime, tweaks};

    use crate::{ui, win32};

    const EXE: &str = edition::EXE;

    fn home() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("OutRunLauncher")
    }

    pub fn log_line(line: &str) {
        let dir = home();
        let _ = fs::create_dir_all(&dir);
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("launcher.log"))
        {
            let _ = writeln!(file, "{line}");
        }
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

    /// `--settings` exit codes for or-play: closed without playing, or no
    /// window could be shown (play with the saved settings).
    const WINDOW_CLOSED: u32 = 3;
    const WINDOW_UNAVAILABLE: u32 = 4;

    fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
        let found = args.iter().any(|arg| arg == flag);
        args.retain(|arg| arg != flag);
        found
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

    /// `--download-hd-pack`: the HD Pack tab's download without the window,
    /// logging progress every 10 percent; switches the pack on when done.
    fn download_hd_pack(settings_path: &Path) -> Result<u32, String> {
        use or_launcher::hdpack::{self, Pack, Phase};
        let pack = Pack::pinned();
        let home = home();
        let job = std::sync::Arc::new(hdpack::Job::default());
        let worker = job.clone();
        let thread_pack = pack.clone();
        let thread_home = home.clone();
        let handle = std::thread::spawn(move || {
            let free = win32::free_space(&thread_home);
            hdpack::install(&thread_pack, &thread_home, free, &win32::WinHttp, &worker)
        });
        let mut logged = None;
        while !handle.is_finished() {
            let step = match job.phase() {
                Phase::Downloading { done, total } => {
                    Some(("downloading", done * 10 / total.max(1)))
                }
                Phase::Unpacking { done, total } => Some(("unpacking", done * 10 / total.max(1))),
                _ => None,
            };
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
                let text =
                    fs::read_to_string(settings_path).unwrap_or_else(|_| config::TEMPLATE.into());
                let text = config::set_key(&text, "hd_textures", "on");
                let text = config::set_key(&text, "hd_textures_folder", &dir.display().to_string());
                if let Some(parent) = settings_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(settings_path, text).map_err(|e| e.to_string())?;
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

    pub fn run() -> Result<u32, String> {
        log_line(concat!("OutRunLauncher ", env!("CARGO_PKG_VERSION")));
        let settings_path = settings_path();
        let mut cli: Vec<String> = std::env::args().skip(1).collect();
        // Non-short-circuiting, so all three spellings are removed.
        let help =
            take_flag(&mut cli, "--help") | take_flag(&mut cli, "-h") | take_flag(&mut cli, "/?");
        if help {
            win32::message("OutRun 2006 launcher", config::USAGE, false);
            return Ok(0);
        }
        let settings_only = take_flag(&mut cli, "--settings");
        let linux = take_flag(&mut cli, "--linux");
        let no_window = take_flag(&mut cli, "--no-window");
        let check_only = take_flag(&mut cli, "--check");
        if take_flag(&mut cli, "--download-hd-pack") {
            return download_hd_pack(&settings_path);
        }
        if settings_only || (cli.is_empty() && !no_window && !check_only) {
            if win32::is_wine() {
                match win32::prefer_gpu_opengl() {
                    Ok(true) => log_line("Wine settings window: OpenGL through GLX (UseEGL=N)"),
                    Ok(false) => {}
                    Err(error) => log_line(&format!("warning: {error}")),
                }
            }
            let text =
                fs::read_to_string(&settings_path).unwrap_or_else(|_| config::TEMPLATE.into());
            match ui::show(&settings_path, &home(), &text, linux) {
                Ok(ui::Outcome::Play) => {
                    log_line("settings window: play");
                    if settings_only {
                        return Ok(0);
                    }
                }
                Ok(ui::Outcome::Closed) => {
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

        let existing = fs::read_to_string(&settings_path).ok();
        let file_text = existing.as_deref().unwrap_or(config::TEMPLATE);
        let (saved, warnings) = Settings::parse_file(file_text);
        for warning in warnings {
            log_line(&format!("warning: {warning}"));
        }
        let settings = saved.clone().merged(Settings::parse_args(cli)?);
        let game = find_game(&settings)?;
        let exe_path = game.join(EXE);
        let exe_bytes = fs::read(&exe_path)
            .map_err(|error| format!("cannot read {}: {error}", exe_path.display()))?;
        let bundled =
            launcher_dir().and_then(|dir| fs::read(dir.join(tweaks::TWEAKS_DIR).join(EXE)).ok());
        let detected = edition::identify_with_bundled(&exe_bytes, bundled.as_deref()).map_err(|digest| {
            format!(
                "This OR2006C2C.exe is a version the launcher doesn't know.\n\nSHA-256: {}\n\nPlease report it at https://github.com/akitaonrails/rea-outrun-2006/issues with where your copy comes from (Steam, DVD, which repack).",
                edition::to_hex(&digest)
            )
        })?;
        log_line(&format!("game: {} ({})", game.display(), detected.name));
        if check_only {
            win32::message(
                "OutRun 2006 launcher",
                &format!(
                    "Ready to play.\n\nGame folder: {}\nVersion: {}",
                    game.display(),
                    detected.name
                ),
                false,
            );
            return Ok(0);
        }

        let runtime_dir = settings
            .runtime
            .clone()
            .unwrap_or_else(|| home().join("OutRun2006"));
        runtime::check_separate(&game, &runtime_dir)?;
        let copied = runtime::sync(&game, &runtime_dir)
            .map_err(|error| format!("cannot prepare {}: {error}", runtime_dir.display()))?;
        log_line(&format!(
            "runtime: {} ({copied} files copied)",
            runtime_dir.display()
        ));

        // The player's saves live outside the runtime copy; the game reads
        // and writes its own SaveGame folder, kept in step before and
        // after.
        let saves = settings.saves.clone().unwrap_or_else(default_saves);
        let rt_save = runtime::save_dir(&runtime_dir)
            .map_err(|error| format!("cannot prepare saves: {error}"))?;
        if !saves.is_dir() {
            // First run with this folder: start it from what the game has.
            let n = runtime::mirror(&rt_save, &saves)
                .map_err(|error| format!("cannot create {}: {error}", saves.display()))?;
            log_line(&format!("saves: created {} ({n} files)", saves.display()));
        }
        let n = runtime::mirror(&saves, &rt_save)
            .map_err(|error| format!("cannot read saves from {}: {error}", saves.display()))?;
        log_line(&format!(
            "saves: {n} files updated from {}",
            saves.display()
        ));

        let wine = win32::is_wine();
        tweaks::install(
            &runtime_dir,
            &launcher_dir().unwrap_or_default(),
            wine,
            &settings,
            &mut |line| log_line(line),
        )?;
        if wine {
            match win32::wine_controller_baseline() {
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
            fs::write(&settings_path, text)
                .map_err(|error| format!("cannot save {}: {error}", settings_path.display()))?;
        }

        if !wine {
            match win32::prefer_fast_gpu(&runtime_dir.join(EXE)) {
                Ok(true) => log_line("Windows: high-performance GPU for the game"),
                Ok(false) => {}
                Err(error) => log_line(&format!("warning: {error}")),
            }
        } else {
            log_line(&format!("wine: DLL overrides {}", wine_overrides()));
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
        win32::stop_rumble();
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
        match runtime::mirror(&rt_save, &saves) {
            Ok(n) if n > 0 => log_line(&format!("saves: {n} files written back")),
            Ok(_) => {}
            Err(error) => log_line(&format!(
                "warning: saves not written back to {}: {error}",
                saves.display()
            )),
        }
        Ok(code)
    }
}
