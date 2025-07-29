# Architecture of Reemap

The purpose of this file is to give a general overview of how Reemap is structured. The hope is to
give you a head-start if you need to dive into the code.

## Resources

Here are some resources you might find helpful.

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
threads separate just ensures the UI thread doesn't add any delay to the hook thread, which might
otherwise create input latency.

The two threads are event loop-based, and they use "proxies" to post messages to each-other's
message loop. The UI thread is a [winit] event loop, and the hook thread is a plain Windows message
loop.

The two threads run mostly independently, but they do communicate through this message passing.

The UI thread sends these messages to the hook thread:
- `Update` to change the remaps when the user clicks *Apply* in the UI
- `Quit` as the UI thread exits to instruct the hook thread to stop running

The hook thread sends the UI thread these messages:
- `ChangedProfile` to inform the UI thread of the current active profile so it can display this to
the user

## The UI thread

The UI thread displays the GUI where the user can configure their remap configuration. The GUI uses
[egui] for the UI.

Typically, desktop egui apps use [eframe]. Unfortunately, eframe
has a [bug](https://github.com/emilk/egui/issues/3655) preventing the window from displaying once
it has been minimized; this is a dealbreaker for Reemap. Additionally, crates that add an icon to
the Windows tray require access to the event loop.

Reemap therefore has its own egui implementation based closely on [this example][glow example].
Additional code exists to implement running in the background and adding an icon to the tray.

The grungy setup happens in `gui/mod.rs` and `gui/glutin_ctx.rs`. If you want to skip to the actual
egui application code where things like buttons exist, go to `gui/reemapp.rs`.

If [eframe] adds support for running in the system tray, Reemap should likely switch to just
using it.

## The Hook thread

The Hook thread is responsible for managing the low-level keyboard and mouse hooks.

Like AutoHotkey and X-Mouse, Reemap uses Windows low-level keyboard and mouse hooks.

According to the API docs, a low-level hook's callback procedure runs in the context of
the thread that established the hook. The thread also must have a Windows message loop
running, since Windows seems to invoke the hook through this message loop. So, it is important this
message loop and the callback procedures run quickly. Any delay might manifest as input latency.

This is partially why Reemap does not use [winit] on the hook thread. I don't have a full picture of
what [winit] does, so having more control over the event loop makes me feel a little more confident
there won't be unexpected delays. I'm also not sure whether any of [winit]'s abstractions would be
incompatible with these low-level hooks I'm installing.

[eframe]: https://crates.io/crates/eframe
[egui]: https://github.com/emilk/egui
[winit]: https://github.com/rust-windowing/winit
[glow example]: https://github.com/emilk/egui/blob/main/crates/egui_glow/examples/pure_glow.rs