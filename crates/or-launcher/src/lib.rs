//! OutRun 2006 launcher: runs the user's own copy of OutRun 2006: Coast 2
//! Coast on modern Windows and under Proton/Wine. The host-testable logic
//! lives here; the Windows program (`main.rs`, `ui.rs`, `win32.rs`) uses it.

pub mod config;
pub mod edition;
pub mod form;
pub mod hdpack;
pub mod ini;
pub mod runtime;
pub mod soundtrack;
pub mod tweaks;

/// Whether the environment says Wine: the variables Wine, Proton and
/// `or-play` set. A fallback for Wine builds that hide `wine_get_version`;
/// `is_set` reads one variable.
pub fn wine_env(is_set: impl Fn(&str) -> bool) -> bool {
    ["WINEPREFIX", "WINELOADER", "STEAM_COMPAT_DATA_PATH"]
        .into_iter()
        .any(is_set)
}

#[cfg(test)]
mod tests {
    #[test]
    fn wine_is_recognised_from_its_environment() {
        assert!(super::wine_env(|name| name == "WINEPREFIX"));
        assert!(super::wine_env(|name| name == "STEAM_COMPAT_DATA_PATH"));
        assert!(!super::wine_env(|name| name == "USERPROFILE"));
    }
}
