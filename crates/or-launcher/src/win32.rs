//! The one Win32 call the SDK (`rea-launcher-win32`) does not cover: the HD
//! texture pack download through WinHTTP. All of the launcher's own
//! `unsafe` lives here.

use std::ffi::c_void;
use std::ptr::{null, null_mut};

use rea_launcher_win32::wide::wide;

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
