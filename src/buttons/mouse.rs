use windows::Win32::UI::Input::KeyboardAndMouse;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum, strum::EnumIter,
)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

impl MouseButton {
    pub fn to_mousedown_input(&self) -> KeyboardAndMouse::INPUT {
        use KeyboardAndMouse as KBM;
        use windows::Win32::UI::WindowsAndMessaging as WM;

        let (dw_flags, mouse_data): (KBM::MOUSE_EVENT_FLAGS, u32) = match self {
            Self::Left => (KBM::MOUSEEVENTF_LEFTDOWN, 0),
            Self::Middle => (KBM::MOUSEEVENTF_MIDDLEDOWN, 0),
            Self::Right => (KBM::MOUSEEVENTF_RIGHTDOWN, 0),
            Self::X1 => (KBM::MOUSEEVENTF_XDOWN, WM::XBUTTON1 as u32),
            Self::X2 => (KBM::MOUSEEVENTF_XDOWN, WM::XBUTTON2 as u32),
        };

        KBM::INPUT {
            r#type: KBM::INPUT_MOUSE,
            Anonymous: KBM::INPUT_0 {
                mi: KBM::MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data,
                    dwFlags: dw_flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
    pub fn to_mouseup_input(&self) -> KeyboardAndMouse::INPUT {
        use KeyboardAndMouse as KBM;
        use windows::Win32::UI::WindowsAndMessaging as WM;

        let (dw_flags, mouse_data): (KBM::MOUSE_EVENT_FLAGS, u32) = match self {
            Self::Left => (KBM::MOUSEEVENTF_LEFTUP, 0),
            Self::Middle => (KBM::MOUSEEVENTF_MIDDLEUP, 0),
            Self::Right => (KBM::MOUSEEVENTF_RIGHTUP, 0),
            Self::X1 => (KBM::MOUSEEVENTF_XUP, WM::XBUTTON1 as u32),
            Self::X2 => (KBM::MOUSEEVENTF_XUP, WM::XBUTTON2 as u32),
        };

        KBM::INPUT {
            r#type: KBM::INPUT_MOUSE,
            Anonymous: KBM::INPUT_0 {
                mi: KBM::MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data,
                    dwFlags: dw_flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

impl std::fmt::Display for MouseButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "Left Click"),
            Self::Middle => write!(f, "Middle Click"),
            Self::Right => write!(f, "Right Click"),
            Self::X1 => write!(f, "Mouse X1"),
            Self::X2 => write!(f, "Mouse X2"),
        }
    }
}
