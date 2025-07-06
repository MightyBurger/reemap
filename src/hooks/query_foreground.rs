use std::path::PathBuf;
use thiserror::Error;
use tracing::{instrument, trace};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ForegroundWindowInfo {
    pub title: String,
    pub process: String,
}

#[derive(Debug, Error, Clone)]
pub enum ForegroundWindowError {
    #[error("foreground window handle is invalid")]
    InvalidHWND,
    #[error("window text is of negative length: {0}")]
    NegativeTextLen(i32),
    #[error("failed to open process")]
    OpenProcessError,
    #[error("failed to query process name")]
    QueryProcessError,
    #[error("failed to extract filename from process image path")]
    ExtractFilenameError,
}

pub type ForegroundWindowResult<T> = Result<T, ForegroundWindowError>;

#[instrument]
pub fn get_foreground_window() -> ForegroundWindowResult<ForegroundWindowInfo> {
    use windows::Win32::System::Threading as TH;
    use windows::Win32::UI::WindowsAndMessaging as WM;
    use windows::core::PWSTR;
    const CAP: usize = 512;

    trace!("getting foreground window");

    let hwnd = unsafe { WM::GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(ForegroundWindowError::InvalidHWND);
    }

    let process_id = {
        let mut process_id = 0u32;
        let process_id_ptr: *mut u32 = &mut process_id;
        let thread_id = unsafe { WM::GetWindowThreadProcessId(hwnd, Some(process_id_ptr)) };
        if thread_id == 0 {
            return Err(ForegroundWindowError::InvalidHWND);
        }
        process_id
    };

    /*
        A special case - the focused window is Reemap.
        It is important we do not attempt to grab the window text of Reemap. Why? Because otherwise,
        Reemap may hang when closing.

        See this blog:
        https://devblogs.microsoft.com/oldnewthing/20030821-00/?p=42833
        "The Secret Life of GetWindowText", Raymond Chen, Microsoft Dev Blogs, 21 Aug 2003

        As described there, GetWindowText behaves differently when the window belongs to the same
        process that called the function. Specifically, it sends a WM_GETTEXT message, and the
        event loop is responsible for responding.

        The problem happens when Reemap exits. In Reemap, the UI stops first, then the hook thread.
        So, for a brief period of time, the UI thread is not actively running a Windows event loop
        while the hook thread is still active.

        If, during this brief period, GetWindowText gets called and Reemap is the foreground window,
        GetWindowText hangs indefinitely, because it's waiting for the message loop to handle its
        WM_GETTEXT message but it never does.
    */

    // TODO rather than having hard-coded values, just have a special case for this.
    // Remaps should probably behave differently when Reemap is the focused window, anyways.
    let reemap_process_id = unsafe { TH::GetCurrentProcessId() };
    if process_id == reemap_process_id {
        return Ok(ForegroundWindowInfo {
            title: String::from("Reemap"),
            process: String::from("reemap.exe"),
        });
    }

    let title = {
        trace!("getting window title");

        let mut title = [0u16; CAP];
        let len = unsafe { WM::GetWindowTextW(hwnd, &mut title) };
        trace!("getwindowtext done");
        if len < 0 {
            return Err(ForegroundWindowError::NegativeTextLen(len));
        }
        let len = std::cmp::min(len as usize, CAP - 1);
        String::from_utf16_lossy(&title[0..len])
    };

    let process = {
        let hprocess = unsafe {
            match TH::OpenProcess(TH::PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                Ok(hprocess) => hprocess,
                Err(_) => {
                    return Err(ForegroundWindowError::OpenProcessError);
                }
            }
        };

        let mut title = [0u16; CAP];
        let mut len = (CAP - 1) as u32;
        match unsafe {
            TH::QueryFullProcessImageNameW(
                hprocess,
                TH::PROCESS_NAME_WIN32,
                PWSTR(&mut title as *mut u16),
                &mut len,
            )
        } {
            Ok(()) => (),
            Err(_) => {
                return Err(ForegroundWindowError::QueryProcessError);
            }
        };
        let len = std::cmp::min(len as usize, CAP - 1);
        let title = String::from_utf16_lossy(&title[0..len]);
        let title = PathBuf::from(title);
        let Some(title) = title.file_name() else {
            return Err(ForegroundWindowError::ExtractFilenameError);
        };
        String::from(title.to_string_lossy())
    };

    Ok(ForegroundWindowInfo { title, process })
}
