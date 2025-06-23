use windows::Win32::UI::Input::KeyboardAndMouse;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum, strum::EnumIter,
)]
pub enum MouseWheelButton {
    ScrollUp,
    ScrollDown,
    ScrollHorzRight,
    ScrollHorzLeft,
}

impl MouseWheelButton {
    // note: not an impl From<> to be consistent with the other button types
    pub fn to_input(&self) -> KeyboardAndMouse::INPUT {
        use KeyboardAndMouse as KBM;
        use windows::Win32::UI::WindowsAndMessaging as WM;

        // This is gross. The function takes in an unsigned number, but we need
        // a signed one. Bear with me.
        let plus_click: u32 = WM::WHEEL_DELTA;
        let minus_click: u32 = -(WM::WHEEL_DELTA as i32) as u32;

        let (dw_flags, mouse_data): (KBM::MOUSE_EVENT_FLAGS, u32) = match self {
            Self::ScrollUp => (KBM::MOUSEEVENTF_WHEEL, plus_click),
            Self::ScrollDown => (KBM::MOUSEEVENTF_WHEEL, minus_click),
            Self::ScrollHorzRight => (KBM::MOUSEEVENTF_HWHEEL, plus_click),
            Self::ScrollHorzLeft => (KBM::MOUSEEVENTF_HWHEEL, minus_click),
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

impl std::fmt::Display for MouseWheelButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScrollUp => write!(f, "Scroll Up"),
            Self::ScrollDown => write!(f, "Scroll Down"),
            Self::ScrollHorzRight => write!(f, "Scroll Right"),
            Self::ScrollHorzLeft => write!(f, "Scroll Left"),
        }
    }
}
