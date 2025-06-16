# Architecture of Reemap

The purpose of this file is to give a general overview of how Reemap is structured since it may
not be clear just from looking at the code.

## Resources

Here are some resources you might find helpful for understanding Reemap.

- [egui]
- this [egui integration example][glow example]
- [tray-icon](https://github.com/tauri-apps/tray-icon)
- Windows [message loops](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues)
- Using custom app messages in message loops; see [WM_APP](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-app)
- Win32 hooks; see [SetWindowsHookExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw)
- Keyboard and mouse hooks; see [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc) and [LowLevelMouseProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)
- Hooks and functions to detect the foreground window; see [GetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow?redirectedfrom=MSDN) and [CBTProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/cbtproc)

## The two threads

Reemap runs with two threads: the UI thread, and the Hook thread. Keeping the two
threads separate just ensures nothing happening on the UI thread adds unexpected latency to remaps
on the hook thread.

Reemap first spawns the Hook thread which gives back a `HookthreadProxy`. Reemap then runs the UI 
in the main thread.

The `HookthreadProxy` is what the UI thread uses to control the Hook thread. It is the only means of
communication between the two threads. The proxy works by posting messages to the message loop in
the Hook thread.

The UI uses this proxy in two ways. First, the proxy is used to let the user change remaps while
Reemap is running. When the user clicks *Apply* on the UI, the UI thread calls `update()` on the
proxy to send over a new remap configuration.

Second, after the UI has closed, the UI thread calls `quit()` on the proxy. This signals to the Hook
thread to stop running and close out the Windows hooks.

## The UI thread

The UI thread displays the GUI where the user can configure their remap configuration. The GUI uses
[egui].

The [egui] crate is a straightforward approach to implementing a GUI in a Rust program. It is not
great for highly configurable layout, but the GUI is only a secondary need in Reemap.

Typically, desktop egui apps use [eframe](https://crates.io/crates/eframe). Unfortunately, eframe
has a [bug](https://github.com/emilk/egui/issues/3655) preventing the window from displaying once
it has been minimized; this is a dealbreaker for Reemap. Additionally, crates that add an icon to
the Windows tray require access to the event loop.

Reemap therefore has its own egui implementation based closely on [this example][glow example].
Additional code exists to implement running in the background and adding an icon to the tray.

The grungy setup happens in `gui/mod.rs` and `gui/glutin_ctx.rs`. If you want to skip to the actual
egui application code where things like buttons exist, go to `gui/reemapp.rs`.

## The Hook thread

The Hook thread is responsible for managing the low-level keyboard and mouse hooks.

You may be surprised to hear a Windows low-level hook's callback procedure runs in the context of
the thread that established the hook. Additionally, the thread *must* have a Windows message loop
running.

So, the Hook thread's operation centers around its message loop. It is critical this message loop
and the callback procedures run efficiently. Any delay will manifest as additional input latency.

This is partially why Reemap does not use [winit] on the hook thread. It is harder to guarantee
[winit] is running the event loop in a maximally efficient manner. Additionally, I do not know
how well [winit]'s message loop abstraction works when I manually add these low-level hooks. That
isn't really what [winit] is designed to do. Besides, since Reemap is a Windows-only program, it
gains no benefit from the cross-platform abstractions in [winit].


[egui]: https://github.com/emilk/egui
[winit]: https://github.com/rust-windowing/winit
[glow example]: https://github.com/emilk/egui/blob/main/crates/egui_glow/examples/pure_glow.rs