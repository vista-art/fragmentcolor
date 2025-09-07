use crate::Size;
use super::{HasDisplaySize, TestWindow};

pub enum TargetInput {
    Surface {
        target: wgpu::SurfaceTarget<'static>,
        size: wgpu::Extent3d,
    },
    Texture(Size),
}

impl<T> From<T> for TargetInput
where
    T: Into<wgpu::SurfaceTarget<'static>> + Clone + HasDisplaySize,
{
    fn from(window: T) -> Self {
        let size = window.size();
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let target = window.clone().into();
        TargetInput::Surface { target, size: extent }
    }
}

impl From<TestWindow> for TargetInput {
    fn from(w: TestWindow) -> Self {
        TargetInput::Texture(w.size)
    }
}

impl From<Size> for TargetInput {
    fn from(size: Size) -> Self {
        TargetInput::Texture(size)
    }
}

impl From<[u32; 2]> for TargetInput {
    fn from(size: [u32; 2]) -> Self {
        TargetInput::Texture(size.into())
    }
}

impl From<(u32, u32)> for TargetInput {
    fn from(size: (u32, u32)) -> Self {
        TargetInput::Texture([size.0, size.1].into())
    }
}

