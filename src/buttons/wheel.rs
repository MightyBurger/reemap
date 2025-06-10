use windows::Win32::UI::Input::KeyboardAndMouse;

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    enum_map::Enum,
    strum::EnumIter,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[repr(u8)]
pub enum MouseWheelButton {
    ScrollUp,
    ScrollDown,
    ScrollHorzLeft,
    ScrollHorzRight,
}

impl From<MouseWheelInput> for MouseWheelButton {
    fn from(value: MouseWheelInput) -> Self {
        value.button
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseWheelInput {
    pub button: MouseWheelButton,
}

// note: intentionally not an impl From<> to be consistent with other button-to-input conversions
impl MouseWheelInput {
    pub fn wheel_from(button: MouseWheelButton) -> Self {
        Self { button }
    }
}

impl From<MouseWheelInput> for KeyboardAndMouse::INPUT {
    fn from(value: MouseWheelInput) -> Self {
        todo!()
    }
}
