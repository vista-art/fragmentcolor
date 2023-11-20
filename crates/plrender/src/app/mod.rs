//! # PLRender Application Module
//!
//! This module contains the main App struct and its related types.
//!
//! The main App instance is a singleton responsible for managing the global
//! resources of this Library, namely the main Event Loop and the Renderer.
//!
//! It also contains internal utility functions, build metadata, errors, and
//! the Window/Canvas object to create and manage windows.

/// The **App** module contains the global **PLRender** accessor, which will
/// lazily initialize the App and the Renderer.
///
/// The only methods from this module users typically need to care about are:
/// - `PLRender::config()`
/// - `PLRender::run()`
///
/// # Examples
///
/// ## Typical usage
///
/// ```
/// use plrender::{PLRender, AppOptions};
///
/// // User configures the App (optional)
/// PLRender::config(AppOptions {
///    ..Default::default()
/// });
///
/// // User creates stuff...
///
/// // User runs the App
/// PLRender::run();
/// ```
pub mod app;

/// Defines a list of commands that can be issued to the App's Event Loop.
///
/// Users can send events to the App by calling `app.command()`.
pub mod commands;

/// The Container is a HashMap that stores the given Type in a `Arc<RwLock<T>>`
/// and returns a ready-to-use `RwLockReadGuard` or `RwLockWriteGuard` when the
/// user calls `get` or `get_mut` respectively.
///
/// This is an internal module, API users do not need to use it directly.
pub(crate) mod container;

/// Handy internal macro to implement the `Container` trait for a type.
pub(crate) mod macros;

/// Defines internal Error strings.
pub(crate) mod error;

/// Event Loop module.
///
/// This is the core of the App lifecycle and its communication channel
/// with the OS. For multiplatform compatibility reasons, only one
/// instance of the Event Loop can exist at a time and it needs
/// to run in the main thread.
///
/// PLRender initializes main Event Loop when the user calls
/// `PLRender::run()`. This function never returns, it will
/// live as long as the App is running and only exits when
/// all windows are closed.
pub mod event_loop;

/// Events module.
///
/// Defines the main events that can be emitted by the App,
/// mainly Window events and the Draw event.
pub mod events;

/// Meta module with static Build data.
pub mod meta;

/// The Window module.
///
/// This module wraps both a OS Window and a Web Canvas.
///
/// It manages the Window / Canvas lifecycle and events.
/// Desktop users might use the Window directly, but Web
/// users do not need to use it because it automatically
/// wraps a native `<canvas>` element.
pub mod window;

pub use app::*;
pub use commands::*;
pub use container::*;
pub use event_loop::*;
pub use events::*;
pub use meta::*;
pub use window::*;
