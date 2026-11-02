//! OutRun 2006 launcher: runs the user's own copy of OutRun 2006: Coast 2
//! Coast on modern Windows and under Proton/Wine. The host-testable logic
//! lives here; the settings window (`window`) draws on any host; the Windows program
//! (`main.rs`, `ui.rs`) gives it the system's dialogs, pad and downloads.

pub mod config;
pub mod edition;
pub mod form;
pub mod hdpack;
pub mod plan;
pub mod runtime;
pub mod soundtrack;
pub mod tweaks;
pub mod window;
