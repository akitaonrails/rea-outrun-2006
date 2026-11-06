//! Microsoft's DirectX 9 extension libraries (`d3dx9_24.dll` …
//! `d3dx9_43.dll`, `D3DCompiler_*.dll`, `xinput1_1.dll` … `xinput1_3.dll`),
//! fetched on the player's request.
//!
//! `OR2006C2C.exe` statically links `d3dx9_29.dll`; Windows 10/11 do not
//! ship it, and the game's own installer put it in with Microsoft's DirectX
//! runtime.
//!
//! It cannot be bundled: the only package holding it, Microsoft's *DirectX
//! End-User Runtime* (June 2010), is licensed to the person who installs it
//! ("you may not publish the software for others to copy", its EULA §2),
//! and the DirectX SDK's redistribution rights require Microsoft's
//! DirectSetup installer. So the window has a Download button (and
//! `--download-directx`): the launcher fetches that package from Microsoft,
//! pinned by SHA-256, takes the 32-bit DLLs out of its cabinets and keeps
//! them in its own folder; each launch puts them next to the game in the
//! private copy (through the component ledger, so a file of the player's is
//! never replaced).

use std::fs;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use rea_launcher::download::{self, Fetch, FreeSpace, Job, Phase, Pin};
use rea_launcher::ledger::{Ledger, Outcome};

/// Microsoft's June 2010 DirectX End-User Runtime (re-signed by Microsoft
/// since; this is the file download.microsoft.com serves today). Fields:
/// URL, SHA-256, size, the unpacked DLLs' size.
pub const PIN: &str = "https://download.microsoft.com/download/8/4/A/84A35BF1-DAFE-4AE8-82AF-AD2AE20B6B14/directx_Jun2010_redist.exe \
053f76dcbb28802e23341b6a787e3b0791c0fa5c8d4d011b1044172dbf89c73b 100275120 80000000";

pub fn pin() -> Pin {
    Pin::parse(PIN).expect("a valid pin")
}

/// Where the unpacked DLLs live under the launcher's home folder.
pub const ROOT_DIR: &str = "directx";

/// The ledger of what the launcher put into the private copy.
pub const LEDGER: &str = ".or-directx";

/// The row label in the settings window.
pub const LABEL: &str = "Microsoft DirectX 9 files (d3dx9_29.dll)";

/// The DLL `OR2006C2C.exe` needs on Windows that clean Windows 10 and 11
/// installs do not include out of the box.
pub const REQUIRED_DLLS: [&str; 1] = ["d3dx9_29.dll"];

/// The folder under `home` (`%LOCALAPPDATA%\OutRunLauncher`) that holds the
/// downloaded DLLs.
pub fn root(home: &Path) -> PathBuf {
    home.join(ROOT_DIR)
}

/// The unpacked DirectX folder when the download has completed.
pub fn installed(home: &Path) -> Option<PathBuf> {
    download::installed(&pin(), &root(home))
}

/// Which of [`REQUIRED_DLLS`] are missing from the game folder, the launcher's
/// downloaded DirectX folder (`directx_dir`), and Windows' 32-bit system
/// folders (`system_dirs`).
pub fn missing_required_dlls(
    game: Option<&Path>,
    directx_dir: Option<&Path>,
    system_dirs: &[PathBuf],
) -> Vec<&'static str> {
    REQUIRED_DLLS
        .into_iter()
        .filter(|dll| {
            let in_game =
                game.is_some_and(|dir| rea_launcher::paths::find_ignoring_case(dir, dll).is_some());
            let in_directx = directx_dir
                .is_some_and(|dir| rea_launcher::paths::find_ignoring_case(dir, dll).is_some());
            let in_system = system_dirs
                .iter()
                .any(|dir| rea_launcher::paths::find_ignoring_case(dir, dll).is_some());
            !(in_game || in_directx || in_system)
        })
        .collect()
}

/// Windows' 32-bit DLL search directories (`SysWOW64` on 64-bit Windows,
/// `System32` on 32-bit Windows).
pub fn windows_system_dirs() -> Vec<PathBuf> {
    let root = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    vec![root.join("SysWOW64"), root.join("System32")]
}

/// The 32-bit cabinets wanted from the package: every D3DX9, D3DCompiler
/// and XInput 1.x version (`OR2006C2C.exe` imports `d3dx9_29.dll`).
pub fn wanted_cab(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with("_x86.cab")
        && (lower.contains("_d3dx9_")
            || lower.contains("_d3dcompiler_")
            || lower.contains("_xinput_"))
}

/// The DLLs wanted from an inner cabinet.
fn wanted_dll(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".dll")
        && (lower.starts_with("d3dx9_")
            || lower.starts_with("d3dcompiler_")
            || lower.starts_with("xinput1_"))
        && !lower.contains(['/', '\\'])
}

fn cab_names<R: Read + io::Seek>(cabinet: &cab::Cabinet<R>) -> Vec<String> {
    cabinet
        .folder_entries()
        .flat_map(|folder| folder.file_entries().map(|file| file.name().to_string()))
        .collect()
}

/// The outer cabinet inside the self-extracting package: the first
/// `MSCF` signature that parses as a cabinet.
fn outer_cabinet(package: &[u8]) -> Option<cab::Cabinet<Cursor<&[u8]>>> {
    package
        .windows(4)
        .enumerate()
        .filter(|(_, window)| *window == b"MSCF")
        .find_map(|(at, _)| cab::Cabinet::new(Cursor::new(&package[at..])).ok())
}

/// The wanted DLLs of a package: `(file name, bytes)`, sorted by name.
pub fn extract(package: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut outer = outer_cabinet(package).ok_or("no cabinet in the DirectX package")?;
    let mut dlls = Vec::new();
    for name in cab_names(&outer).into_iter().filter(|n| wanted_cab(n)) {
        let mut inner_bytes = Vec::new();
        outer
            .read_file(&name)
            .and_then(|mut file| file.read_to_end(&mut inner_bytes))
            .map_err(|error| format!("{name}: {error}"))?;
        let mut inner = cab::Cabinet::new(Cursor::new(&inner_bytes[..]))
            .map_err(|error| format!("{name}: {error}"))?;
        for dll in cab_names(&inner).into_iter().filter(|n| wanted_dll(n)) {
            let mut bytes = Vec::new();
            inner
                .read_file(&dll)
                .and_then(|mut file| file.read_to_end(&mut bytes))
                .map_err(|error| format!("{name}/{dll}: {error}"))?;
            dlls.push((dll, bytes));
        }
    }
    if dlls.is_empty() {
        return Err("the DirectX package holds none of the D3DX9 files".into());
    }
    dlls.sort_by_key(|(name, _)| name.to_ascii_lowercase());
    Ok(dlls)
}

/// The extractor `download::install_with` runs on the package: the wanted
/// DLLs go into `into`, reported as unpacked bytes through `job`.
pub fn extract_into(archive: &Path, into: &Path, job: &Job) -> Result<(), String> {
    let package = fs::read(archive).map_err(|error| format!("{}: {error}", archive.display()))?;
    let dlls = extract(&package)?;
    let total = dlls.iter().map(|(_, bytes)| bytes.len() as u64).sum();
    let mut done = 0;
    job.set_phase(Phase::Unpacking { done, total });
    for (name, bytes) in &dlls {
        if job.cancelled() {
            return Err("cancelled".into());
        }
        let path = into.join(name);
        fs::write(&path, bytes).map_err(|error| format!("{}: {error}", path.display()))?;
        done += bytes.len() as u64;
        job.set_phase(Phase::Unpacking { done, total });
    }
    Ok(())
}

/// Downloads the package (resumable, verified) and unpacks the DLLs into
/// `download::installed(pin, root)` (SDK `install_with`, which deletes the
/// package afterwards). Runs on a worker thread; the window polls `job`.
pub fn install(pin: &Pin, root: &Path, free: FreeSpace, fetch: &dyn Fetch, job: &Job) -> Phase {
    if let Err(error) = fs::create_dir_all(root) {
        let phase = Phase::Failed(format!("{}: {error}", root.display()));
        job.set_phase(phase.clone());
        return phase;
    }
    download::install_with(pin, root, free, fetch, job, &extract_into)
}

/// The DLL names in an unpacked folder, sorted.
pub fn dlls(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| wanted_dll(name))
        .collect();
    names.sort_by_key(|name| name.to_ascii_lowercase());
    names
}

/// Puts the unpacked DLLs (`directx_dir`, or none) next to the game in the
/// private copy through the component ledger, and takes out the ones it put
/// there earlier that are no longer wanted.
pub fn for_launch(
    game: &Path,
    runtime: &Path,
    directx_dir: Option<&Path>,
) -> Result<Vec<String>, String> {
    let mut ledger = Ledger::load(game, runtime, LEDGER);
    let wanted = directx_dir.map(dlls).unwrap_or_default();
    ledger
        .batch(|ledger| {
            let mut notes = Vec::new();
            for name in &wanted {
                let bytes = fs::read(directx_dir.expect("listed from dir").join(name))?;
                let recorded = ledger.names().any(|owned| owned == name);
                if recorded && fs::read(runtime.join(name)).is_ok_and(|current| current == bytes) {
                    continue;
                }
                match ledger.install(name, &bytes, &[])? {
                    Outcome::NotOurs => notes.push(format!(
                        "{name}: the runtime's file is not the launcher's; left as it is"
                    )),
                    _ => notes.push(format!("{name}: installed")),
                }
            }
            let stale: Vec<String> = ledger
                .names()
                .filter(|name| !wanted.iter().any(|w| w.eq_ignore_ascii_case(name)))
                .map(str::to_string)
                .collect();
            for name in stale {
                if ledger.take_out(&name, &[])? == Outcome::Restored {
                    notes.push(format!("{name}: removed"));
                }
            }
            Ok(notes)
        })
        .map_err(|e| format!("DirectX 9 files: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("or-directx-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cabinet(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = cab::CabinetBuilder::new();
        let folder = builder.add_folder(cab::CompressionType::None);
        for (name, _) in files {
            folder.add_file(*name);
        }
        let mut writer = builder.build(Cursor::new(Vec::new())).unwrap();
        let mut index = 0;
        while let Some(mut file) = writer.next_file().unwrap() {
            io::Write::write_all(&mut file, files[index].1).unwrap();
            index += 1;
        }
        writer.finish().unwrap().into_inner()
    }

    fn package() -> Vec<u8> {
        let d3dx29 = cabinet(&[("d3dx9_29.dll", b"d3dx 29"), ("d3dx9_29_x86.inf", b"inf")]);
        let compiler = cabinet(&[("D3DCompiler_43.dll", b"compiler 43")]);
        let xinput = cabinet(&[("xinput1_3.dll", b"xinput 1.3")]);
        let x64 = cabinet(&[("d3dx9_29.dll", b"64-bit")]);
        let outer = cabinet(&[
            ("Feb2006_d3dx9_29_x86.cab", &d3dx29),
            ("Feb2006_d3dx9_29_x64.cab", &x64),
            ("Jun2010_D3DCompiler_43_x86.cab", &compiler),
            ("APR2007_xinput_x86.cab", &xinput),
            ("DXSETUP.exe", b"MZ setup"),
        ]);
        let mut package = b"MZ\x90\x00 self-extractor MSCF not a cabinet ".to_vec();
        package.extend(outer);
        package
    }

    #[test]
    fn pin_is_microsofts_package() {
        let p = pin();
        assert!(p.url.starts_with("https://download.microsoft.com/"));
        assert_eq!(p.file_name(), "directx_Jun2010_redist.exe");
        assert_eq!(p.size, 100_275_120);
    }

    #[test]
    fn extracts_32_bit_dlls_and_places_into_runtime() {
        let dir = temp("unpack");
        let dlls = extract(&package()).unwrap();
        let names: Vec<&str> = dlls.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            ["D3DCompiler_43.dll", "d3dx9_29.dll", "xinput1_3.dll"]
        );

        let unpacked = dir.join("unpacked");
        fs::create_dir_all(&unpacked).unwrap();
        for (name, bytes) in &dlls {
            fs::write(unpacked.join(name), bytes).unwrap();
        }

        let game = dir.join("game");
        let runtime = dir.join("runtime");
        fs::create_dir_all(&game).unwrap();
        fs::create_dir_all(&runtime).unwrap();

        let notes = for_launch(&game, &runtime, Some(&unpacked)).unwrap();
        assert_eq!(notes.len(), 3);
        assert_eq!(fs::read(runtime.join("d3dx9_29.dll")).unwrap(), b"d3dx 29");

        // Removing the download removes the launcher's copies from the runtime.
        let notes = for_launch(&game, &runtime, None).unwrap();
        assert_eq!(notes.len(), 3);
        assert!(!runtime.join("d3dx9_29.dll").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_required_dlls_checks_game_directx_and_system_dirs() {
        let dir = temp("missing");
        let game = dir.join("game");
        let dx = dir.join("dx");
        let sys = dir.join("sys");
        fs::create_dir_all(&game).unwrap();
        fs::create_dir_all(&dx).unwrap();
        fs::create_dir_all(&sys).unwrap();

        assert_eq!(
            missing_required_dlls(Some(&game), Some(&dx), std::slice::from_ref(&sys)),
            vec!["d3dx9_29.dll"]
        );

        fs::write(sys.join("D3DX9_29.DLL"), b"sys").unwrap();
        assert!(
            missing_required_dlls(Some(&game), Some(&dx), std::slice::from_ref(&sys)).is_empty()
        );

        fs::remove_dir_all(&dir).unwrap();
    }
}
