use super::HoldInputType;
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
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl From<MouseInput> for MouseButton {
    fn from(value: MouseInput) -> Self {
        value.button
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseInput {
    pub button: MouseButton,
    pub input_type: HoldInputType,
}

impl MouseInput {
    pub fn mousedown_from(button: MouseButton) -> Self {
        Self {
            button,
            input_type: HoldInputType::Down,
        }
    }
    pub fn mouseup_from(button: MouseButton) -> Self {
        Self {
            button,
            input_type: HoldInputType::Up,
        }
    }
}

impl From<MouseInput> for KeyboardAndMouse::INPUT {
    fn from(value: MouseInput) -> Self {
        todo!()
    }
}
