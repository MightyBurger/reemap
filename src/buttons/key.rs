use serde::{Deserialize, Serialize};
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
    Serialize,
    Deserialize,
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

impl KeyButton {
    pub fn from_vk(vk: u8) -> Option<Self> {
        use num_traits::FromPrimitive;
        Self::from_u8(vk)
    }
    pub fn to_vk(self) -> u8 {
        use num_traits::ToPrimitive;
        self.to_u8()
            .expect("button should always be convertable to virtual key code")
    }
    pub fn to_keydown_input(self) -> KeyboardAndMouse::INPUT {
        use KeyboardAndMouse as KBM;
        let vk = self.to_vk();
        KBM::INPUT {
            r#type: KBM::INPUT_KEYBOARD,
            Anonymous: KBM::INPUT_0 {
                ki: KBM::KEYBDINPUT {
                    wVk: KBM::VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KBM::KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
    pub fn to_keyup_input(self) -> KeyboardAndMouse::INPUT {
        use KeyboardAndMouse as KBM;
        let vk = self.to_vk();
        KBM::INPUT {
            r#type: KBM::INPUT_KEYBOARD,
            Anonymous: KBM::INPUT_0 {
                ki: KBM::KEYBDINPUT {
                    wVk: KBM::VIRTUAL_KEY(vk as u16),
                    wScan: 0,
                    dwFlags: KBM::KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

impl std::fmt::Display for KeyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftShift => write!(f, "Left Shift"),
            Self::RightShift => write!(f, "Right Shift"),
            Self::Space => write!(f, "Space"),
            Self::LeftCtrl => write!(f, "Left Ctrl"),
            Self::RightCtrl => write!(f, "Right Ctrl"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::G => write!(f, "G"),
            Self::H => write!(f, "H"),
            Self::I => write!(f, "I"),
            Self::J => write!(f, "J"),
            Self::K => write!(f, "K"),
            Self::L => write!(f, "L"),
            Self::M => write!(f, "M"),
            Self::N => write!(f, "N"),
            Self::O => write!(f, "O"),
            Self::P => write!(f, "P"),
            Self::Q => write!(f, "Q"),
            Self::R => write!(f, "R"),
            Self::S => write!(f, "S"),
            Self::T => write!(f, "T"),
            Self::U => write!(f, "U"),
            Self::V => write!(f, "V"),
            Self::W => write!(f, "W"),
            Self::X => write!(f, "X"),
            Self::Y => write!(f, "Y"),
            Self::Z => write!(f, "Z"),
        }
    }
}
