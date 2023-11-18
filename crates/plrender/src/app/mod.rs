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
/// ## Typical usage
///
/// ```rust
/// use plrender::PLRender;
///
/// // User configures the App (optional)
/// PLRender::config({
///    // optional log level setup
///    // optional Renderer setup
/// });
///
/// // ... User creates stuff...
///
/// // User runs the App
/// PLRender::run();
/// ```
pub mod app;

/// The Container is a HashMap that stores the given Type in a `Arc<RwLock<T>>`
/// and returns a ready-to-use `RwLockReadGuard` or `RwLockWriteGuard` when the
/// user calls `get` or `get_mut` respectively.
///
/// This is an internal module, users do not need to use it directly.
pub(crate) mod container;

/// Handy internal macro to implement the `Container` trait for the given type.
pub(crate) mod macros;

/// Errors module.
///
/// @TODO describe it
pub mod error;

/// Event Loop module
/// .
/// @TODO describe it
pub mod event_loop;

/// Events module.
///
/// @TODO describe it
pub mod events;

/// Meta module with static Build data.
///
/// @TODO describe it
pub mod meta;

/// Window module.
///
/// @TODO describe it
pub mod window;

pub use app::*;
pub use container::*;
pub use event_loop::*;
pub use events::*;
pub use window::*;
