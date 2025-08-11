mod foreground_hook;
mod hooklocal;
mod input_hooks;
mod minimize_end_hook;

use crate::config;
use crate::gui;
use crate::query_windows::get_foreground_window;
use hooklocal::HOOKLOCAL;
use tracing::{debug, info, instrument, trace, warn};

use std::sync::Mutex;

use windows::Win32::Foundation;
use windows::Win32::System::Threading;
use windows::Win32::UI::WindowsAndMessaging;

const CHECK_FOREGROUND_INTERVAL_MS: u32 = 500;

// The main way to launch the hook thread. Pass in a std::thread::scope, and this function
// will spawn the thread to handle all the hooks involved in Reemap. It will return a proxy to the
// thread.
pub fn spawn_scoped<'scope, 'env>(
    s: &'scope std::thread::Scope<'scope, 'env>,
    config: config::Config,
    ui_proxy: winit::event_loop::EventLoopProxy<gui::ReemapGuiEvent>,
) -> HookthreadProxy {
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();
    s.spawn(|| {
        run(oneshot_sender, config, ui_proxy);
    });
    oneshot_receiver.recv().unwrap()
}

// Run the hook thread and return a proxy through the oneshot.
// Panics if the hook thread is already running.
#[instrument(skip_all, name = "hooks")]
pub fn run(
    sender: oneshot::Sender<HookthreadProxy>,
    config: config::Config,
    ui_proxy: winit::event_loop::EventLoopProxy<gui::ReemapGuiEvent>,
) {
    debug!("entering hook thread");
    use WindowsAndMessaging as WM;
    use num_traits::FromPrimitive;

    static RUNNING: Mutex<bool> = Mutex::new(false);

    let mut running = RUNNING.lock().unwrap();
    if *running {
        panic!("Attempted to start hook thread while it was already running");
    } else {
        *running = true;
    }
    std::mem::drop(running);

    // Initialize the persistent thread data.
    let mut hooklocal = HOOKLOCAL.lock().unwrap();
    *hooklocal = Some(hooklocal::HookLocalData::init_settings(config, ui_proxy));
    std::mem::drop(hooklocal);

    // Force Windows to create a message queue for this thread. We want to have one before we
    // give out our thread ID, which other threads use to post messages to.
    unsafe {
        let mut lpmsg_unused = WM::MSG::default();
        let _ = WM::PeekMessageW(&mut lpmsg_unused, None, 0, 0, WM::PM_NOREMOVE);
    }

    // Create a proxy and give it back to whoever spawned us.
    let thread_id = unsafe { Threading::GetCurrentThreadId() };
    let proxy = HookthreadProxy { thread_id };
    sender.send(proxy.clone()).unwrap();

    // Create a timer with which to check the foreground window.
    // It is the only polling done in Reemap, and it is only done as a backup.
    // Windows is not perfectly reliable in sending events when the foreground window changes.
    let timer_id = unsafe { WM::SetTimer(None, 0, CHECK_FOREGROUND_INTERVAL_MS, None) };
    if timer_id == 0 {
        panic!("could not create timer");
    }

    // Establish all of the hooks.
    let mouse_hhk = input_hooks::set_mouse_hook().unwrap();
    let keybd_hhk = input_hooks::set_keybd_hook().unwrap();
    let foreground_hhk = foreground_hook::set_hook(proxy.clone()).unwrap();
    let minimize_end_hhk = minimize_end_hook::set_hook(proxy.clone()).unwrap();

    let mut lpmsg = WM::MSG::default();
    unsafe {
        loop {
            let bret = WM::GetMessageW(&mut lpmsg, None, 0, 0);
            if !bret.as_bool() {
                break;
            }
            if bret.0 == -1 {
                warn!(?bret, "error from GetMessageW");
                break;
            }
            trace!("handling a new message");
            match HookMessage::from_u32(lpmsg.message) {
                Some(HookMessage::Quit) => {
                    trace!("handling Quit message");
                    let _ = input_hooks::remove_hook(mouse_hhk);
                    let _ = input_hooks::remove_hook(keybd_hhk);
                    let _ = foreground_hook::remove_hook(foreground_hhk);
                    let _ = minimize_end_hook::remove_hook(minimize_end_hhk);
                    WM::PostQuitMessage(0);
                    trace!("done handling Quit message");
                }
                Some(HookMessage::Update) => {
                    trace!("handling Update message");
                    info!("updating config");
                    let Foundation::WPARAM(raw_usize) = lpmsg.wParam;
                    let raw = raw_usize as *mut config::Config;
                    let config_boxed = Box::from_raw(raw);
                    let config = *config_boxed;

                    let mut hook_local_guard = HOOKLOCAL.lock().expect("mutex poisoned");
                    let hook_local = hook_local_guard
                        .as_mut()
                        .expect("local data should have been initialized");
                    hook_local.update_config(config);
                    drop(hook_local_guard);
                    trace!("done handling Update message");
                }
                Some(HookMessage::TimerExpire) | Some(HookMessage::CheckForeground) => {
                    trace!("handling TimerExpire or CheckForeground message");
                    match get_foreground_window() {
                        Ok(info) => {
                            let mut hook_local_guard = HOOKLOCAL.lock().expect("mutex poisoned");
                            let hook_local = hook_local_guard
                                .as_mut()
                                .expect("local data should have been initialized");
                            hook_local.update_from_foreground(info);
                            drop(hook_local_guard);
                        }
                        Err(e) => {
                            warn!(?e, "failed to get foreground window");
                        }
                    }
                    trace!("done handling TimerExpire or CheckForeground message");
                }
                Some(HookMessage::RegisterUIObserveInputs) => {
                    trace!("handling RegisterUIObserveInputs message");
                    debug!("registered to send inputs to UI");
                    let mut hook_local_guard = HOOKLOCAL.lock().expect("mutex poisoned");
                    let hook_local = hook_local_guard
                        .as_mut()
                        .expect("local data should have been initialized");
                    hook_local.ui_observing_inputs = true;
                    drop(hook_local_guard);
                    trace!("done handling RegisterUIObserveInputs message");
                }
                Some(HookMessage::UnregisterUIObserveInputs) => {
                    trace!("handling UnregisterUIObserveInputs message");
                    debug!("unregistered to send inputs to UI");
                    let mut hook_local_guard = HOOKLOCAL.lock().expect("mutex poisoned");
                    let hook_local = hook_local_guard
                        .as_mut()
                        .expect("local data should have been initialized");
                    hook_local.ui_observing_inputs = false;
                    drop(hook_local_guard);
                    trace!("done handling UnregisterUIObserveInputs message");
                }
                None => {
                    trace!("got unknown message; treat as normal");
                    let _ = WM::TranslateMessage(&lpmsg);
                    let _ = WM::DispatchMessageA(&lpmsg);
                    trace!("done treating as normal");
                }
            }
        }
    }

    let mut running = RUNNING.lock().unwrap();
    *running = false;
    if let Err(e) = unsafe { WM::KillTimer(None, timer_id) } {
        warn!(?e, "error killing timer");
    }
    if let Err(e) = unsafe { WM::ClipCursor(None) } {
        warn!(?e, "error removing the cursor clip");
    };
    debug!("exiting hook thread");
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[repr(u32)]
enum HookMessage {
    TimerExpire = WindowsAndMessaging::WM_TIMER,
    Quit = WindowsAndMessaging::WM_APP,
    Update = WindowsAndMessaging::WM_APP + 1,
    CheckForeground = WindowsAndMessaging::WM_APP + 2,
    RegisterUIObserveInputs = WindowsAndMessaging::WM_APP + 3,
    UnregisterUIObserveInputs = WindowsAndMessaging::WM_APP + 4,
}

#[derive(Debug, Clone)]
pub struct HookthreadProxy {
    thread_id: u32,
}

impl HookthreadProxy {
    pub fn quit(&self) {
        use num_traits::ToPrimitive;
        debug!("telling hookthread to quit");
        unsafe {
            WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                HookMessage::Quit
                    .to_u32()
                    .expect("msg should always be representable as u32"),
                Foundation::WPARAM(0),
                Foundation::LPARAM(0),
            )
            .expect("could not send to hookthread");
        }
    }
    pub fn update(&self, config: config::Config) {
        use num_traits::ToPrimitive;

        let config_boxed = Box::new(config);
        let raw = Box::into_raw(config_boxed);
        let raw_usize = raw as usize;

        unsafe {
            WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                HookMessage::Update
                    .to_u32()
                    .expect("msg should always be representable as u32"),
                Foundation::WPARAM(raw_usize),
                Foundation::LPARAM(0),
            )
            .expect("could not send to hookthread");
        }
    }
    pub fn check_foreground(&self) {
        use num_traits::ToPrimitive;
        unsafe {
            WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                HookMessage::CheckForeground
                    .to_u32()
                    .expect("msg should always be representable as u32"),
                Foundation::WPARAM(0),
                Foundation::LPARAM(0),
            )
            .expect("could not send to hookthread");
        }
    }
    // The UI thread calls this when it wants to be notified of inputs.
    pub fn register_observe_inputs(&self) {
        use num_traits::ToPrimitive;
        unsafe {
            WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                HookMessage::RegisterUIObserveInputs
                    .to_u32()
                    .expect("msg should always be representable as u32"),
                Foundation::WPARAM(0),
                Foundation::LPARAM(0),
            )
            .expect("could not send to hookthread");
        }
    }
    // The UI thread calls this when it is done being notified of inputs.
    pub fn unregister_observe_inputs(&self) {
        use num_traits::ToPrimitive;
        unsafe {
            WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                HookMessage::UnregisterUIObserveInputs
                    .to_u32()
                    .expect("msg should always be representable as u32"),
                Foundation::WPARAM(0),
                Foundation::LPARAM(0),
            )
            .expect("could not send to hookthread");
        }
    }
}
