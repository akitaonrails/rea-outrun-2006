//! Portable mode (#6) on the real OutRunLauncher.exe, under
//! `sdk/scripts/wine-test` (and on Windows): a copy of the exe in a scratch
//! folder, with and without `portable.txt` beside it. Every run passes
//! `--linux`, so no message becomes a dialog that would wait forever in a
//! headless test; a run that hangs is killed after [`LIMIT`] and fails.

#[cfg(windows)]
mod program {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const LIMIT: Duration = Duration::from_secs(60);

    /// A scratch folder with a hostile name (spaces, apostrophe, accents).
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("or-portable-{name}-{}", std::process::id()))
            .join("Jogos d'Ação");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Runs `exe --check --linux` with `local` as `%LOCALAPPDATA%`, from
    /// another folder, so the launcher can't lean on the current one.
    fn check(exe: &Path, local: &Path, cwd: &Path) {
        let mut child = Command::new(exe)
            .args(["--check", "--linux"])
            .env("LOCALAPPDATA", local)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let started = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if started.elapsed() > LIMIT {
                let _ = child.kill();
                let _ = child.wait();
                panic!("--check did not exit (a dialog waiting for OK?)");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn portable_txt_keeps_everything_beside_the_launcher() {
        let root = scratch("marker");
        let local = root.join("AppData Local");
        let elsewhere = root.join("elsewhere");
        let launcher = root.join("OutRun portable");
        let game = root.join("OutRun 2006");
        for dir in [&local, &elsewhere, &launcher, &game] {
            fs::create_dir_all(dir).unwrap();
        }
        let exe = launcher.join("OutRunLauncher.exe");
        fs::copy(env!("CARGO_BIN_EXE_OutRunLauncher"), &exe).unwrap();

        // Without portable.txt: the usual home, nothing beside the exe.
        check(&exe, &local, &elsewhere);
        assert!(local.join("OutRunLauncher").join("launcher.log").is_file());
        assert!(!launcher.join("data").exists());
        fs::remove_dir_all(local.join("OutRunLauncher")).unwrap();

        // With it: the data folder beside the exe holds the log and
        // launcher.cfg, whose game folder becomes relative to the launcher.
        fs::write(launcher.join("portable.txt"), b"").unwrap();
        let data = launcher.join("data");
        fs::create_dir_all(&data).unwrap();
        let cfg = data.join("launcher.cfg");
        fs::write(&cfg, format!("game = {}\n", game.display())).unwrap();
        check(&exe, &local, &elsewhere);
        let log = fs::read_to_string(data.join("launcher.log")).unwrap();
        assert!(log.contains("portable: everything in"), "{log}");
        assert!(
            log.contains(r"..\OutRun 2006\OR2006C2C.exe"),
            "the game is looked for through the relative folder: {log}"
        );
        assert_eq!(
            fs::read_to_string(&cfg).unwrap(),
            "game = ..\\OutRun 2006\n"
        );
        assert!(
            !local.join("OutRunLauncher").exists(),
            "nothing under %LOCALAPPDATA%"
        );
        assert!(fs::read_dir(&elsewhere).unwrap().next().is_none());
        let _ = fs::remove_dir_all(root.parent().unwrap());
    }
}
