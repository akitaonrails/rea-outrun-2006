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
