// use crate::settings::{BaseRemapPolicy, LayerType, ProfileCondition, RemapPolicy};
use crate::config::{BaseLayer, BaseRemapPolicy, Layer, LayerType, RemapPolicy};

use crate::buttons::key::KeyButton;
use crate::buttons::mouse::MouseButton;
use crate::buttons::wheel::MouseWheelButton;
use crate::buttons::{Button, HoldButton, TapButton};

use crate::hooks::hooklocal::{ActiveProfile, HOOKLOCAL, HoldButtonState};

use windows::Win32::Foundation;
use windows::Win32::UI::Input::KeyboardAndMouse;
use windows::Win32::UI::WindowsAndMessaging;

pub fn set_mouse_hook() -> Result<WindowsAndMessaging::HHOOK, windows::core::Error> {
    use Foundation::{LPARAM, LRESULT, WPARAM};
    use WindowsAndMessaging::{SetWindowsHookExW, WH_MOUSE_LL};
    let idhook = WH_MOUSE_LL;
    let lpfn = Some(mouse_hook as unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT);
    let hmod = None;
    let dwthreadid = 0;
    unsafe { SetWindowsHookExW(idhook, lpfn, hmod, dwthreadid) }
}

pub fn set_keybd_hook() -> Result<WindowsAndMessaging::HHOOK, windows::core::Error> {
    use Foundation::{LPARAM, LRESULT, WPARAM};
    use WindowsAndMessaging::{SetWindowsHookExW, WH_KEYBOARD_LL};
    let idhook = WH_KEYBOARD_LL;
    let lpfn = Some(keybd_hook as unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT);
    let hmod = None;
    let dwthreadid = 0;
    unsafe { SetWindowsHookExW(idhook, lpfn, hmod, dwthreadid) }
}

pub unsafe fn remove_hook(hhk: WindowsAndMessaging::HHOOK) -> Result<(), windows::core::Error> {
    unsafe { WindowsAndMessaging::UnhookWindowsHookEx(hhk) }
}

// This is a LowLevelKeyboardProc.
// https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
#[allow(non_snake_case)]
unsafe extern "system" fn keybd_hook(
    nCode: i32,
    wParam: Foundation::WPARAM,
    lParam: Foundation::LPARAM,
) -> Foundation::LRESULT {
    use WindowsAndMessaging as WM;
    // From the above docs: If nCode is less than zero, this callback must call CallNextHookEx
    // and return the result.
    if nCode < 0 {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    // Let's disable hooks while scroll lock is enabled.
    if unsafe { KeyboardAndMouse::GetKeyState(KeyboardAndMouse::VK_SCROLL.0.into()) & 1 > 0 } {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    let hookstruct: WM::KBDLLHOOKSTRUCT = unsafe { *(lParam.0 as *const WM::KBDLLHOOKSTRUCT) };

    // Filter out any synthesized inputs to:
    //  1.  Avoid responding to our own inputs (note: could also do this with dwExtraInfo)
    //  2.  Avoid responding to inputs from something like AHK; this could create a loop depending
    //      on how Reemap and AHK are configured
    if hookstruct.flags.contains(WM::LLKHF_INJECTED) {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    // If we don't know about this button, let's just forward it along uninterrupted.
    let Some(key) = KeyButton::from_vk(hookstruct.vkCode as u8) else {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    };

    // Convert to an input and call the function.
    // If it's intercepted, do not let this message pass on.

    if hookstruct.flags.contains(WM::LLKHF_UP) {
        if intercept_hold_up_input(HoldButton::from(key)) {
            return Foundation::LRESULT(1);
        }
    } else if intercept_hold_down_input(HoldButton::from(key)) {
        return Foundation::LRESULT(1);
    }
    unsafe { WM::CallNextHookEx(None, nCode, wParam, lParam) }
}

// This is a LowLevelMouseProc.
// https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc
#[allow(non_snake_case)]
unsafe extern "system" fn mouse_hook(
    nCode: i32,
    wParam: Foundation::WPARAM,
    lParam: Foundation::LPARAM,
) -> Foundation::LRESULT {
    use WindowsAndMessaging as WM;
    // From the above docs: If nCode is less than zero, this callback must call CallNextHookEx
    // and return the result.
    if nCode < 0 {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    // Let's disable hooks while scroll lock is enabled.
    if unsafe { KeyboardAndMouse::GetKeyState(KeyboardAndMouse::VK_SCROLL.0.into()) & 1 > 0 } {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    let hookstruct: WM::MSLLHOOKSTRUCT = unsafe { *(lParam.0 as *const WM::MSLLHOOKSTRUCT) };

    // Filter out any synthesized inputs to:
    //  1.  Avoid responding to our own inputs (note: could also do this with dwExtraInfo)
    //  2.  Avoid responding to inputs from something like AHK; this could create a loop depending
    //      on how Reemap and AHK are configured
    if hookstruct.flags & WM::LLMHF_INJECTED != 0 {
        unsafe {
            return WM::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    enum Action {
        Down,
        Up,
    }
    enum MouseOrWheel {
        Mouse { button: MouseButton, action: Action },
        Wheel(MouseWheelButton),
    }
    use Action::{Down, Up};
    use MouseOrWheel::{Mouse, Wheel};

    let button: MouseOrWheel = match wParam.0 as u32 {
        WM::WM_LBUTTONDOWN => Mouse {
            button: MouseButton::Left,
            action: Down,
        },
        WM::WM_LBUTTONUP => Mouse {
            button: MouseButton::Left,
            action: Up,
        },
        WM::WM_MBUTTONDOWN => Mouse {
            button: MouseButton::Middle,
            action: Down,
        },
        WM::WM_MBUTTONUP => Mouse {
            button: MouseButton::Middle,
            action: Up,
        },
        WM::WM_RBUTTONDOWN => Mouse {
            button: MouseButton::Right,
            action: Down,
        },
        WM::WM_RBUTTONUP => Mouse {
            button: MouseButton::Right,
            action: Up,
        },
        WM::WM_XBUTTONDOWN => {
            let higher_word: u16 = ((hookstruct.mouseData & 0xFF00) >> 16) as u16;
            match higher_word {
                WM::XBUTTON1 => Mouse {
                    button: MouseButton::X1,
                    action: Down,
                },
                WM::XBUTTON2 => Mouse {
                    button: MouseButton::X2,
                    action: Down,
                },
                _ => {
                    // Malformed input; ignore it.
                    unsafe {
                        return WM::CallNextHookEx(None, nCode, wParam, lParam);
                    }
                }
            }
        }
        WM::WM_XBUTTONUP => {
            let higher_word: u16 = ((hookstruct.mouseData & 0xFF00) >> 16) as u16;
            match higher_word {
                WM::XBUTTON1 => Mouse {
                    button: MouseButton::X1,
                    action: Up,
                },
                WM::XBUTTON2 => Mouse {
                    button: MouseButton::X2,
                    action: Up,
                },
                _ => {
                    // Malformed input; ignore it.
                    unsafe {
                        return WM::CallNextHookEx(None, nCode, wParam, lParam);
                    }
                }
            }
        }
        WM::WM_MOUSEWHEEL => {
            let higher_word: u16 = ((hookstruct.mouseData & 0xFF00) >> 16) as u16;
            let higher_word_signed: i16 = higher_word as i16;
            if higher_word_signed > 0 {
                Wheel(MouseWheelButton::Up)
            } else if higher_word_signed < 0 {
                Wheel(MouseWheelButton::Down)
            } else {
                // Malformed input; ignore it.
                unsafe {
                    return WM::CallNextHookEx(None, nCode, wParam, lParam);
                }
            }
        }
        WM::WM_MOUSEHWHEEL => {
            let higher_word: u16 = ((hookstruct.mouseData & 0xFF00) >> 16) as u16;
            let higher_word_signed: i16 = higher_word as i16;
            if higher_word_signed > 0 {
                Wheel(MouseWheelButton::HorzRight)
            } else if higher_word_signed < 0 {
                Wheel(MouseWheelButton::HorzLeft)
            } else {
                // Malformed input; ignore it.
                unsafe {
                    return WM::CallNextHookEx(None, nCode, wParam, lParam);
                }
            }
        }
        _ => {
            // A mouse event we don't care about.
            // Forward it on.
            unsafe {
                return WM::CallNextHookEx(None, nCode, wParam, lParam);
            }
        }
    };

    let intercepted = match button {
        Mouse {
            button,
            action: Down,
        } => intercept_hold_down_input(HoldButton::from(button)),
        Mouse { button, action: Up } => intercept_hold_up_input(HoldButton::from(button)),
        Wheel(wheel) => intercept_tap_input(TapButton::from(wheel)),
    };

    if intercepted {
        return Foundation::LRESULT(1);
    }
    unsafe { WM::CallNextHookEx(None, nCode, wParam, lParam) }
}

/*

On button down:

    1.  Check button_state. Key repeat means it is likely we receive many DOWN inputs before an UP
        input. We want to map subsequent DOWN inputs the same as we mapped the first DOWN input.
        Additionally, if this DOWN input is due to key repeat, we do not want to toggle any toggle
        layers.

        i.  If the input state is "HeldNoRemap", forward the input unmodified and quit.

        ii. If the input state is "HeldWithRemap", send the listed Hold inputs as DOWN and quit.
            Do not send any Tap inputs because we've already sent the one for this button press.

        iii.If the input state is "NotHeld", we have a new button press. Let's process it.

    2.  Update which layers are enabled.

        For each layer (exclduing the base layer, which is always enabled):

        a.  If this input is in the layer's condition list, and all other buttons in this list
            are already held:

            i.  If this is a modifier layer, make it enabled.

            ii. If this is a toggle layer, toggle whether it is enabled.

    3.  Layers are now up-to-date with this latest button press. Now, dispatch inputs and mark
        button_state.

        For each enabled layer, starting from the highest priority, check the layer's policy
        for this button.

        a.  If this layer's policy is "Defer", check the next layer (not available for base layer).

        b.  If this layer's policy is "NoRemap", immediately forward the input unmodified. Mark the
            input in button_state as "HeldNoRemap".

        c.  If this layer's policy is "Remap", immediately send the specified inputs.
            Send Hold inputs as a DOWN input.
            Send Tap inputs.
            Mark the input in button_state as "HeldWithRemap".

On button up:

    1.  Every button up is a fresh, new button up. Update which layers are enabled.

        For each layer (excluding the base layer, which is always enabled):

        a.  If this input is in the layer's condition list:

            i.  If this is a modifier layer, make it disabled.

            ii. If this is a toggle layer, do nothing. Toggle layers only change on a depress.


    2.  Layers are now up-to-date with this latest button press. Now check and update button_state:

        a.  If the input is "HeldNoRemap", immediately forward the input unmodified.

        b.  If the input is "HeldWithRemap", immediately send the specified inputs as UP inputs.

        c.  If the input is "NotHeld", well, this shouldn't have happened. We got a KEYDOWN or
            MOUSEDOWN without remembering seeing a corresponding KEYUP or MOUSEUP. Ah well - let's
            just forward the input unmodified.

    (Notice we do not check the mappings in the layers on a button up.)

    3.  Mark the button_state as NotHeld.

On tap:

    Layers cannot be conditional on tap inputs. Therefore, we do not need to adjust any "enabled"
    values for the layers.

    For each layer, check the layer's policy for this button.

    1.  If this layer's policy is "Defer", check the next layer (not available for base layer).

    2.  If this layer's policy is "NoRemap", immediately forward the input unmodified.

    3.  If this layer's policy is "Remap", immediately send the specified inputs.
        Every Hold input in the policy should be sent together as a DOWN/UP pair.
        Every Tap input should be sent.

*/

/*

    Intercept Hold Down Input

*/

// Returns "true" if the input is intercepted.
// Refer to the above pseudocode.
fn intercept_hold_down_input(hold_button: HoldButton) -> bool {
    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
    let hook_local = hook_local
        .as_mut()
        .expect("local data should have been initialized");

    hook_local.update_active_profile();

    let (current_base, current_layers, current_layer_actives): (&BaseLayer, &[Layer], &mut [bool]) =
        match hook_local.active_profile {
            ActiveProfile::Default => (
                &hook_local.config.default.base,
                &hook_local.config.default.layers,
                &mut hook_local.active_layers_default,
            ),
            ActiveProfile::Other(profile_idx) => (
                &hook_local.config.profiles[profile_idx].base,
                &hook_local.config.profiles[profile_idx].layers,
                &mut hook_local.active_layers_profile[profile_idx],
            ),
        };

    // Step 1
    // An early return to handle key repeat - the case when you get multiple keydowns before a keyup
    match &hook_local.button_state[hold_button] {
        // Already held (key repeat), and there's no remap. So do not intercept it.
        HoldButtonState::HeldNoRemap => {
            return false;
        }

        // Already held (key repeat), and there is a remap.
        // Let's intercept it and repeat any keyboard keys this remap targets.
        HoldButtonState::HeldWithRemap(targets) => {
            let target_keys: Vec<KeyboardAndMouse::INPUT> = targets
                .iter()
                .filter_map(|btn| match btn {
                    Button::Key(key) => Some(key.to_keydown_input()),
                    _ => None,
                })
                .collect();

            send_input_batch(&target_keys);
            return true;
        }

        // Not held - this is a fresh input. Let's continue processing.
        HoldButtonState::NotHeld => {}
    }

    // Step 2
    // Update layers
    for (layer, active) in current_layers
        .iter()
        .zip(current_layer_actives.iter_mut())
        .filter(|(layer, _)| layer.enabled)
    {
        // Only update layers for which this button is a condition.
        if layer.condition.contains(&hold_button) {
            // All conditions met?
            if layer
                .condition
                .iter()
                .filter(|&condition| *condition != hold_button)
                .all(|condition| hook_local.button_state[*condition] != HoldButtonState::NotHeld)
            {
                // All conditions met. Let's enable/toggle this layer.
                match &layer.layer_type {
                    LayerType::Modifier => *active = true,
                    LayerType::Toggle => *active = !*active,
                }
            }
        }
    }

    // Step 3
    // Identify the appropriate remap and apply it. At the same time, set button_state.
    for (layer, _) in current_layers
        .iter()
        .zip(current_layer_actives.iter())
        .filter(|(layer, active)| layer.enabled && **active)
        .rev()
    {
        match &layer.policy[Button::from(hold_button)] {
            RemapPolicy::Defer => {}
            RemapPolicy::Remap(output) => {
                let target_buttons: Vec<KeyboardAndMouse::INPUT> = output
                    .iter()
                    .map(|button| match button {
                        Button::Key(key) => key.to_keydown_input(),
                        Button::Mouse(mouse) => mouse.to_mousedown_input(),
                        Button::Wheel(wheel) => wheel.to_input(),
                    })
                    .collect();
                send_input_batch(&target_buttons);
                hook_local.button_state[hold_button] =
                    HoldButtonState::HeldWithRemap(output.clone());
                return true;
            }
            RemapPolicy::NoRemap => {
                hook_local.button_state[hold_button] = HoldButtonState::HeldNoRemap;
                return false;
            }
        }
    }
    match &current_base.policy[Button::from(hold_button)] {
        BaseRemapPolicy::Remap(output) => {
            let target_buttons: Vec<KeyboardAndMouse::INPUT> = output
                .iter()
                .map(|button| match button {
                    Button::Key(key) => key.to_keydown_input(),
                    Button::Mouse(mouse) => mouse.to_mousedown_input(),
                    Button::Wheel(wheel) => wheel.to_input(),
                })
                .collect();
            send_input_batch(&target_buttons);
            hook_local.button_state[hold_button] = HoldButtonState::HeldWithRemap(output.clone());
            true
        }
        BaseRemapPolicy::NoRemap => {
            hook_local.button_state[hold_button] = HoldButtonState::HeldNoRemap;
            false
        }
    }
}

/*

    Intercept Hold Up Input

*/

// Returns "true" if the input is intercepted.
// Refer to the above pseudocode.
fn intercept_hold_up_input(hold_button: HoldButton) -> bool {
    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
    let hook_local = hook_local
        .as_mut()
        .expect("local data should have been initialized");

    hook_local.update_active_profile();

    let (current_layers, current_layer_actives): (&[Layer], &mut [bool]) =
        match hook_local.active_profile {
            ActiveProfile::Default => (
                &hook_local.config.default.layers,
                &mut hook_local.active_layers_default,
            ),
            ActiveProfile::Other(profile_idx) => (
                &hook_local.config.profiles[profile_idx].layers,
                &mut hook_local.active_layers_profile[profile_idx],
            ),
        };

    // Step 1
    // Update layers
    for (layer, active) in current_layers
        .iter()
        .zip(current_layer_actives.iter_mut())
        .filter(|(layer, _)| layer.enabled)
    {
        // Only update layers for which this button is a condition.
        // These layers are no longer active.
        if layer.condition.contains(&hold_button) {
            match &layer.layer_type {
                LayerType::Modifier => *active = false,
                LayerType::Toggle => (), // Toggle buttons not affected by keyup
            }
        }
    }

    // Step 2
    // See what this button was mapped to.
    // Note we never consult the profile. The original decision of what a button maps to is only
    // made when the button is first pressed.
    let remapped = match &hook_local.button_state[hold_button] {
        // This button down was not intercepted, so let's not intercept the button up.
        HoldButtonState::HeldNoRemap | HoldButtonState::NotHeld => false,

        // This button down was intercepted, so let's intercept the button up the same way.
        HoldButtonState::HeldWithRemap(targets) => {
            let target_buttons: Vec<KeyboardAndMouse::INPUT> = targets
                .iter()
                .filter_map(|button| match button {
                    Button::Key(key) => Some(key.to_keyup_input()),
                    Button::Mouse(mouse) => Some(mouse.to_mouseup_input()),
                    Button::Wheel(_wheel) => None, // Wheel input only sent on down press
                })
                .collect();

            send_input_batch(&target_buttons);
            true
        }
    };

    // Step 3
    hook_local.button_state[hold_button] = HoldButtonState::NotHeld;
    remapped
}

/*

    Intercept Tap Input

*/

// Returns "true" if the input is intercepted.
// Refer to the above pseudocode.
fn intercept_tap_input(tap_button: TapButton) -> bool {
    // Layers are not allowed to depend on tap inputs.
    // Additionally, we do not try to remember which tap inputs are "held", because it is
    // meaningless to "hold" a scroll wheel button.
    // This makes the job much easier.

    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
    let hook_local = hook_local
        .as_mut()
        .expect("local data should have been initialized");

    hook_local.update_active_profile();

    let (current_base, current_layers, current_layer_actives): (&BaseLayer, &[Layer], &mut [bool]) =
        match hook_local.active_profile {
            ActiveProfile::Default => (
                &hook_local.config.default.base,
                &hook_local.config.default.layers,
                &mut hook_local.active_layers_default,
            ),
            ActiveProfile::Other(profile_idx) => (
                &hook_local.config.profiles[profile_idx].base,
                &hook_local.config.profiles[profile_idx].layers,
                &mut hook_local.active_layers_profile[profile_idx],
            ),
        };

    for (layer, _) in current_layers
        .iter()
        .zip(current_layer_actives.iter())
        .filter(|(layer, active)| layer.enabled && **active)
        .rev()
    {
        match &layer.policy[Button::from(tap_button)] {
            RemapPolicy::Defer => {}
            RemapPolicy::Remap(output) => {
                let target_buttons: Vec<KeyboardAndMouse::INPUT> = output
                    .iter()
                    .flat_map(|button| match button {
                        Button::Key(key) => {
                            vec![key.to_keydown_input(), key.to_keyup_input()].into_iter()
                        }
                        Button::Mouse(mouse) => {
                            vec![mouse.to_mousedown_input(), mouse.to_mouseup_input()].into_iter()
                        }
                        Button::Wheel(wheel) => vec![wheel.to_input()].into_iter(),
                    })
                    .collect();
                send_input_batch(&target_buttons);
                return true;
            }
            RemapPolicy::NoRemap => {
                return false;
            }
        }
    }
    match &current_base.policy[Button::from(tap_button)] {
        BaseRemapPolicy::Remap(output) => {
            let target_buttons: Vec<KeyboardAndMouse::INPUT> = output
                .iter()
                .flat_map(|button| match button {
                    Button::Key(key) => {
                        vec![key.to_keydown_input(), key.to_keyup_input()].into_iter()
                    }
                    Button::Mouse(mouse) => {
                        vec![mouse.to_mousedown_input(), mouse.to_mouseup_input()].into_iter()
                    }
                    Button::Wheel(wheel) => vec![wheel.to_input()].into_iter(),
                })
                .collect();
            send_input_batch(&target_buttons);
            true
        }
        BaseRemapPolicy::NoRemap => false,
    }
}

fn send_input_batch(input: &[KeyboardAndMouse::INPUT]) {
    let cbsize = std::mem::size_of::<KeyboardAndMouse::INPUT>() as i32;
    unsafe {
        KeyboardAndMouse::SendInput(input, cbsize);
    }
}
