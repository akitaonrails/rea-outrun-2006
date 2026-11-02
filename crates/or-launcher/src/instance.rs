//! One launcher at a time (SDK STANDARDS, "Clean exit, single instance").
//!
//! The launcher holds a named mutex (`rea_launcher_win32::instance`) from
//! the moment it starts until the game it started has ended, so a second
//! start can neither open a second window nor start a second game from
//! the same runtime copy: it brings the first launcher's window forward,
//! or says the game is running. `--help` and `--check` only read and run
//! alongside. An action flag (`--download-hd-pack`) never exits 0 as if
//! its work were done: it says why it stopped and fails (SDK v0.3.2
//! "action flags"). On Linux, `or-play`'s session lock does the same job.

/// The instance name (`Local\rea-launcher-rea-outrun-2006`).
pub const NAME: &str = "rea-outrun-2006";

/// Exit code of an action flag that another launcher kept from running.
pub const ALREADY_RUNNING: u32 = 5;

/// Shown when a second start finds no launcher window to bring forward:
/// the first launcher is waiting for the game, or is still starting, or
/// runs without a window.
pub const GAME_RUNNING: &str =
    "OutRun 2006 or its launcher is already running. Quit it first, then start the launcher again.";

/// What `--download-hd-pack` says when the settings window is open: that
/// window may be downloading the same pack right now.
pub const WINDOW_OPEN: &str = "The launcher window is open: download the pack from its HD Pack tab, or close the window and run --download-hd-pack again.";

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

/// What a second start does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondStart {
    /// The running launcher's window came to the front; nothing to say.
    WindowRaised,
    /// No launcher window: the game (or a window-less run) is in
    /// progress. Say so ([`GAME_RUNNING`]).
    GameRunning,
    /// An action flag found the settings window open: say to use it
    /// ([`WINDOW_OPEN`]) and leave it where it is.
    WindowOpen,
}

impl SecondStart {
    /// `window_found` is the SDK's `AlreadyRunning::is_window_found` and
    /// `bring_to_front` its `AlreadyRunning::bring_to_front` (fakes in
    /// tests). An action never brings the window forward.
    pub fn of(
        action: bool,
        window_found: impl FnOnce() -> bool,
        bring_to_front: impl FnOnce() -> bool,
    ) -> SecondStart {
        if action {
            if window_found() {
                SecondStart::WindowOpen
            } else {
                SecondStart::GameRunning
            }
        } else if bring_to_front() {
            SecondStart::WindowRaised
        } else {
            SecondStart::GameRunning
        }
    }

    /// The message for the player, if any.
    pub fn message(self) -> Option<&'static str> {
        match self {
            SecondStart::WindowRaised => None,
            SecondStart::GameRunning => Some(GAME_RUNNING),
            SecondStart::WindowOpen => Some(WINDOW_OPEN),
        }
    }

    /// The exit code: an action always fails ([`ALREADY_RUNNING`]); other
    /// runs exit 0, or `closed` (or-play's "nothing was started") for
    /// `--settings`.
    pub fn exit_code(self, action: bool, settings_only: bool, closed: u32) -> u32 {
        if action {
            ALREADY_RUNNING
        } else if settings_only {
            closed
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn switches(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_second_start_raises_the_window_or_says_the_game_runs() {
        let second = SecondStart::of(false, || unreachable!(), || true);
        assert_eq!(second, SecondStart::WindowRaised);
        assert_eq!(
            (second.message(), second.exit_code(false, false, 3)),
            (None, 0)
        );
        let second = SecondStart::of(false, || unreachable!(), || false);
        assert_eq!(second.message(), Some(GAME_RUNNING));
        assert_eq!(second.exit_code(false, true, 3), 3);
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
        let second = SecondStart::of(action, || true, || unreachable!());
        assert_eq!(second, SecondStart::WindowOpen);
        assert_eq!(second.message(), Some(WINDOW_OPEN));
        assert_eq!(second.exit_code(action, false, 3), ALREADY_RUNNING);
        // The game runs: say so, fail.
        let second = SecondStart::of(action, || false, || unreachable!());
        assert_eq!(second.message(), Some(GAME_RUNNING));
        assert_eq!(second.exit_code(action, false, 3), ALREADY_RUNNING);
        // Other runs are no action; --check takes no instance.
        assert!(!is_action(&switches(&["--linux", "--no-window"])));
        assert!(!needs_instance(&switches(&["--check"])));
    }

    /// Under Wine (`sdk/scripts/wine-test`): while one launcher holds the
    /// instance, a second start is refused, finds no window to raise
    /// (headless: no settings window), and so says the game is running;
    /// once the first ends, the next start gets it.
    #[cfg(windows)]
    #[test]
    fn a_second_launcher_is_refused_while_the_first_runs() {
        use rea_launcher_win32::instance::{self, Window};
        let name = format!("{NAME}-test-{}", std::process::id());
        let first = instance::single(&name).expect("the first launcher");
        let second = instance::single(&name).expect_err("a second launcher");
        let window = Window::Title("rea-outrun-2006 no such window");
        let raised = SecondStart::of(false, || unreachable!(), || second.bring_to_front(window));
        assert_eq!(raised, SecondStart::GameRunning);
        let action = SecondStart::of(true, || second.is_window_found(window), || unreachable!());
        assert_eq!(action.exit_code(true, false, 3), ALREADY_RUNNING);
        drop(first);
        drop(instance::single(&name).expect("free once the first ended"));
    }
}
