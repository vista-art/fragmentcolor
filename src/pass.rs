use crate::{Color, Region, Renderable, Shader, ShaderObject};
use std::cell::RefCell;
use std::sync::Arc;

// Resource Definitions
#[derive(Debug, Clone)]
pub enum PassInput {
    Load,
    Clear(Color),
}

#[derive(Debug)]
pub enum PassType {
    Compute,
    Render,
}

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

    pub fn set_clear_color(&self, color: impl Into<Color>) {
        self.object.set_clear_color(color.into());
    }

    pub fn load_previous(&self) {
        *self.object.input.borrow_mut() = PassInput::Load;
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
}

impl Renderable for Pass {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        vec![self.object.as_ref()]
    }
}

#[derive(Debug)]
pub struct PassObject {
    pub(crate) name: Arc<str>,
    pub(crate) input: RefCell<PassInput>,
    pub(crate) shaders: RefCell<Vec<Arc<ShaderObject>>>,
    pub(crate) region: RefCell<Option<Region>>,
    pub(crate) required_buffer_size: RefCell<u64>,
    pub pass_type: PassType,
}

impl PassObject {
    pub fn new(name: &str, pass_type: PassType) -> Self {
        Self {
            name: Arc::from(name),
            shaders: RefCell::new(Vec::new()),
            region: RefCell::new(None),
            input: RefCell::new(PassInput::Load),
            required_buffer_size: RefCell::new(0),
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
            shaders: RefCell::new(vec![shader]),
            region: RefCell::new(None),
            input: RefCell::new(PassInput::Load),
            required_buffer_size: RefCell::new(total_bytes),
            pass_type,
        }
    }

    pub fn set_clear_color(&self, color: impl Into<Color>) {
        *self.input.borrow_mut() = PassInput::Clear(color.into());
    }

    pub fn get_input(&self) -> PassInput {
        self.input.borrow().clone()
    }

    pub fn add_shader(&self, shader: &Shader) {
        if shader.object.is_compute() == self.is_compute() {
            *self.required_buffer_size.borrow_mut() += shader.object.total_bytes;
            self.shaders.borrow_mut().push(shader.object.clone());
        } else {
            log::warn!("Cannot add a compute shader to a render pass or vice versa");
        }
    }

    pub fn set_region(&self, region: Region) {
        *self.region.borrow_mut() = Some(region);
    }

    pub fn is_compute(&self) -> bool {
        matches!(self.pass_type, PassType::Compute)
    }
}
