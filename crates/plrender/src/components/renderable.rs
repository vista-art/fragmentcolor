use crate::{Color, Quad, TextureId};

/// The bounds of a 2D object in screen space.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Bounds(pub Quad);

/// The bounding radius of a 2D object in screen space.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Radius(pub f32);

/// The border width of a 2D object in screen space.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Border(pub f32);

/// Internal flag to indicate the type of shape to render.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ShapeFlag(pub f32);

/// The collection of components required to render a 2D object.
#[derive(hecs::Bundle, Clone, Copy, Debug, PartialEq)]
pub struct Renderable2D {
    pub color: Color,
    pub bounds: Bounds,
    pub border: Border,
    pub image: Option<TextureId>,
    pub sdf_flags: ShapeFlag,
}
