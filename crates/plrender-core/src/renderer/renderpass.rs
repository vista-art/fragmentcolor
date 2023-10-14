use crate::renderer::{target::TargetRef, Context};
use crate::scene::{camera::Camera, Scene};

pub trait RenderPass {
    fn draw(&mut self, targets: &[TargetRef], scene: &Scene, camera: &Camera, context: &Context);
}
