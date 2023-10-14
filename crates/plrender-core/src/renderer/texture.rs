// This is essentially a TextureView
pub struct Image {
    pub view: wgpu::TextureView,
    pub size: wgpu::Extent3d,
}

pub struct ImageInfo {
    pub size: mint::Vector2<i16>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageRef(pub u32);
