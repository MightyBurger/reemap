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
#[allow(non_camel_case_types)] // Just for here, to be consistent with VK codes...
pub enum KeyButton {
    LBUTTON = 0x01, // TODO check
    RBUTTON = 0x02, // TODO check
    // CANCEL = 0x03,   // Control-break processing... ?????
    MBUTTON = 0x04,  // TODO check
    XBUTTON1 = 0x05, // TODO check
    XBUTTON2 = 0x06, // TODO check
    // Reserved = 0x07,
    BACK = 0x08,
    TAB = 0x09,
    // Reserved = 0x0A,
    // Reserved = 0x0B,
    CLEAR = 0x0C, // What is this??
    RETURN = 0x0D,
    // Reserved = 0x0E,
    // Reserved = 0x0F,
    SHIFT = 0x10,   // TODO check - not left or right?
    CONTROL = 0x11, // TODO check
    MENU = 0x12,    // TODO check (just one alt)
    PAUSE = 0x13,   // TODO check
    CAPITAL = 0x14,
    KANA_HANGUL = 0x15, // TODO unusual
    IME_ON = 0x16,      // TODO unusual
    JUNJA = 0x17,       // TODO unusual
    FINAL = 0x18,       // TODO unusual
    HANJA_KANJI = 0x19, // TODO unusual
    IME_OFF = 0x1A,     // TODO unusual
    ESCAPE = 0x1B,

    SPACE = 0x20,
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
    LSHIFT = 0xA0,
    RSHIFT = 0xA1,
    LCONTROL = 0xA2,
    RCONTROL = 0xA3,
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
        let name = match self {
            Self::LBUTTON => "Left Click as key",
            Self::RBUTTON => "Right Click as key",
            // Self::CANCEL =>  "Cancel",
            Self::MBUTTON => "Middle Click as key",
            Self::XBUTTON1 => "Mouse X1 as key",
            Self::XBUTTON2 => "Mouse X2 as key",
            Self::BACK => "Backspace",
            Self::TAB => "Tab",
            Self::CLEAR => "Clear",
            Self::RETURN => "Enter",
            Self::SHIFT => "Shift (ambidextrous)",
            Self::CONTROL => "Ctrl (ambidextrous)",
            Self::MENU => "Alt (ambidextrous)",
            Self::PAUSE => "Pause",
            Self::CAPITAL => "Caps Lock",
            Self::KANA_HANGUL => "IME Kana/Hangul",
            Self::IME_ON => "IME On",
            Self::JUNJA => "IME Junja",
            Self::FINAL => "IME Final",
            Self::HANJA_KANJI => "IME Hanja/Kanji",
            Self::IME_OFF => "IME Off",
            Self::ESCAPE => "Escape",

            Self::SPACE => "Space",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::LSHIFT => "Left Shift",
            Self::RSHIFT => "Right Shift",
            Self::LCONTROL => "Left Ctrl",
            Self::RCONTROL => "Right Ctrl",
        };
        write!(f, "{name}")
    }
}
