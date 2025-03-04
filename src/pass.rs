use std::sync::Arc;

use crate::{Color, Compute, Region, Shader, Target, Texture};

#[derive(Debug)]
/// A Pass can be a Render Pass or a Compute Pass.
pub enum Pass {
    Render(RenderPass),
    Compute(ComputePass),
}

// Resource Definitions
#[derive(Debug)]
pub enum PassInput {
    Clear(Color),
    Pass,
    Texture(Arc<Texture>),
}

#[derive(Debug)]
pub struct RenderPassConfig {
    pub shaders: Vec<Arc<Shader>>,
    pub targets: Vec<Arc<Target>>,
    pub region: Option<Region>,
}

#[derive(Debug)]
pub struct RenderPass {
    pub name: String,
    pub(crate) shaders: Vec<Arc<Shader>>,
    pub(crate) input: PassInput,
    pub(crate) targets: Vec<Arc<Target>>,
    pub(crate) region: Option<Region>,
}

impl RenderPass {
    pub fn new(name: &str, input: impl Into<PassInput>) -> Self {
        Self {
            name: name.to_string(),
            shaders: Vec::new(),
            targets: Vec::new(),
            region: None,
            input: input.into(),
        }
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.input = PassInput::Clear(color);
    }

    pub fn get_input(&self) -> &PassInput {
        &self.input
    }

    pub fn add_shader(&mut self, shader: Arc<Shader>) {
        self.shaders.push(shader);
    }

    pub fn add_target(&mut self, target: Arc<Target>) {
        self.targets.push(target);
    }

    pub fn set_region(&mut self, region: Region) {
        self.region = Some(region);
    }

    pub fn execute(&self, _encoder: &mut wgpu::CommandEncoder) {
        // @TODO Execute draw calls
    }
}

#[derive(Default, Debug)]
pub struct ComputePass {
    _computes: Vec<Arc<Compute>>,
    // input: ResourceId,
    // output: Target, // @TODO
}

impl ComputePass {
    pub fn new() -> Self {
        unimplemented!("Compute Pass is not implemented yet")
        // Self {
        //     computes: Vec::new(),
        //     // dependencies: Vec::new(),
        //     // input: ResourceId::default(),
        //     // output: Target::default(),
        // }
    }

    pub fn add_compute(&mut self, _compute: Compute) {
        unimplemented!("Compute Pass is not implemented yet")
        // self.computes.push(compute);
    }

    // pub fn add_dependency(&mut self, id: ResourceId) {
    //     self.dependencies.push(id);
    // }
}
