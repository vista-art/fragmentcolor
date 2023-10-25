use crate::renderer::Renderer;
use crate::scene::{camera::Camera, Scene};

pub trait RenderPass {
    fn draw(&mut self, scene: &Scene, camera: &Camera, context: &Renderer);
}
