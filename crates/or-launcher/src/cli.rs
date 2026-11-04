//! The command line, read once at start-up: what it asks for and who sees
//! the messages of the run (the SDK's `report::Run`, for
//! `rea_launcher_win32::report::audience`).

use rea_launcher_win32::report::Run;

use crate::config::Settings;
use crate::instance;

/// What the command line asks for.
#[derive(Debug)]
pub enum Ask {
    /// `--help`, `-h` or `/?`: the usage, exit 0.
    Help,
    /// A command line that fails to parse (an unknown flag, a bad value):
    /// the message, then exit [`BAD_COMMAND_LINE`].
    Bad(String),
    /// A run with these settings from flags and these switches.
    Go(Settings, Vec<String>),
}

/// The family's exit code for a bad command line (1 is a refusal).
pub const BAD_COMMAND_LINE: u32 = rea_launcher_win32::report::BAD_COMMAND_LINE as u32;

/// Reads `cli` (the arguments, program name excluded). The [`Run`] is the
/// SDK's: a bad command line is reported like a command
/// (`Run::bad_command_line`: the terminal, a dialog only when nobody sees
/// it), never like a play run, whose modal box would hang a script with a
/// typo.
pub fn read(cli: Vec<String>) -> (Ask, Run) {
    if cli
        .iter()
        .any(|a| matches!(a.as_str(), "--help" | "-h" | "/?"))
    {
        let run = Run::from_args(&cli, instance::COMMANDS);
        return (Ask::Help, run);
    }
    match Settings::parse_args(cli.clone()) {
        Ok((args, switches)) => {
            let run = instance::run(&switches);
            (Ask::Go(args, switches), run)
        }
        Err(message) => (Ask::Bad(message), Run::bad_command_line(&cli)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher_win32::report::{Audience, Mode, Terminal, route};

    fn read_list(list: &[&str]) -> (Ask, Run) {
        read(list.iter().map(|s| s.to_string()).collect())
    }

    /// The mode on native Windows (`wine = false`) and under Wine.
    fn modes(run: Run) -> [Mode; 2] {
        [
            Audience::of(run, false).mode(),
            Audience::of(run, true).mode(),
        ]
    }

    /// Windows 10 test (rea-launcher-sdk#1): a bad command line from a
    /// terminal must print and exit 2, never wait on a modal box.
    #[test]
    fn a_bad_command_line_is_a_command_that_exits_2() {
        for list in [
            &["--bogus-flag"][..],
            &["--look", "invalid"],
            &["--no-window", "--look", "invalid"],
            &["--resolution", "abc"],
        ] {
            let (ask, run) = read_list(list);
            assert!(matches!(ask, Ask::Bad(_)), "{list:?}: {ask:?}");
            assert!(run.command, "{list:?}");
            for mode in modes(run) {
                assert!(matches!(mode, Mode::Command { .. }), "{list:?}: {mode:?}");
                // From PowerShell: printed, no dialog.
                let terminal = Terminal {
                    attached: true,
                    redirected: false,
                    wine: false,
                };
                let to = route(mode, terminal);
                assert!(to.print && !to.dialog, "{list:?}: {to:?}");
            }
        }
        assert_eq!(BAD_COMMAND_LINE, 2);
    }

    #[test]
    fn help_is_a_command_and_a_play_run_is_not() {
        let (ask, run) = read_list(&["--help"]);
        assert!(matches!(ask, Ask::Help) && run.command);
        let (ask, run) = read_list(&["--no-window"]);
        assert!(matches!(ask, Ask::Go(..)) && !run.command);
        assert!(modes(run).iter().all(|m| matches!(m, Mode::Play { .. })));
    }
}
