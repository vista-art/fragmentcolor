use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Key {
    Digit(u8),
    Letter(char),
    Function(u8),
    Up,
    Down,
    Left,
    Right,
    Space,
    Escape,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Button {
    Left,
    Middle,
    Right,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Resize { width: u32, height: u32 },
    Keyboard { key: Key, pressed: bool },
    Pointer { x: f32, y: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
    Click { button: Button, pressed: bool },
    Draw,
    Exit,
}
