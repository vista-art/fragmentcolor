mod buffer;
mod flat;
mod phong;
mod real;
mod solid;
mod toy;

pub(crate) use flat::*;
pub(crate) use phong::*;
pub(crate) use real::*;
pub(crate) use solid::*;
pub(crate) use toy::*;

use crate::{
    renderer::{Commands, RenderedFrames},
    scene::SceneState,
};
use std::sync::RwLockReadGuard;

pub(crate) type RenderPassResult = Result<(Commands, RenderedFrames), wgpu::SurfaceError>;

pub(crate) trait RenderPass {
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult;
}
