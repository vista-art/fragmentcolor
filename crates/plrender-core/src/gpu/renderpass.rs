use crate::gpu::{target::TargetRef, Context};
use crate::scene::{camera::Camera, Scene};

// @TODO this will be renamed to RenderPass
// I'll keep both trait definitions for now
// until I change it in the other crates
pub trait Pass {
    fn draw(&mut self, targets: &[TargetRef], scene: &Scene, camera: &Camera, context: &Context);
}

pub trait RemderPass {
    fn draw(&mut self, targets: &[TargetRef], scene: &Scene, camera: &Camera, context: &Context);
}
