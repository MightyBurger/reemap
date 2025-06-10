use windows::Win32::UI::Input::KeyboardAndMouse;

use super::HoldInputType;

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
pub enum KeyButton {
    LeftShift = 0xA0,
    RightShift = 0xA1,
    Space = 0x20,
    LeftCtrl = 0xA2,
    RightCtrl = 0xA3,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
}

impl From<KeyInput> for KeyButton {
    fn from(value: KeyInput) -> Self {
        value.button
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyInput {
    pub button: KeyButton,
    pub input_type: HoldInputType,
}

impl KeyInput {
    pub fn keydown_from(button: KeyButton) -> Self {
        Self {
            button,
            input_type: HoldInputType::Down,
        }
    }
    pub fn keyup_from(button: KeyButton) -> Self {
        Self {
            button,
            input_type: HoldInputType::Up,
        }
    }
}

impl From<KeyInput> for KeyboardAndMouse::INPUT {
    fn from(value: KeyInput) -> Self {
        todo!()
    }
}
