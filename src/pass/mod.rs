use crate::{Color, Region, Renderable, Shader, ShaderObject};
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod features;

#[cfg_attr(feature = "python", pyclass)]
// Resource Definitions
#[derive(Debug, Clone)]
pub enum PassInput {
    Load(),
    Clear(Color),
}

#[derive(Debug)]
pub enum PassType {
    Compute,
    Render,
}

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug)]
pub struct Pass {
    pub(crate) object: Arc<PassObject>,
}

impl Pass {
    pub fn new(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Render)),
        }
    }

    pub fn compute(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Compute)),
        }
    }

    pub fn from_shader(name: &str, shader: &Shader) -> Self {
        Self {
            object: Arc::new(PassObject::from_shader_object(name, shader.object.clone())),
        }
    }

    pub fn load_previous(&self) {
        *self.object.input.write() = PassInput::Load();
    }

    pub fn get_input(&self) -> PassInput {
        self.object.get_input()
    }

    pub fn add_shader(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    pub fn set_region(&self, region: Region) {
        self.object.set_region(region);
    }

    pub fn set_clear_color(&self, color: [f32; 4]) {
        self.object.set_clear_color(color);
    }
}

impl Renderable for Pass {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        vec![self.object.as_ref()]
    }
}

#[derive(Debug)]
pub struct PassObject {
    pub(crate) name: Arc<str>,
    pub(crate) input: RwLock<PassInput>,
    pub(crate) shaders: RwLock<Vec<Arc<ShaderObject>>>,
    pub(crate) region: RwLock<Option<Region>>,
    pub(crate) required_buffer_size: RwLock<u64>,
    pub pass_type: PassType,
}

impl PassObject {
    pub fn new(name: &str, pass_type: PassType) -> Self {
        Self {
            name: Arc::from(name),
            shaders: RwLock::new(Vec::new()),
            region: RwLock::new(None),
            input: RwLock::new(PassInput::Load()),
            required_buffer_size: RwLock::new(0),
            pass_type,
        }
    }

    pub(crate) fn from_shader_object(name: &str, shader: Arc<ShaderObject>) -> Self {
        let pass_type = if shader.is_compute() {
            PassType::Compute
        } else {
            PassType::Render
        };

        let total_bytes = shader.total_bytes;

        Self {
            name: Arc::from(name),
            shaders: RwLock::new(vec![shader]),
            region: RwLock::new(None),
            input: RwLock::new(PassInput::Load()),
            required_buffer_size: RwLock::new(total_bytes),
            pass_type,
        }
    }

    pub fn set_clear_color(&self, color: impl Into<Color>) {
        *self.input.write() = PassInput::Clear(color.into());
    }

    pub fn get_input(&self) -> PassInput {
        self.input.read().clone()
    }

    pub fn add_shader(&self, shader: &Shader) {
        if shader.object.is_compute() == self.is_compute() {
            *self.required_buffer_size.write() += shader.object.total_bytes;
            self.shaders.write().push(shader.object.clone());
        } else {
            log::warn!("Cannot add a compute shader to a render pass or vice versa");
        }
    }

    pub fn set_region(&self, region: Region) {
        *self.region.write() = Some(region);
    }

    pub fn is_compute(&self) -> bool {
        matches!(self.pass_type, PassType::Compute)
    }
}
