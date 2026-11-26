//! The in-game wheel layer (the SDK's `rea_dinput.dll`) on a start:
//! whether the game is shown the virtual wheel, and what the layer gets.
//!
//! `rea_dinput.dll` is imported into the private runtime copy of
//! `OR2006C2C.exe` alongside OutRun2006Tweaks' `dinput8.dll`. When a wheel is
//! active, OutRun2006Tweaks' SDL pad hook (`UseNewInput`) is turned off so
//! `OR2006C2C.exe` reads DirectInput through `rea_dinput.dll` while keeping
//! all of Tweaks' graphics, widescreen, framerate, and HD-pack fixes.

use rea_launcher::wheel::dinput::{HEADER, Mode, Profile, Setup};
use rea_launcher::wheel::rig::{self, Device};

/// The layer's DLL, next to the runtime's `OR2006C2C.exe`, which imports it
/// (`rea_dinput::import_into`).
pub const DLL: &str = rea_dinput::DLL;

/// The layer's on/off for this start, in the game's environment.
pub const MODE_ENV: &str = rea_dinput::MODE_ENV;

/// Whether the game is shown the virtual wheel: `On` and `Off` as set;
/// `Auto` asks `attached` for the controllers plugged in now and shows it
/// when a wheel is among them.
pub fn shown(setup: &Setup, attached: impl FnOnce() -> Vec<Device>) -> bool {
    match setup.mode {
        Mode::Auto => rig::shows_virtual_wheel(setup, &attached()),
        mode => mode == Mode::On,
    }
}

/// [`MODE_ENV`]'s value for this start.
pub fn mode_value(shown: bool) -> &'static str {
    if shown { "on" } else { "off" }
}

/// The runtime's `rea_wheel.ini`: this start's `setup` and any saved
/// calibration (`""`: none yet, the layer guesses one).
pub fn runtime_ini(setup: &Setup, calibration: &str) -> String {
    rig::save("", setup, Profile::from_cfg(calibration).as_ref(), HEADER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher::wheel::{DeviceId, DeviceState, Kind};

    fn device(kind: Kind) -> Device {
        let state = DeviceState {
            axes: vec![0; 8],
            buttons: vec![false; 12],
        };
        Device {
            id: DeviceId {
                name: "Logitech G923".into(),
                vid: 0x046d,
                pid: 0xc266,
                guid: "{g923}".into(),
            },
            kind,
            axes: 8,
            buttons: 24,
            ffb: kind == Kind::Wheel,
            initial: state.clone(),
            state,
        }
    }

    fn setup(mode: Mode) -> Setup {
        Setup {
            mode,
            ..Setup::default()
        }
    }

    #[test]
    fn auto_shows_the_wheel_only_with_a_wheel_attached() {
        assert!(shown(&setup(Mode::Auto), || vec![device(Kind::Wheel)]));
        assert!(!shown(&setup(Mode::Auto), || vec![device(Kind::Pad)]));
        assert!(!shown(&setup(Mode::Auto), Vec::new));
    }

    #[test]
    fn on_and_off_never_list_the_controllers() {
        let unlisted = || -> Vec<Device> { panic!("listed") };
        assert!(shown(&setup(Mode::On), unlisted));
        assert!(!shown(&setup(Mode::Off), unlisted));
        assert_eq!((mode_value(true), mode_value(false)), ("on", "off"));
    }

    #[test]
    fn runtime_ini_round_trips_setup_and_profile() {
        let g923 = device(Kind::Wheel);
        let profile = Profile::guess(g923.id, &g923.initial, None);
        let calibration = profile.to_cfg("# mine\n", "");
        let start = setup(Mode::On);
        let text = runtime_ini(&start, &calibration);
        assert!(text.starts_with(&format!("# {HEADER}\n")), "{text}");
        assert_eq!(Setup::from_cfg(&text), start);
        assert_eq!(Profile::from_cfg(&text), Some(profile), "{text}");
    }
}
