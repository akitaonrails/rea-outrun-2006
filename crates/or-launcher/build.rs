//! The Windows executable's version information, icon and manifest
//! (rea-launcher-build): a named, versioned program with the launcher's
//! icon, instead of an anonymous one that antivirus heuristics distrust.
fn main() {
    rea_launcher_build::Resources::new(
        "OutRun 2006 Launcher",
        "Launcher for OutRun 2006: Coast 2 Coast on modern systems",
    )
    .company("AkitaOnRails")
    .file_name("OutRunLauncher.exe")
    .icon("../../packaging/appimage/outrun-launcher.png")
    .embed()
    .unwrap();
}
