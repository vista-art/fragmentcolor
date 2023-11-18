//! Components library for the Entity Component System.
//!
//! Typically, users do not need to use the components directly,
//! but rather use the `SceneObject` struct, which is a wrapper
//! around spatial components which can be added to a Scene.
//!
//! Types of Components:
//! - SceneObject (spatial components)
//!     - Sprite
//!     - Shape
//!     - Mesh
//!
//! - Marker components

/// Animation component.
///
/// This component is used to animate a sprite atlas.
/// It has not been converted to work with a SceneObject yet.
///
/// I plan to make any SceneObject that has "Sprite" and "Animator"
/// components to be able to use the methods in this component.
///
/// The original was copied from the examples folder of the legacy
/// engine I forked. It was not supposed to be a component in the
/// original engine, but an implementation detail.
pub mod animation;

/// Camera component.
pub mod camera;

/// Color component.
///
/// Allows a SceneObject to have a color.
pub mod color;

/// Controller component.
///
/// Allows a SceneObject to respond to Keyboard and Mouse events.
pub mod controller;

/// Empty component.
pub mod empty;

/// Light component.
pub mod light;

/// Mesh component.
pub mod mesh;

/// # Sprite component.
///
/// Allows a SceneObject to display an image or video frame.
///
/// The Sprite can display the entire image or a clipped part of it.
///
/// A Sprite can also be animated using the Animator component, which
/// will change the UV coordinates of the Sprite to display a different
/// part of the image in timed intervals.
pub mod sprite;

/// # SpriteMap component.
///
/// Configures the CellSize and CellCount of a Sprite, and allows
/// the Animator component to access the UV coordinates of each cell.
pub mod sprite_map;

/// Transform is a reserved component, mandatory for every SceneObject.
///
/// SceneObjects contain a Node, which is a struct containing a Transform
/// component and a parent NodeId. This allows the Scene to calculate the
/// global positions and orientations of every object in the screen.
pub mod transform;

pub use animation::*;
pub use camera::*;
pub use color::*;
pub use controller::*;
pub use empty::*;
pub use light::*;
pub use mesh::*;
pub use sprite::*;
pub use sprite_map::*;
pub use transform::*;
