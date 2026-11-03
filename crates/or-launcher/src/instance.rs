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
//! `--help` and `--check` only read and run alongside. On Linux,
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

/// Whether a run with these switches is an action that does its work
/// without the window.
pub fn is_action(switches: &[String]) -> bool {
    switches.iter().any(|switch| switch == "--download-hd-pack")
}

/// Whether a run with these switches takes the instance: every run but
/// `--check`, which only reads.
pub fn needs_instance(switches: &[String]) -> bool {
    !switches.iter().any(|switch| switch == "--check")
}

/// What to tell the player about a second start, in OutRun's words;
/// `None` when the running launcher's window came forward. The exit code
/// is the SDK's (`SecondStart::exit_code`: 0, or 5 when refused).
pub fn message(second: SecondStart) -> Option<&'static str> {
    match second {
        SecondStart::Focused => None,
        SecondStart::Busy => Some(GAME_RUNNING),
        SecondStart::WindowOpen => Some(WINDOW_OPEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher_win32::instance::ALREADY_RUNNING;
    use rea_launcher_win32::report::Audience;

    fn switches(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_second_start_raises_the_window_or_says_the_game_runs() {
        let second = SecondStart::decide(false, || unreachable!(), || true);
        assert_eq!(second, SecondStart::Focused);
        assert_eq!((message(second), second.exit_code()), (None, 0));
        // No window: refused with the family's 5, also for or-play's
        // --settings (the helper says "nothing started" for a 5).
        let second = SecondStart::decide(false, || unreachable!(), || false);
        assert_eq!(message(second), Some(GAME_RUNNING));
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
        let action = is_action(&switches(&["--download-hd-pack"]));
        assert!(action && needs_instance(&switches(&["--download-hd-pack"])));
        // The window is open: say to use it, don't raise it, fail.
        let second = SecondStart::decide(action, || true, || unreachable!());
        assert_eq!(second, SecondStart::WindowOpen);
        assert_eq!(message(second), Some(WINDOW_OPEN));
        assert_eq!(second.exit_code(), ALREADY_RUNNING);
        // The game runs: say so, fail.
        let second = SecondStart::decide(action, || false, || unreachable!());
        assert_eq!(message(second), Some(GAME_RUNNING));
        assert_eq!(second.exit_code(), ALREADY_RUNNING);
        // Other runs are no action; --check takes no instance.
        assert!(!is_action(&switches(&["--linux", "--no-window"])));
        assert!(!needs_instance(&switches(&["--check"])));
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
}
