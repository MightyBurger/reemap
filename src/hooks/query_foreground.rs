use std::path::PathBuf;
use thiserror::Error;
use tracing::{instrument, trace};

const CAP: usize = 512;

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
    #[error("thread ID is invalid")]
    InvalidThreadID,
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
    use windows::Win32::UI::WindowsAndMessaging as WM;
    trace!("getting foreground window");

    let hwnd = unsafe { WM::GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(ForegroundWindowError::InvalidHWND);
    }

    trace!("got foreground HWND");

    let title = unsafe { get_window_title(hwnd) }?;
    trace!(?title, "got foreground title");
    let process = unsafe { get_process_name(hwnd) }?;
    trace!(?process, "got foreground process");

    Ok(ForegroundWindowInfo { title, process })
}

// SAFETY: hwnd must be valid
#[instrument]
unsafe fn get_window_title(
    hwnd: windows::Win32::Foundation::HWND,
) -> ForegroundWindowResult<String> {
    use windows::Win32::UI::WindowsAndMessaging as WM;

    trace!("getting window title");

    let mut title = [0u16; CAP];
    // TODO
    // This hangs on program exit!!
    // This is why: https://devblogs.microsoft.com/oldnewthing/20030821-00/?p=42833
    // "The Secret Life of GetWindowText", Raymond Chen, Microsoft Dev Blogs, 21 Aug 2003
    let len = unsafe { WM::GetWindowTextW(hwnd, &mut title) };
    trace!("getwindowtext done");
    if len < 0 {
        return Err(ForegroundWindowError::NegativeTextLen(len));
    }
    let len = std::cmp::min(len as usize, CAP - 1);
    Ok(String::from_utf16_lossy(&title[0..len]))
}

// SAFETY: hwnd must be valid
#[instrument]
unsafe fn get_process_name(
    hwnd: windows::Win32::Foundation::HWND,
) -> ForegroundWindowResult<String> {
    use windows::Win32::System::Threading as TH;
    use windows::Win32::UI::WindowsAndMessaging as WM;
    use windows::core::PWSTR;

    trace!("getting process name");

    let mut lpdwprocessid = 0u32;
    let lpdwprocessid_ptr: *mut u32 = &mut lpdwprocessid;
    let dwprocessid = unsafe { WM::GetWindowThreadProcessId(hwnd, Some(lpdwprocessid_ptr)) };
    if dwprocessid == 0 {
        return Err(ForegroundWindowError::InvalidThreadID);
    }

    let hprocess = unsafe {
        match TH::OpenProcess(TH::PROCESS_QUERY_LIMITED_INFORMATION, false, lpdwprocessid) {
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
    Ok(String::from(title.to_string_lossy()))
}
