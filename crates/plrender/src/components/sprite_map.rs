use crate::math::geometry::Quad;

/// A common ancestor of "sprite sheet", "tile map".
pub struct SpriteMap {
    pub origin: mint::Point2<u16>,
    pub cell_size: mint::Vector2<u16>,
}

impl SpriteMap {
    pub fn at(&self, index: mint::Point2<usize>) -> Quad {
        Quad::from_region(
            index.x as u32,
            index.y as u32,
            self.cell_size.x as u32,
            self.cell_size.y as u32,
        )
    }
}
