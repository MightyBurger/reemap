/*
                                A theory of inputs, I guess.

    Here, we make a distinction between buttons and inputs. A button is just something that
    corresponds to some key, button, etc. you actuate on a physical device. Buttons generate inputs.

    An "input" happens when the user presses or releases a button. An input can be thought of as an
    event, or message, carrying information that a button was pressed, released, or actuated.

    Windows gives inputs to games. Reemap is a low-level hook, which means Windows first gives the
    input to Reemap. Reemap decides what to do with the input, possibly including blocking it from
    moving further and instead sending its own set of inputs.

    Inputs and buttons are closely related but are distinct. Every input is generated because the
    user actuated a button. Each input contains which button it corresponds to, plus other useful
    information like whether the button was pressed or released to generate this input.

    As a user, it makes sense to map buttons to other buttons. It makes less sense to map inputs to
    other inputs. The background service works with inputs, however.

    In Reemap, the user uses the GUI to create a button map: Button -> Button x N. This map is
    handed over to the background service, which then uses the button map to determine what to do
    with each input the background service receives.

    -----

    There is another distinction that must be made. There are two different types of inputs. I call
    them Hold Buttons and Tap Buttons.

    Almost all buttons are Hold Buttons. Hold Buttons are buttons where "holding it down" makes
    sense. Inputs corresponding to Hold Buttons can be "down" or "up" inputs. Windows generates
    "down" inputs when the user presses a button and "up" inputs when the user releases the button.

    Tap buttons are a special exception. Tap buttons are buttons where "holding it down"
    is meaningless. Pretty much the only button like this is the scroll wheel. You can scroll up,
    but you can't "hold" a scroll up. Each scroll input is a transient event.

    If you read the Windows API, you will find Windows input messages are structured the same way.
    Keyboard inputs are given in either KEYDOWN or KEYUP messages. Every mouse input also comes in
    DOWN and UP pairs like the RBUTTONDOWN and RBUTTONUP messages. As the sole exception, the scroll
    wheel only has the MOUSEWHEEL message.

    -----

    There is yet another distinction to be made. Mouse and keyboard buttons behave a little
    differently in Windows. When a keyboard button is held down, Windows repeatedly generates
    KEYDOWN inputs according to the user's key repeat settings. This does not happen for mouse
    inputs.

    This is a key factor in why mapping mouse buttons to keyboard buttons inside X-Mouse produces
    different results for Ori glitches like double-bash compared to doing this mapping in vendor
    mouse software. One results in key repeat behavior, and the other does not.

    This has implications for how Reemap should handle remaps between these inputs.

    Maps from keyboard buttons to others:

    1.  key -> key

        This should preserve key repeat behavior. Simply put, Reemap should just transform and send
        every KEYDOWN message.

    2.  key -> mouse

        This should suppress key repeat behavior. Because mouse inputs don't have repeat behavior,
        games might treat each DOWN input as a separate button press, and the user might be able to
        achieve a turbo effect.

        In practice, this means Reemap should send the DOWN input only on the first KEYDOWN, but
        not on later KEYDOWNs. Reemap should send the UP input on KEYUP.

    3.  key -> scroll

        This should most definitely suppress key repeat behavior. Otherwise, the user could achieve
        turbo behavior by exploiting key repeat.

        Reemap should send the scroll input only on the first KEYDOWN, not on later KEYDOWNs.

    Maps from mouse buttons to others:

    4.  mouse -> key

        This is the tricky one.

        Theoretically, this map should produce key repeat behavior. This is what happens when you
        remap mouse buttons to keys in mouse vendor software, which produce key repeat in driver
        software. Skipping over much detail, certain styles of double-bash used by some high-level
        runners rely on this key-repeat behavior.

        Achieving this, however, requires timed inputs. There is much discussion that could be had,
        but what it boils down to is this is kind of a gray area with respect to leaderboard rules.
        My opinion is it should be fine, but when I brought it up with the community, there wasn't
        a solid yes or no.

        One of Reemap's primary goals is to be unquestionably allowed per leaderboard rules. So,
        to implement this feature, we'd likely want community consensus on it's validity.
        Unfortunately, getting any kind of consensus from the Ori speedrunning community is a
        herculean task. Every discussion on leaderboard rules in this community ends with thousands
        of messages, a bunch of fuming people, and no progress. So, I really, really do not feel
        like seeking some consensus from this community, especially for a nuanced, highly technical
        topic such as this.

        So, while I am a little annoyed about the situation, I have made the engineering / political
        decision Reemap will not emulate key repeat, at least for the time being. This maintains
        Reemap's usefulness as an uncontroversial remap software useful for leaderboard runs.
        Unfortunately, the cost is Reemap will have the same flaw as X-Mouse: to achieve the
        specific flavor of double-bash key repeat unlocks, you will still need to use mouse vendor
        software. For me, that means manually reconfiguring my mouse every time I want to do a
        speedrun. This kills one reason I was making this software in the first place. :(

        Now, what were we doing? Oh right - software.

        Reemap will send a KEYDOWN on each mouse DOWN input, and a KEYUP on each mouse
        UP input.

    5.  mouse -> mouse

        Simple; because key repeat doesn't happen for mouse inputs, Reemap does not need to worry
        about it.

        Reemap should send DOWN inputs in response to each DOWN input, and UP inputs in response to
        each UP input.

    6.  mouse -> scroll

        Again, since key repeat doesn't happen for mouse inputs, Reemap does not need to worry about
        it.

        Reemap should send scroll inputs in response to each DOWN input.

    Maps from scroll to others:

    7.  scroll -> key

        Simply send a KEYDOWN and KEYUP pair on each scroll input.

    8.  scroll -> mouse

        Simply send the corresponding DOWN/UP input pair on each scroll input.

    9.  scroll -> scroll

        Just send the scroll input in response to a scroll input.

*/

pub mod key;
use key::KeyButton;
pub mod mouse;
use mouse::MouseButton;
pub mod wheel;
use serde::{Deserialize, Serialize};
use wheel::MouseWheelButton;

// -------------------------------------------------------------------------------------------------

// -------------------- Hold Button --------------------

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum, Serialize, Deserialize,
)]
pub enum HoldButton {
    Key(KeyButton),
    Mouse(MouseButton),
}

impl From<KeyButton> for HoldButton {
    fn from(value: KeyButton) -> Self {
        Self::Key(value)
    }
}

impl From<MouseButton> for HoldButton {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl std::fmt::Display for HoldButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(key_button) => write!(f, "{key_button}"),
            Self::Mouse(mouse_button) => write!(f, "{mouse_button}"),
        }
    }
}

// -------------------- Tap Button --------------------

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum, Serialize, Deserialize,
)]
pub enum TapButton {
    Wheel(MouseWheelButton),
}

impl From<MouseWheelButton> for TapButton {
    fn from(value: MouseWheelButton) -> Self {
        Self::Wheel(value)
    }
}

impl std::fmt::Display for TapButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wheel(mouse_wheel_button) => write!(f, "{mouse_wheel_button}"),
        }
    }
}

// -------------------- Button --------------------

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum, Serialize, Deserialize,
)]
pub enum Button {
    Key(KeyButton),
    Mouse(MouseButton),
    Wheel(MouseWheelButton),
}

impl From<KeyButton> for Button {
    fn from(value: KeyButton) -> Self {
        Self::Key(value)
    }
}

impl From<MouseButton> for Button {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl From<MouseWheelButton> for Button {
    fn from(value: MouseWheelButton) -> Self {
        Self::Wheel(value)
    }
}

impl From<HoldButton> for Button {
    fn from(value: HoldButton) -> Self {
        match value {
            HoldButton::Key(key) => Self::Key(key),
            HoldButton::Mouse(mouse) => Self::Mouse(mouse),
        }
    }
}

impl From<TapButton> for Button {
    fn from(value: TapButton) -> Self {
        match value {
            TapButton::Wheel(wheel) => Self::Wheel(wheel),
        }
    }
}

impl std::fmt::Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(key_button) => write!(f, "{key_button}"),
            Self::Mouse(mouse_button) => write!(f, "{mouse_button}"),
            Self::Wheel(mouse_wheel_button) => write!(f, "{mouse_wheel_button}"),
        }
    }
}

// -------------------- ListableButton Trait --------------------
// (the name is bad)

pub trait ListableButton: PartialEq + std::fmt::Display {
    fn device(&self) -> &str;
}

impl ListableButton for Button {
    fn device(&self) -> &str {
        match self {
            Self::Key(_) => "Keyboard",
            Self::Mouse(_) | Self::Wheel(_) => "Mouse",
        }
    }
}

impl ListableButton for HoldButton {
    fn device(&self) -> &str {
        match self {
            Self::Key(_) => "Keyboard",
            Self::Mouse(_) => "Mouse",
        }
    }
}

impl ListableButton for TapButton {
    fn device(&self) -> &str {
        match self {
            Self::Wheel(_) => "Mouse",
        }
    }
}
