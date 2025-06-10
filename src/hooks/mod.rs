use super::config::{LayerType, ProfileState, RemapPolicy};

use crate::buttons::key::KeyInput;
use crate::buttons::mouse::MouseInput;
use crate::buttons::wheel::MouseWheelInput;
use crate::buttons::{Button, HoldButton, Input, TapButton};
use crate::config::BaseRemapPolicy;

use std::sync::Mutex;

use windows::Win32::Foundation;
use windows::Win32::UI::Input::KeyboardAndMouse;
use windows::Win32::UI::WindowsAndMessaging;

use enum_map::EnumMap;

// This is a LowLevelKeyboardProc.
// https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
#[allow(non_snake_case)]
unsafe extern "system" fn keybd_hook(
    nCode: i32,
    wParam: Foundation::WPARAM,
    lParam: Foundation::LPARAM,
) -> Foundation::LRESULT {
    // From the above docs: If nCode is less than zero, this callback must call CallNextHookEx
    // and return the result.
    if nCode < 0 {
        unsafe {
            return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    let hookstruct: WindowsAndMessaging::MSLLHOOKSTRUCT =
        unsafe { *(lParam.0 as *const WindowsAndMessaging::MSLLHOOKSTRUCT) };

    // Filter out any synthesized inputs to:
    //  1.  Avoid responding to our own inputs (note: could also do this with dwExtraInfo)
    //  2.  Avoid responding to inputs from something like AHK; this could create a loop depending
    //      on how Reemap and AHK are configured
    if hookstruct.flags & WindowsAndMessaging::LLMHF_INJECTED != 0 {
        unsafe {
            return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    // Convert to an input and call the function.
    // If it's intercepted, do not let this message pass on.
    todo!()
    // if intercept(Input::Key(KeyInput::A)) {
    //     return Foundation::LRESULT(1);
    // }
    // unsafe {
    //     return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
    // }
}

// This is a LowLevelMouseProc.
// https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc
#[allow(non_snake_case)]
unsafe extern "system" fn mouse_hook(
    nCode: i32,
    wParam: Foundation::WPARAM,
    lParam: Foundation::LPARAM,
) -> Foundation::LRESULT {
    // From the above docs: If nCode is less than zero, this callback must call CallNextHookEx
    // and return the result.
    if nCode < 0 {
        unsafe {
            return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    let hookstruct: WindowsAndMessaging::MSLLHOOKSTRUCT =
        unsafe { *(lParam.0 as *const WindowsAndMessaging::MSLLHOOKSTRUCT) };

    // Filter out any synthesized inputs to:
    //  1.  Avoid responding to our own inputs (note: could also do this with dwExtraInfo)
    //  2.  Avoid responding to inputs from something like AHK; this could create a loop depending
    //      on how Reemap and AHK are configured
    if hookstruct.flags & WindowsAndMessaging::LLMHF_INJECTED != 0 {
        unsafe {
            return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
        }
    }

    // Convert to an input and call the function.
    // If it's intercepted, do not let this message pass on.
    todo!()
    // if intercept(Input::Mouse(MouseButtonInput::Left)) {
    //     return Foundation::LRESULT(1);
    // }
    // unsafe {
    //     return WindowsAndMessaging::CallNextHookEx(None, nCode, wParam, lParam);
    // }
}

/// The active layer toggles.
// #[derive(Debug, Default, Clone, PartialEq, Eq)]
// pub struct CurrentLayerToggles {
//     layers: HashMap<config::LayerCondition, usize>,
// }

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum HoldButtonState {
    NotHeld,
    HeldNoRemap,
    HeldWithRemap(Vec<Button>),
}

impl Default for HoldButtonState {
    fn default() -> Self {
        Self::NotHeld
    }
}

#[derive(Debug, Default, Clone)]
struct HookLocalData {
    button_state: EnumMap<HoldButton, HoldButtonState>,
    profile: ProfileState,
}

static HOOKLOCAL: Mutex<Option<HookLocalData>> = Mutex::new(None);

// Returns "true" if the input is intercepted.
// Refer to the above pseudocode.
fn intercept_hold_down_input(hold_button: HoldButton) -> bool {
    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
    let hook_local = hook_local
        .as_mut()
        .expect("local data should have been initialized");

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
            let target_keys: Vec<Input> = targets
                .iter()
                .filter_map(|btn| match btn {
                    Button::Key(key) => Some(key),
                    _ => None,
                })
                .map(|key| KeyInput::keydown_from(*key))
                .map(|keydown| Input::from(keydown))
                .collect();

            Input::send_batch(&target_keys);
            return true;
        }

        // Not held - this is a fresh input. Let's continue processing.
        HoldButtonState::NotHeld => {}
    }

    // Step 2
    // Update layers
    for layer in hook_local.profile.layers.iter_mut() {
        // Only update layers for which this button is a condition.
        if layer.condition.contains(&hold_button) {
            // All conditions met?
            if layer
                .condition
                .iter()
                .filter(|&condition| *condition != HoldButton::from(hold_button))
                .all(|condition| hook_local.button_state[*condition] != HoldButtonState::NotHeld)
            {
                // All conditions met. Let's enable/toggle this layer.
                match &layer.layer_type {
                    LayerType::Modifier => layer.enabled = true,
                    LayerType::Toggle => layer.enabled = !layer.enabled,
                }
            }
        }
    }

    // Step 3
    for layer in hook_local
        .profile
        .layers
        .iter()
        .filter(|layer| layer.enabled)
        .rev()
    {
        match &layer.policy[Button::from(hold_button)] {
            RemapPolicy::Defer => {}
            RemapPolicy::Remap(output) => {
                let target_buttons: Vec<Input> = output
                    .iter()
                    .map(|button| match button {
                        Button::Key(key) => Input::from(KeyInput::keydown_from(*key)),
                        Button::Mouse(mouse) => Input::from(MouseInput::mousedown_from(*mouse)),
                        Button::Wheel(wheel) => Input::from(MouseWheelInput::wheel_from(*wheel)),
                    })
                    .collect();
                Input::send_batch(&target_buttons);
                return true;
            }
            RemapPolicy::NoRemap => {
                return false;
            }
        }
    }
    match &hook_local.profile.base.policy[Button::from(hold_button)] {
        BaseRemapPolicy::Remap(output) => {
            let target_buttons: Vec<Input> = output
                .iter()
                .map(|button| match button {
                    Button::Key(key) => Input::from(KeyInput::keydown_from(*key)),
                    Button::Mouse(mouse) => Input::from(MouseInput::mousedown_from(*mouse)),
                    Button::Wheel(wheel) => Input::from(MouseWheelInput::wheel_from(*wheel)),
                })
                .collect();
            Input::send_batch(&target_buttons);
            return true;
        }
        BaseRemapPolicy::NoRemap => {
            return false;
        }
    }
}

// Returns "true" if the input is intercepted.
// Refer to the above pseudocode.
fn intercept_hold_up_input(hold_button: HoldButton) -> bool {
    let mut hook_local = HOOKLOCAL.lock().expect("mutex poisoned");
    let hook_local = hook_local
        .as_mut()
        .expect("local data should have been initialized");

    // Step 1
    // Update layers
    for layer in hook_local.profile.layers.iter_mut() {
        // Only update layers for which this button is a condition.
        // These layers are no longer active.
        if layer.condition.contains(&hold_button) {
            match &layer.layer_type {
                LayerType::Modifier => layer.enabled = false,
                LayerType::Toggle => (), // Toggle buttons not affected by keyup
            }
        }
    }

    // Step 2
    // See what this button was mapped to.
    // Note we never consult the profile. The original decision of what a button maps to is only
    // made when the button is first pressed.
    match &hook_local.button_state[hold_button] {
        // This button down was not intercepted, so let's not intercept the button up.
        HoldButtonState::HeldNoRemap | HoldButtonState::NotHeld => {
            return false;
        }

        // This button down was intercepted, so let's intercept the button up the same way.
        HoldButtonState::HeldWithRemap(targets) => {
            let target_keys: Vec<Input> = targets
                .iter()
                .filter_map(|button| match button {
                    Button::Key(key) => Some(Input::from(KeyInput::keyup_from(*key))),
                    Button::Mouse(mouse) => Some(Input::from(MouseInput::mouseup_from(*mouse))),
                    Button::Wheel(_wheel) => None, // Wheel input only sent on down press
                })
                .collect();

            Input::send_batch(&target_keys);
            return true;
        }
    }
}

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

    for layer in hook_local
        .profile
        .layers
        .iter()
        .filter(|layer| layer.enabled)
        .rev()
    {
        match &layer.policy[Button::from(tap_button)] {
            RemapPolicy::Defer => {}
            RemapPolicy::Remap(output) => {
                let target_buttons: Vec<Input> = output
                    .iter()
                    .flat_map(|button| match button {
                        Button::Key(key) => vec![
                            Input::from(KeyInput::keydown_from(*key)),
                            Input::from(KeyInput::keyup_from(*key)),
                        ]
                        .into_iter(),
                        Button::Mouse(mouse) => vec![
                            Input::from(MouseInput::mousedown_from(*mouse)),
                            Input::from(MouseInput::mouseup_from(*mouse)),
                        ]
                        .into_iter(),
                        Button::Wheel(wheel) => {
                            vec![Input::from(MouseWheelInput::wheel_from(*wheel))].into_iter()
                        }
                    })
                    .collect();
                Input::send_batch(&target_buttons);
                return true;
            }
            RemapPolicy::NoRemap => {
                return false;
            }
        }
    }
    match &hook_local.profile.base.policy[Button::from(tap_button)] {
        BaseRemapPolicy::Remap(output) => {
            let target_buttons: Vec<Input> = output
                .iter()
                .flat_map(|button| match button {
                    Button::Key(key) => vec![
                        Input::from(KeyInput::keydown_from(*key)),
                        Input::from(KeyInput::keyup_from(*key)),
                    ]
                    .into_iter(),
                    Button::Mouse(mouse) => vec![
                        Input::from(MouseInput::mousedown_from(*mouse)),
                        Input::from(MouseInput::mouseup_from(*mouse)),
                    ]
                    .into_iter(),
                    Button::Wheel(wheel) => {
                        vec![Input::from(MouseWheelInput::wheel_from(*wheel))].into_iter()
                    }
                })
                .collect();
            Input::send_batch(&target_buttons);
            return true;
        }
        BaseRemapPolicy::NoRemap => {
            return false;
        }
    }
}
