// Copyright 2025 Jordan Johnson
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::hooks::HookthreadProxy;
use std::sync::Mutex;
use thiserror::Error;
use tracing::debug;
use tracing::{instrument, warn};
use windows::Win32::Foundation;
use windows::Win32::UI::Accessibility;
use windows::Win32::UI::WindowsAndMessaging;

#[derive(Debug, Error, Clone)]
pub enum ForegroundHookError {
    #[error("error setting event hook")]
    UnsuccessfulHook,
    #[error("error removing event hook")]
    UnsuccessfulUnhook,
}

pub type ForegroundHookResult<T> = Result<T, ForegroundHookError>;

static FOREGROUND_LOCAL: Mutex<Option<HookthreadProxy>> = Mutex::new(None);

pub fn set_hook(proxy: HookthreadProxy) -> ForegroundHookResult<Accessibility::HWINEVENTHOOK> {
    {
        let mut local = FOREGROUND_LOCAL.lock().unwrap();
        *local = Some(proxy);
    }
    let eventmin = WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;
    let eventmax = WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;
    let hmodwineventproc = None;
    let pfnwineventproc = Some(
        on_foreground
            as unsafe extern "system" fn(
                Accessibility::HWINEVENTHOOK,
                u32,
                Foundation::HWND,
                i32,
                i32,
                u32,
                u32,
            ),
    );
    let idprocess = 0;
    let idthread = 0;
    let dwflags = WindowsAndMessaging::WINEVENT_OUTOFCONTEXT;
    let hook = unsafe {
        Accessibility::SetWinEventHook(
            eventmin,
            eventmax,
            hmodwineventproc,
            pfnwineventproc,
            idprocess,
            idthread,
            dwflags,
        )
    };
    if hook.is_invalid() {
        return Err(ForegroundHookError::UnsuccessfulHook);
    }
    Ok(hook)
}

pub fn remove_hook(hhk: Accessibility::HWINEVENTHOOK) -> ForegroundHookResult<()> {
    {
        let mut local = FOREGROUND_LOCAL.lock().unwrap();
        *local = None;
    }
    let success = unsafe { Accessibility::UnhookWinEvent(hhk) };
    match success.as_bool() {
        true => Ok(()),
        false => Err(ForegroundHookError::UnsuccessfulUnhook),
    }
}

#[allow(non_snake_case)]
#[instrument(skip_all)]
unsafe extern "system" fn on_foreground(
    _hwineventhook: Accessibility::HWINEVENTHOOK,
    event: u32,
    _hwnd: Foundation::HWND,
    _idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    debug!("triggered");
    if event != WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND {
        warn!("got unexpected event");
        return;
    }
    let proxy = {
        let local = FOREGROUND_LOCAL.lock().unwrap();
        local.as_ref().unwrap().clone()
    };
    proxy.check_foreground();
}
