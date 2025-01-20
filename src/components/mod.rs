#![allow(clippy::new_ret_no_self)]

/// Camera and Projection Components.
///
/// A Camera is the link between the Scene and the Renderer.
///
/// It contains the inputs for building a Projection matrix,
/// the near and far clip distances in Scene units, and the
/// reference for the Scene's Transform that owns the camera.
mod camera;

/// Color component.
///
/// Allows an Object to have a color.
mod color;

/// Controller component.
///
/// Allows an Object to respond to Keyboard and Mouse events.
mod controller;

/// Empty component
///
/// Creates an empty object with spatial information that can
/// be used as a parent for other objects.
///
/// Mathematically, this is equivalent to a Point.
mod empty;

/// Hidden component.
///
/// This component is a marker that does not contain any data.
/// The presence of this component in an Object will make the
/// renderer ignore it.
mod is_hidden;

/// Light component.
///
/// Allows an Object to emit light.
/// Currently not supported in 2D Scenes.
mod light;

/// Mesh component.
///
/// Allows an Object to display a 3D Mesh
mod mesh;

/// Shader component.
///
/// In the current implementation, it creates a ShaderToy-like
/// fullscreen fragment shader.
///
/// It is planned feature to make it change the appearance of one object.
mod shader;

/// # Sprite component.
///
/// Allows an Object to display an image or video frame.
///
/// The Sprite can display the entire image or a clipped part of it.
///
/// A Sprite can also be animated using the Animator component, which
/// will change its clip region coordinates to display a different
/// part of the image in timed intervals.
mod sprite;

/// Renderable component.
///
/// They can change an object behavior or how the renderer deals
/// with them. They contain no data. Examples: `Hidden`, `Is2D` `Is3D`.
mod renderable;

pub use camera::*;
pub use color::*;
pub use controller::*;
pub use empty::*;
pub use is_hidden::*;
pub use light::*;
pub use mesh::*;
pub use renderable::*;
pub use shader::*;
pub use sprite::*;
