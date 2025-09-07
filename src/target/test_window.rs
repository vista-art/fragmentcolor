use crate::{Size, Target, TargetFrame};

pub struct TestWindow {
    inner: crate::TextureTarget,
}

impl From<crate::TextureTarget> for TestWindow {
    fn from(inner: crate::TextureTarget) -> Self {
        Self { inner }
    }
}

impl TestWindow {
    pub fn size(&self) -> [u32; 2] {
        let s = <Self as Target>::size(self);
        [s.width, s.height]
    }
}

impl Target for TestWindow {
    fn size(&self) -> Size {
        self.inner.size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        <crate::TextureTarget as Target>::resize(&mut self.inner, size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.inner.get_current_frame()
    }

    fn get_image(&self) -> Vec<u8> {
        <crate::TextureTarget as Target>::get_image(&self.inner)
    }
}
