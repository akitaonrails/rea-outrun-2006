//! One launcher at a time (SDK STANDARDS, "Clean exit, single instance").
//!
//! The launcher holds a named mutex (`rea_launcher_win32::instance`) from
//! the moment it starts until the game it started has ended, so a second
//! start can neither open a second window nor start a second game from
//! the same runtime copy: it brings the first launcher's window forward
//! (exit 0), or says the game is running and exits 5
//! (`rea_launcher_win32::instance::SecondStart`, found by
//! `Window::Own(window::TITLE)`). Play is also refused while the game from
//! the runtime copy still runs (a launcher killed while it played).
//! `--help` and a bad command line answer without it. An action
//! (`--check`, `--download-hd-pack`: the SDK's `instance::is_action`)
//! takes it like every run and is refused with exit 5 while the window
//! is open (STANDARDS, "Action flags and one instance"). On Linux,
//! `or-play`'s session lock does the same job.

use rea_launcher_win32::instance::SecondStart;
use rea_launcher_win32::report::Run;

/// The instance name (`Local\rea-launcher-rea-outrun-2006`).
pub const NAME: &str = "rea-outrun-2006";

/// Shown when a second start finds no launcher window to bring forward
/// (`SecondStart::Busy`): the first launcher is waiting for the game, or
/// is still starting, or runs without a window; also when the game from
/// this runtime copy still runs (`process::find_running`).
pub const GAME_RUNNING: &str =
    "OutRun 2006 or its launcher is already running. Quit it first, then start the launcher again.";

/// What `--download-hd-pack` says when the settings window is open
/// (`SecondStart::WindowOpen`): that window may be downloading the same
/// pack right now.
pub const WINDOW_OPEN: &str = "The launcher window is open: download the pack from its HD Pack tab, or close the window and run --download-hd-pack again.";

/// What another action (`--check`) says when the settings window is open:
/// the family's words (`SecondStart::message`).
pub const WINDOW_OPEN_CHECK: &str =
    "The OutRun 2006 launcher window is open. Use it there, or close it and try again.";

/// The switches whose output is their result: a command for the SDK's
/// audience rule (`--help` is one too, read before the switches).
pub const COMMANDS: &[&str] = &["--check", "--download-hd-pack"];

/// The family flags of a run with these (parsed) switches, for
/// `rea_launcher_win32::report::audience`. Never decide who is watching
/// from `--no-window` or `--linux` alone: or-play starts the game run with
/// `--no-window` for a player in front of the screen.
pub fn run(switches: &[String]) -> Run {
    Run::from_args(switches, COMMANDS)
}

/// What to tell the player about a second start (`switches`: its parsed
/// switches), in OutRun's words;
/// `None` when the running launcher's window came forward. The exit code
/// is the SDK's (`SecondStart::exit_code`: 0, or 5 when refused).
pub fn message(second: SecondStart, switches: &[String]) -> Option<&'static str> {
    match second {
        SecondStart::Focused => None,
        SecondStart::Busy => Some(GAME_RUNNING),
        SecondStart::WindowOpen if switches.iter().any(|s| s == "--download-hd-pack") => {
            Some(WINDOW_OPEN)
        }
        SecondStart::WindowOpen => Some(WINDOW_OPEN_CHECK),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher_win32::instance::ALREADY_RUNNING;

    /// The second start's `action`, as `main` computes it.
    fn is_action(list: &[String]) -> bool {
        rea_launcher_win32::instance::is_action(list, &[])
    }
    use rea_launcher_win32::report::Audience;

    fn switches(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_second_start_raises_the_window_or_says_the_game_runs() {
        let second = SecondStart::decide(false, || unreachable!(), || true);
        assert_eq!(second, SecondStart::Focused);
        assert_eq!((message(second, &[]), second.exit_code()), (None, 0));
        // No window: refused with the family's 5, also for or-play's
        // --settings (the helper says "nothing started" for a 5).
        let second = SecondStart::decide(false, || unreachable!(), || false);
        assert_eq!(message(second, &[]), Some(GAME_RUNNING));
        assert_eq!(second.exit_code(), ALREADY_RUNNING);
    }

    #[test]
    fn the_message_does_not_claim_the_game_is_what_runs() {
        // No window yet can also be a launcher still starting, or one
        // running --no-window or --download-hd-pack.
        assert!(GAME_RUNNING.starts_with("OutRun 2006 or its launcher is already running"));
    }

    #[test]
    fn download_hd_pack_never_exits_0_while_another_launcher_runs() {
        let list = switches(&["--download-hd-pack"]);
        let action = is_action(&list);
        assert!(action);
        // The window is open: say to use it, don't raise it, fail.
        let second = SecondStart::decide(action, || true, || unreachable!());
        assert_eq!(second, SecondStart::WindowOpen);
        assert_eq!(message(second, &list), Some(WINDOW_OPEN));
        assert_eq!(second.exit_code(), ALREADY_RUNNING);
        // The game runs: say so, fail.
        let second = SecondStart::decide(action, || false, || unreachable!());
        assert_eq!(message(second, &list), Some(GAME_RUNNING));
        assert_eq!(second.exit_code(), ALREADY_RUNNING);
        // Other runs are no action.
        assert!(!is_action(&switches(&["--linux", "--no-window"])));
        assert!(!is_action(&switches(&["--settings", "--linux"])));
    }

    /// Windows 10 test (rea-launcher-sdk#1): `--check` with the window
    /// open ran anyway. It is an action like the download: refused with
    /// exit 5 and the "window is open" message, the window left alone.
    #[test]
    fn check_is_refused_while_the_window_is_open() {
        for list in [&["--check"][..], &["--check", "--linux"]] {
            let list = switches(list);
            let action = is_action(&list);
            assert!(action, "{list:?}");
            let second = SecondStart::decide(action, || true, || unreachable!());
            assert_eq!(second, SecondStart::WindowOpen);
            assert_eq!(second.exit_code(), ALREADY_RUNNING);
            let text = message(second, &list).unwrap();
            assert!(text.contains("window is open"), "{text}");
            assert!(!text.contains("--download-hd-pack"), "{text}");
            assert_eq!(Some(text.to_string()), second.message("OutRun 2006"));
            // The game runs: refused too.
            let second = SecondStart::decide(action, || false, || unreachable!());
            assert_eq!(message(second, &list), Some(GAME_RUNNING));
            assert_eq!(second.exit_code(), ALREADY_RUNNING);
        }
    }

    /// Who sees a message is the SDK's rule (`report::Audience`), for the
    /// command lines or-play, the AppImage, Proton and a player use.
    #[test]
    fn the_helpers_runs_have_the_sdks_audience() {
        let watching =
            |list: &[&str], wine: bool| Audience::of(run(&switches(list)), wine).interactive;
        // or-play's game run (REA_PLAY_ARGS) under Wine: a player is in
        // front of the screen and nobody reads stdout, so errors are
        // dialogs.
        assert!(watching(&["--no-window"], true));
        assert!(watching(&["--no-window", "--linux"], true));
        // Proton or a double-click: the window, then the game.
        assert!(watching(&[], true));
        assert!(watching(&[], false));
        // or-play's settings window shows its own messages; or-play reads
        // --check's output and exit code itself.
        assert!(!watching(&["--settings", "--linux"], true));
        assert!(!watching(&["--check", "--linux"], true));
        // A script on Windows.
        assert!(!watching(&["--no-window"], false));
        // Commands answer on the terminal.
        assert!(run(&switches(&["--check"])).command);
        assert!(run(&switches(&["--download-hd-pack", "--no-window"])).command);
        assert!(!run(&switches(&["--no-window", "--linux"])).command);
    }

    /// Under Wine (`sdk/scripts/wine-test`): while one launcher holds the
    /// instance, a second start is refused, finds no window of this
    /// launcher to raise (headless: no settings window), and so says the
    /// game is running; once the first ends, the next start gets it.
    #[cfg(windows)]
    #[test]
    fn a_second_launcher_is_refused_while_the_first_runs() {
        use rea_launcher_win32::instance::{self, Window};
        let name = format!("{NAME}-test-{}", std::process::id());
        let first = instance::single(&name).expect("the first launcher");
        let second = instance::single(&name).expect_err("a second launcher");
        let window = Window::Own("rea-outrun-2006 no such window");
        let raised = second.second_start(false, window);
        assert_eq!(raised, SecondStart::Busy);
        assert_eq!(raised.exit_code(), ALREADY_RUNNING);
        let action = second.second_start(true, window);
        assert_eq!(action, SecondStart::Busy);
        drop(first);
        drop(instance::single(&name).expect("free once the first ended"));
    }

    /// Under Wine: no game runs from a runtime copy that doesn't exist,
    /// so Play goes ahead.
    #[cfg(windows)]
    #[test]
    fn no_game_runs_from_an_unused_runtime_copy() {
        let exe = std::env::temp_dir()
            .join(format!("{NAME}-no-runtime-{}", std::process::id()))
            .join(crate::edition::EXE);
        assert!(rea_launcher_win32::process::find_running(&exe).is_empty());
    }
}
