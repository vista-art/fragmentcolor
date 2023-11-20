use crate::app::commands::Command;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, RwLock};
use winit::event::{MouseButton, VirtualKeyCode};

/// Trait for PLRender Callback Functions.
///
/// Because we interact with Javascript and Python, all callback
/// functions must be safely shared between threads, so they must
/// implement `Send` and `Sync`.
///
/// This trait is automatically implemented for all functions
/// that are `FnMut(E) + Send + Sync`.
pub trait CallbackFn<E>: FnMut(E) + Send + Sync {}
impl<E, F> CallbackFn<E> for F where F: FnMut(E) + Send + Sync {}

/// Implements Debug for Callback Functions.
///
/// We can't know the type of the function at compile time, so
/// we return a generic string when it is printed to the console.
impl<E> Debug for dyn CallbackFn<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "< PLRender Callback Function >")
    }
}

/// External Callback Functions are stored internally
/// as a Reference-Counted Read-Write mutex Lock.
pub type Callback<E> = Arc<RwLock<dyn CallbackFn<E>>>;

/// Alias for Winit's VirtualKeyCode.
pub type VirtualKey = VirtualKeyCode;

/// PLRender Events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Command(Command),

    Resized {
        width: u32,
        height: u32,
    },
    Rescaled {
        scale: f64,
        width: u32,
        height: u32,
    },
    Moved {
        x: i32,
        y: i32,
    },

    KeyUp {
        key: Option<VirtualKey>,
        keycode: u32,
    },
    KeyDown {
        key: Option<VirtualKey>,
        keycode: u32,
    },
    Character {
        character: char,
    },

    CursorEntered,
    CursorLeft,
    Pointer {
        x: f32,
        y: f32,
    },
    Scroll {
        delta_x: f32,
        delta_y: f32,
    },
    Click {
        button: MouseButton,
        pressed: bool,
    },

    FileHovered {
        handle: u128,
    },
    FileDropped {
        handle: u128,
    },
    FileHoverCancelled,

    Focus {
        focused: bool,
    },

    Closed,
    Destroyed,
    Draw,
    Exit,
}
