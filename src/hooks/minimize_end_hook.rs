use crate::hooks::HookthreadProxy;
use std::sync::Mutex;
use thiserror::Error;
use tracing::debug;
use tracing::{instrument, warn};
use windows::Win32::Foundation;
use windows::Win32::UI::Accessibility;
use windows::Win32::UI::WindowsAndMessaging;

#[derive(Debug, Error, Clone)]
pub enum MinimizeEndHookError {
    #[error("error setting event hook")]
    UnsuccessfulHook,
    #[error("error removing event hook")]
    UnsuccessfulUnhook,
}

pub type MinimizeEndHookResult<T> = Result<T, MinimizeEndHookError>;

static MINIMIZE_END_LOCAL: Mutex<Option<HookthreadProxy>> = Mutex::new(None);

pub fn set_hook(proxy: HookthreadProxy) -> MinimizeEndHookResult<Accessibility::HWINEVENTHOOK> {
    {
        let mut local = MINIMIZE_END_LOCAL.lock().unwrap();
        *local = Some(proxy);
    }
    let eventmin = WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;
    let eventmax = WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;
    let hmodwineventproc = None;
    let pfnwineventproc = Some(
        on_minimize_end
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
        return Err(MinimizeEndHookError::UnsuccessfulHook);
    }
    Ok(hook)
}

pub fn remove_hook(hhk: Accessibility::HWINEVENTHOOK) -> MinimizeEndHookResult<()> {
    {
        let mut local = MINIMIZE_END_LOCAL.lock().unwrap();
        *local = None;
    }
    let success = unsafe { Accessibility::UnhookWinEvent(hhk) };
    match success.as_bool() {
        true => Ok(()),
        false => Err(MinimizeEndHookError::UnsuccessfulUnhook),
    }
}

#[allow(non_snake_case)]
#[instrument(skip_all)]
unsafe extern "system" fn on_minimize_end(
    _hwineventhook: Accessibility::HWINEVENTHOOK,
    event: u32,
    _hwnd: Foundation::HWND,
    _idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    debug!("triggered");
    if event != WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND {
        warn!("got unexpected event");
        return;
    }
    let proxy = {
        let local = MINIMIZE_END_LOCAL.lock().unwrap();
        local.as_ref().unwrap().clone()
    };
    proxy.check_foreground();
}
