#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
    antialiaser: f32,
    resolution: [f32; 2],
    _padding: f32,
}

impl ScreenUniform {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            antialiaser: 2.0 / f32::min(width, height),
            resolution: [width, height],
            _padding: 0.0,
        }
    }
}
