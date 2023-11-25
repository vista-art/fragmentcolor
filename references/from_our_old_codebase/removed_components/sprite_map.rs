use crate::math::{geometry::Quad, Pixel};

/// A common ancestor of "sprite sheet", "tile map".

#[derive(Debug, Clone)]
pub struct SpriteMap {
    pub origin: Pixel,
    pub cell_size: Pixel,
}

unsafe impl Send for SpriteMap {}

impl SpriteMap {
    pub fn at(&self, index: Pixel) -> Quad {
        Quad::from_region(
            index.x as u32,
            index.y as u32,
            self.cell_size.x as u32,
            self.cell_size.y as u32,
        )
    }
}
