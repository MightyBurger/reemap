pub mod hooklocal;
pub mod input;

use hooklocal::HOOKLOCAL;

use crate::settings::Settings;

use std::sync::Mutex;

use windows::Win32::Foundation;
use windows::Win32::System::Threading;
use windows::Win32::UI::WindowsAndMessaging;

// The main way to launch the hook thread. Pass in a std::thread::scope, and this function
// will spawn the thread to handle all the hooks involved in Reemap. It will return a proxy to the
// thread.
pub fn spawn_scoped<'scope, 'env>(
    s: &'scope std::thread::Scope<'scope, 'env>,
    settings: Settings,
) -> HookthreadProxy {
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();
    s.spawn(|| {
        run(oneshot_sender, settings);
    });
    oneshot_receiver.recv().unwrap()
}

// Run the hook thread and return a proxy through the oneshot.
// Panics if the hook thread is already running.
pub fn run(sender: oneshot::Sender<HookthreadProxy>, settings: Settings) {
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
    *hooklocal = Some(hooklocal::HookLocalData::init_settings(settings));
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
    sender.send(proxy).unwrap();

    let mouse_hhk = input::set_mouse_hook().unwrap();
    let keybd_hhk = input::set_keybd_hook().unwrap();
    let mut lpmsg = WM::MSG::default();
    unsafe {
        loop {
            let bret = WM::GetMessageW(&mut lpmsg, None, 0, 0);
            if !bret.as_bool() {
                break;
            }
            if bret.0 == -1 {
                // TODO handle the error instead of just quitting
                break;
            }
            match HookMessage::from_u32(lpmsg.message) {
                Some(HookMessage::Quit) => {
                    input::remove_hook(mouse_hhk).unwrap();
                    input::remove_hook(keybd_hhk).unwrap();
                    WM::PostQuitMessage(0);
                }
                Some(HookMessage::Update) => {
                    println!("Updating configuration!");
                    let Foundation::WPARAM(raw_usize) = lpmsg.wParam;
                    let raw = raw_usize as *mut Settings;
                    let settings_boxed = Box::from_raw(raw);
                    let settings = *settings_boxed;

                    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
                    let hook_local = hook_local
                        .as_mut()
                        .expect("local data should have been initialized");
                    hook_local.settings = settings;
                }
                None => {
                    let _ = WM::TranslateMessage(&lpmsg);
                    let _ = WM::DispatchMessageA(&lpmsg);
                }
            }
        }
    }

    let mut running = RUNNING.lock().unwrap();
    *running = false;
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
    Quit = WindowsAndMessaging::WM_APP,
    Update = WindowsAndMessaging::WM_APP + 1,
}

#[derive(Debug, Clone, Copy)]
pub struct HookthreadProxy {
    thread_id: u32,
}

impl HookthreadProxy {
    pub fn quit(&self) {
        use num_traits::ToPrimitive;
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
    pub fn update(&self, settings: Settings) {
        use num_traits::ToPrimitive;

        let settings_boxed = Box::new(settings);
        let raw = Box::into_raw(settings_boxed);
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
}
