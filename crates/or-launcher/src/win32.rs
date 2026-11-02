//! Win32 calls: message boxes, the folder picker, screen and pad queries,
//! registry writes, and the Wine detections. All `unsafe` lives here.

use std::ffi::{OsStr, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD,
    REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey, RegCreateKeyExW, RegQueryValueExW,
    RegSetValueExW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MessageBoxW,
};

fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn message(title: &str, text: &str, error: bool) {
    let icon = if error {
        MB_ICONERROR
    } else {
        MB_ICONINFORMATION
    };
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(text).as_ptr(),
            wide(title).as_ptr(),
            MB_OK | icon,
        );
    }
}

pub fn screen_size() -> Option<(u32, u32)> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    let (width, height) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    (width > 0 && height > 0).then_some((width as u32, height as u32))
}

pub fn pick_folder(title: &str) -> Option<PathBuf> {
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree,
    };
    use windows_sys::Win32::UI::Shell::{
        BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW,
        SHGetPathFromIDListW,
    };
    let title = wide(title);
    let mut name = [0u16; 260];
    unsafe {
        CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32);
        let info = BROWSEINFOW {
            hwndOwner: null_mut(),
            pidlRoot: null_mut(),
            pszDisplayName: name.as_mut_ptr(),
            lpszTitle: title.as_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            lpfn: None,
            lParam: 0,
            iImage: 0,
        };
        let list = SHBrowseForFolderW(&info);
        if list.is_null() {
            return None;
        }
        let mut path = [0u16; 1024];
        let ok = SHGetPathFromIDListW(list, path.as_mut_ptr()) != 0;
        CoTaskMemFree(list as *const c_void);
        let len = path.iter().position(|&c| c == 0).unwrap_or(0);
        (ok && len > 0).then(|| std::ffi::OsString::from_wide(&path[..len]).into())
    }
}

/// Opens a web page in the default browser (under Wine, the desktop's).
pub fn open_url(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    unsafe {
        ShellExecuteW(
            null_mut(),
            wide("open").as_ptr(),
            wide(url).as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
    }
}

pub fn is_wine() -> bool {
    // Some Wine builds hide `wine_get_version`, so the environment Wine,
    // Proton and or-play set counts too.
    let exported = unsafe {
        let ntdll = GetModuleHandleW(wide("ntdll.dll").as_ptr());
        !ntdll.is_null() && GetProcAddress(ntdll, c"wine_get_version".as_ptr().cast()).is_some()
    };
    exported || or_launcher::wine_env(|name| std::env::var_os(name).is_some())
}

// XInput, for the rumble test and the pad-aware settings window.

fn xinput_proc(name: &std::ffi::CStr) -> Option<unsafe extern "system" fn() -> isize> {
    for dll in ["xinput1_4.dll", "xinput1_3.dll", "xinput9_1_0.dll"] {
        unsafe {
            let module = LoadLibraryW(wide(dll).as_ptr());
            if !module.is_null()
                && let Some(proc) = GetProcAddress(module, name.as_ptr().cast())
            {
                return Some(proc);
            }
        }
    }
    None
}

fn xinput_states() -> Vec<(u32, [u8; 16])> {
    type GetState = unsafe extern "system" fn(u32, *mut [u8; 16]) -> u32;
    let Some(get) = xinput_proc(c"XInputGetState") else {
        return Vec::new();
    };
    let get: GetState = unsafe { std::mem::transmute(get) };
    (0..4)
        .filter_map(|slot| {
            let mut state = [0u8; 16];
            (unsafe { get(slot, &mut state) } == 0).then_some((slot, state))
        })
        .collect()
}

pub fn xinput_pads() -> Vec<u32> {
    xinput_states().into_iter().map(|(slot, _)| slot).collect()
}

pub fn xinput_buttons() -> u16 {
    xinput_states().into_iter().fold(0, |all, (_, state)| {
        all | u16::from_le_bytes([state[4], state[5]])
    })
}

fn set_motors(speed: u16) -> Vec<u32> {
    type SetState = unsafe extern "system" fn(u32, *mut [u16; 2]) -> u32;
    let Some(set) = xinput_proc(c"XInputSetState") else {
        return Vec::new();
    };
    let set: SetState = unsafe { std::mem::transmute(set) };
    (0..4)
        .filter(|&slot| unsafe { set(slot, &mut [speed, speed]) } == 0)
        .collect()
}

pub fn stop_rumble() {
    set_motors(0);
}

pub fn test_rumble(percent: u32) -> usize {
    let speed = (percent.min(100) * 65535 / 100) as u16;
    let pads = set_motors(speed).len();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        set_motors(0);
    });
    pads
}

// Registry.

struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}

fn create_key(root: HKEY, path: &str) -> Result<Key, u32> {
    let mut key: HKEY = null_mut();
    let rc = unsafe {
        RegCreateKeyExW(
            root,
            wide(path).as_ptr(),
            0,
            null(),
            REG_OPTION_NON_VOLATILE,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        )
    };
    if rc == ERROR_SUCCESS {
        Ok(Key(key))
    } else {
        Err(rc)
    }
}

fn has_value(key: &Key, name: &str) -> bool {
    let name = wide(name);
    let rc = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            null(),
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };
    rc != ERROR_FILE_NOT_FOUND
}

fn write_string(key: &Key, name: &str, data: &str) -> Result<(), String> {
    let name = wide(name);
    let data = wide(data);
    let rc = unsafe {
        RegSetValueExW(
            key.0,
            name.as_ptr(),
            0,
            REG_SZ,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    if rc == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("cannot write registry value (error {rc})"))
    }
}

fn write_dword(key: &Key, name: &str, value: u32) -> Result<(), String> {
    let name = wide(name);
    let data = value.to_le_bytes();
    let rc = unsafe { RegSetValueExW(key.0, name.as_ptr(), 0, REG_DWORD, data.as_ptr().cast(), 4) };
    if rc == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("cannot write registry value (error {rc})"))
    }
}

/// The Wine controller baseline the known-good setup uses: raw evdev and
/// Wine's own SDL-mapped devices stay out, so only the pad itself reaches
/// the game (through the mod's SDL input system). Written once into the
/// private prefix; existing values are left alone.
pub fn wine_controller_baseline() -> Result<bool, String> {
    let key = create_key(
        HKEY_LOCAL_MACHINE,
        r"System\CurrentControlSet\Services\winebus",
    )
    .map_err(|rc| format!("cannot open the Wine bus key (error {rc})"))?;
    let mut wrote = false;
    for (name, value) in [
        ("DisableHidraw", 1),
        ("DisableInput", 1),
        ("Enable SDL", 1),
        ("Map Controllers", 0),
    ] {
        if !has_value(&key, name) {
            write_dword(&key, name, value)?;
            wrote = true;
        }
    }
    Ok(wrote)
}

/// Keeps 32-bit OpenGL working under Wine by asking for GLX instead of EGL
/// (some setups only expose working GL through GLX).
pub fn prefer_gpu_opengl() -> Result<bool, String> {
    let key = create_key(HKEY_CURRENT_USER, r"Software\Wine\X11 Driver")
        .map_err(|rc| format!("cannot open Wine display preferences (error {rc})"))?;
    if has_value(&key, "UseEGL") {
        return Ok(false);
    }
    write_string(&key, "UseEGL", "N")?;
    Ok(true)
}

/// Points Windows at the high-performance GPU for the game (laptops with
/// two GPUs).
pub fn prefer_fast_gpu(exe: &Path) -> Result<bool, String> {
    let key = create_key(
        HKEY_CURRENT_USER,
        r"Software\Microsoft\DirectX\UserGpuPreferences",
    )
    .map_err(|rc| format!("cannot open GPU preferences (error {rc})"))?;
    let name = exe.display().to_string();
    if has_value(&key, &name) {
        return Ok(false);
    }
    write_string(&key, &name, "GpuPreference=2;")?;
    Ok(true)
}

// Downloads (the HD texture pack), through WinHTTP: part of Windows and of
// Wine, so no TLS stack of our own.

struct Internet(*mut c_void);

impl Drop for Internet {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                windows_sys::Win32::Networking::WinHttp::WinHttpCloseHandle(self.0);
            }
        }
    }
}

fn last_error(what: &str) -> String {
    let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    format!("{what} failed (error {code})")
}

/// HTTPS GETs with WinHTTP. Redirects (GitHub's release downloads go to
/// its file servers) are followed by WinHTTP itself.
pub struct WinHttp;

impl or_launcher::hdpack::Fetch for WinHttp {
    fn fetch(
        &self,
        url: &str,
        from: u64,
        start: &mut dyn FnMut(bool),
        chunk: &mut dyn FnMut(&[u8]) -> bool,
    ) -> Result<(), String> {
        use windows_sys::Win32::Networking::WinHttp::*;
        let rest = url
            .strip_prefix("https://")
            .ok_or_else(|| format!("not an https address: {url}"))?;
        let (host, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let path = if path.is_empty() { "/" } else { path };
        unsafe {
            let session = Internet(WinHttpOpen(
                wide(concat!("OutRunLauncher/", env!("CARGO_PKG_VERSION"))).as_ptr(),
                WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                null(),
                null(),
                0,
            ));
            if session.0.is_null() {
                return Err(last_error("WinHttpOpen"));
            }
            WinHttpSetTimeouts(session.0, 30_000, 30_000, 30_000, 60_000);
            let connection = Internet(WinHttpConnect(
                session.0,
                wide(host).as_ptr(),
                INTERNET_DEFAULT_HTTPS_PORT,
                0,
            ));
            if connection.0.is_null() {
                return Err(last_error("connecting"));
            }
            let request = Internet(WinHttpOpenRequest(
                connection.0,
                wide("GET").as_ptr(),
                wide(path).as_ptr(),
                null(),
                null(),
                null(),
                WINHTTP_FLAG_SECURE,
            ));
            if request.0.is_null() {
                return Err(last_error("WinHttpOpenRequest"));
            }
            let headers = if from > 0 {
                wide(format!("Range: bytes={from}-"))
            } else {
                Vec::new()
            };
            let (header_ptr, header_len) = if headers.is_empty() {
                (null(), 0)
            } else {
                (headers.as_ptr(), (headers.len() - 1) as u32)
            };
            if WinHttpSendRequest(request.0, header_ptr, header_len, null(), 0, 0, 0) == 0 {
                return Err(last_error("sending the request"));
            }
            if WinHttpReceiveResponse(request.0, null_mut()) == 0 {
                return Err(last_error("receiving the response"));
            }
            let mut status: u32 = 0;
            let mut size = 4u32;
            WinHttpQueryHeaders(
                request.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                null(),
                (&mut status as *mut u32).cast(),
                &mut size,
                null_mut(),
            );
            match status {
                206 => start(true),
                200 => start(from == 0),
                code => return Err(format!("the server answered {code}")),
            }
            let mut buffer = vec![0u8; 256 * 1024];
            loop {
                let mut read = 0u32;
                if WinHttpReadData(
                    request.0,
                    buffer.as_mut_ptr().cast(),
                    buffer.len() as u32,
                    &mut read,
                ) == 0
                {
                    return Err(last_error("reading"));
                }
                if read == 0 {
                    return Ok(());
                }
                if !chunk(&buffer[..read as usize]) {
                    return Err("stopped".into());
                }
            }
        }
    }
}

/// Free bytes for this user on the drive holding `dir` (or its nearest
/// existing parent).
pub fn free_space(dir: &Path) -> Option<u64> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let existing = dir.ancestors().find(|p| p.exists())?;
    let mut free = 0u64;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide(existing.as_os_str()).as_ptr(),
            &mut free,
            null_mut(),
            null_mut(),
        )
    };
    (ok != 0).then_some(free)
}
