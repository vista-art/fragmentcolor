use crate::{Color, Region, Shader, Texture};
use std::sync::Arc;

// Resource Definitions
#[derive(Debug)]
pub enum PassInput {
    None,
    Clear(Color),
    Texture(Arc<Texture>),
    Pass(Arc<Pass>),
}

#[derive(Debug)]
pub enum PassType {
    Compute,
    Render,
}

#[derive(Debug)]
pub struct Pass {
    pub(crate) name: Arc<str>,
    pub(crate) input: PassInput,
    pub(crate) shaders: Vec<Arc<Shader>>,
    pub(crate) region: Option<Region>,
    pub pass_type: PassType,
}

impl Pass {
    pub fn new(name: &str) -> Self {
        Self {
            name: Arc::from(name),
            shaders: Vec::new(),
            region: None,
            input: PassInput::None,
            pass_type: PassType::Render,
        }
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.input = PassInput::Clear(color);
    }

    pub fn get_input(&self) -> &PassInput {
        &self.input
    }

    pub fn add_shader(&mut self, shader: Arc<Shader>) {
        if shader.is_compute() && self.shaders.len() == 0 {
            self.pass_type = PassType::Compute;
            self.shaders.push(shader);
        } else {
            self.shaders.push(shader);
        }
    }

    pub fn set_region(&mut self, region: Region) {
        self.region = Some(region);
    }

    pub fn execute(&self, _encoder: &mut wgpu::CommandEncoder) {
        // @TODO Execute draw calls
    }

    pub fn is_compute(&self) -> bool {
        matches!(self.pass_type, PassType::Compute)
    }
}
