// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utility functions for Windows

use std::path::PathBuf;
use thiserror::Error;
use tracing::warn;
use windows::Win32::Foundation as FN;
use windows::Win32::System::Threading as TH;
use windows::Win32::UI::WindowsAndMessaging as WM;
use windows::core::PWSTR;

const CAP: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub title: String,
    pub process: String,
    pub rect: Option<FN::RECT>,
}

#[derive(Debug, Error, Clone)]
pub enum WindowError {
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

pub type WindowResult<T> = Result<T, WindowError>;

pub fn get_foreground_window() -> WindowResult<WindowInfo> {
    let hwnd = unsafe { WM::GetForegroundWindow() };
    get_window_info(hwnd)
}

pub fn get_window_info(hwnd: FN::HWND) -> WindowResult<WindowInfo> {
    if hwnd.is_invalid() {
        return Err(WindowError::InvalidHWND);
    }
    unsafe { get_window_info_unchecked(hwnd) }
}

// SAFETY: hwnd must be valid
pub unsafe fn get_window_info_unchecked(hwnd: FN::HWND) -> WindowResult<WindowInfo> {
    let process_id = {
        let mut process_id = 0u32;
        let process_id_ptr: *mut u32 = &mut process_id;
        let thread_id = unsafe { WM::GetWindowThreadProcessId(hwnd, Some(process_id_ptr)) };
        if thread_id == 0 {
            return Err(WindowError::InvalidHWND);
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
        return Ok(WindowInfo {
            title: String::from("Reemap"),
            process: String::from("reemap.exe"),
            rect: None,
        });
    }

    let title = {
        let mut title = [0u16; CAP];
        let len = unsafe { WM::GetWindowTextW(hwnd, &mut title) };
        if len < 0 {
            return Err(WindowError::NegativeTextLen(len));
        }
        let len = std::cmp::min(len as usize, CAP - 1);
        String::from_utf16_lossy(&title[0..len])
    };

    let process = {
        let hprocess = unsafe {
            match TH::OpenProcess(TH::PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                Ok(hprocess) => hprocess,
                Err(_) => {
                    return Err(WindowError::OpenProcessError);
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
                return Err(WindowError::QueryProcessError);
            }
        };
        let len = std::cmp::min(len as usize, CAP - 1);
        let title = String::from_utf16_lossy(&title[0..len]);
        let title = PathBuf::from(title);
        let Some(title) = title.file_name() else {
            return Err(WindowError::ExtractFilenameError);
        };
        String::from(title.to_string_lossy())
    };

    let rect = {
        let mut rect = FN::RECT::default();
        let rect_ptr = &mut rect as *mut FN::RECT;
        let result = unsafe { WM::GetWindowRect(hwnd, rect_ptr) };
        match result {
            Ok(()) => Some(rect),
            Err(e) => {
                warn!(?e, "could not get window rect");
                None
            }
        }
    };

    Ok(WindowInfo {
        title,
        process,
        rect,
    })
}

/// Get every visible window.
pub fn enumerate_open_windows() -> Vec<WindowInfo> {
    let mut windows = Vec::new();
    let windows_ptr = &mut windows as *mut Vec<_>;
    let windows_ptr_isize = windows_ptr as isize;
    unsafe { WM::EnumWindows(Some(enum_windows_proc), FN::LPARAM(windows_ptr_isize)) }.unwrap();
    windows
}

// This is an EnumWindowsProc.
// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms633498(v=vs.85)
unsafe extern "system" fn enum_windows_proc(
    hwnd: FN::HWND,
    lparam: FN::LPARAM,
) -> windows::core::BOOL {
    let FN::LPARAM(windows_ptr_isize) = lparam;
    let windows_ptr = windows_ptr_isize as *mut Vec<WindowInfo>;
    let windows = unsafe { &mut *windows_ptr };

    if let Ok(info) = unsafe { get_window_info_unchecked(hwnd) }
        && !info.title.is_empty()
        && unsafe { WM::IsWindowVisible(hwnd) }.as_bool()
    {
        windows.push(info);
    }

    true.into()
}
