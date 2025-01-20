use crate::{
    renderer::{RenderTarget, Renderer},
    scene::Scene,
    Camera,
};

pub trait Pass {
    fn draw(
        &mut self,
        renderer: &Renderer,
        scene: &Scene,
        camera: &Camera,
        targets: &[RenderTarget],
    );
}
