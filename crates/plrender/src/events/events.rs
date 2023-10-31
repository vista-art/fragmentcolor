#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    Left,
    Middle,
    Right,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Resize { width: u32, height: u32 },
    Keyboard { key: Key, pressed: bool },
    Pointer { position: mint::Vector2<f32> },
    Scroll { delta: mint::Vector2<f32> },
    Click { button: Button, pressed: bool },
    Command(Command),
    Draw,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    NewWindow,
    Exit,
}
