use crate::{Color, Region, Renderable, Shader, ShaderObject};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod platform;

pub mod error;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
// Resource Definitions
#[derive(Debug, Clone)]
pub struct PassInput {
    pub(crate) load: bool,
    pub(crate) color: Color,
}

impl PassInput {
    pub fn load() -> Self {
        Self {
            load: true,
            color: Color::transparent(),
        }
    }

    pub fn clear(color: Color) -> Self {
        Self { load: false, color }
    }
}

#[derive(Debug)]
pub enum PassType {
    Compute,
    Render,
}

#[derive(Debug, Clone)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(wasm, wasm_bindgen)]
#[lsp_doc("docs/api/core/pass/pass.md")]
pub struct Pass {
    pub(crate) object: Arc<PassObject>,
}

impl Pass {
    #[lsp_doc("docs/api/core/pass/new.md")]
    pub fn new(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Render)),
        }
    }

    #[lsp_doc("docs/api/core/pass/compute.md")]
    pub fn compute(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Compute)),
        }
    }

    #[lsp_doc("docs/api/core/pass/from_shader.md")]
    pub fn from_shader(name: &str, shader: &Shader) -> Self {
        Self {
            object: Arc::new(PassObject::from_shader_object(name, shader.object.clone())),
        }
    }

    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous(&self) {
        *self.object.input.write() = PassInput {
            load: true,
            color: Color::transparent(),
        }
    }

    #[lsp_doc("docs/api/core/pass/get_input.md")]
    pub fn get_input(&self) -> PassInput {
        self.object.get_input()
    }

    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh(&self, mesh: &crate::mesh::Mesh) -> Result<(), crate::shader::ShaderError> {
        self.ensure_shader();

        // Attach this mesh to the last shader in the pass.
        if let Some(shader) = self.object.shaders.read().last().cloned() {
            // Recreate a lightweight Shader wrapper to reuse validation
            // (public API requires going through Shader::add_mesh)
            let s = crate::Shader {
                pass: self.object.clone(),
                object: shader.clone(),
            };
            s.add_mesh(mesh)
        } else {
            Ok(())
        }
    }

    #[lsp_doc("docs/api/core/pass/add_mesh_to_shader.md")]
    pub fn add_mesh_to_shader(
        &self,
        mesh: &crate::mesh::Mesh,
        shader: &crate::Shader,
    ) -> Result<(), crate::shader::ShaderError> {
        shader.add_mesh(mesh)
    }

    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport(&self, viewport: Region) {
        self.object.set_viewport(viewport);
    }

    #[lsp_doc("docs/api/core/pass/set_clear_color.md")]
    pub fn set_clear_color(&self, color: [f32; 4]) {
        self.object.set_clear_color(color);
    }

    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch(&self, x: u32, y: u32, z: u32) {
        self.object.set_compute_dispatch(x, y, z);
    }

    /// Ensure at least one Shader exists in this Pass; if none, add a default shader.
    fn ensure_shader(&self) {
        let shaders = self.object.shaders.read();
        if shaders.is_empty() {
            // @TODO create a Shader from a specific Mesh (mimics Vertex layout)
            self.add_shader(&Shader::default());
        }
    }
}

impl Renderable for Pass {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        vec![self.object.as_ref()]
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Pass, crate::pass::error::PassError);

#[derive(Debug)]
pub struct PassObject {
    pub(crate) name: Arc<str>,
    pub(crate) input: RwLock<PassInput>,
    pub(crate) shaders: RwLock<Vec<Arc<ShaderObject>>>,
    pub(crate) viewport: RwLock<Option<Region>>,
    pub(crate) required_buffer_size: RwLock<u64>,
    pub(crate) mesh: RwLock<Option<Arc<crate::mesh::MeshObject>>>,
    pub pass_type: PassType,
    // For compute passes: dispatch size (defaults to 1,1,1)
    pub(crate) compute_dispatch: RwLock<(u32, u32, u32)>,
}

impl PassObject {
    pub fn new(name: &str, pass_type: PassType) -> Self {
        Self {
            name: Arc::from(name),
            shaders: RwLock::new(Vec::new()),
            viewport: RwLock::new(None),
            input: RwLock::new(PassInput::clear(Color::transparent())),
            required_buffer_size: RwLock::new(0),
            mesh: RwLock::new(None),
            pass_type,
            compute_dispatch: RwLock::new((1, 1, 1)),
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
            viewport: RwLock::new(None),
            input: RwLock::new(PassInput::clear(Color::transparent())),
            required_buffer_size: RwLock::new(total_bytes),
            mesh: RwLock::new(None),
            pass_type,
            compute_dispatch: RwLock::new((1, 1, 1)),
        }
    }

    pub fn set_clear_color(&self, color: impl Into<Color>) {
        *self.input.write() = PassInput::clear(color.into());
    }

    /// Set compute dispatch size for compute passes.
    pub fn set_compute_dispatch(&self, x: u32, y: u32, z: u32) {
        *self.compute_dispatch.write() = (x.max(1), y.max(1), z.max(1));
    }

    pub fn compute_dispatch(&self) -> (u32, u32, u32) {
        *self.compute_dispatch.read()
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

    pub fn set_viewport(&self, viewport: Region) {
        *self.viewport.write() = Some(viewport);
    }

    pub fn is_compute(&self) -> bool {
        matches!(self.pass_type, PassType::Compute)
    }
}

impl AsRef<PassObject> for Pass {
    fn as_ref(&self) -> &PassObject {
        &self.object
    }
}

impl AsRef<PassObject> for PassObject {
    fn as_ref(&self) -> &PassObject {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Create a render pass, add two render shaders, and observe required buffer size grows.
    #[test]
    fn adds_render_shaders_and_updates_required_bytes() {
        // Arrange
        let s1 = Shader::default();
        let s2 = Shader::default();
        let pass = Pass::new("p");

        // Act
        let before = *pass.object.required_buffer_size.read();
        pass.add_shader(&s1);
        pass.add_shader(&s2);
        let after = *pass.object.required_buffer_size.read();

        // Assert
        assert!(after >= before);
        assert_eq!(pass.object.shaders.read().len(), 2);
        assert!(!pass.object.is_compute());
    }

    // Story: A compute shader cannot be added to a render pass (and vice versa); count does not change.
    #[test]
    fn rejects_mismatched_shader_kinds() {
        // Arrange: a render pass with one render shader
        let render_shader = Shader::default();
        let compute_src = r#"
            @group(0) @binding(0) var<uniform> u: vec4<f32>;
            @compute @workgroup_size(1)
            fn cs() { _ = u; }
        "#;
        let compute_shader = Shader::new(compute_src).expect("compute shader");
        let pass = Pass::from_shader("render pass", &render_shader);
        let count_before = pass.object.shaders.read().len();
        let bytes_before = *pass.object.required_buffer_size.read();

        // Act: try to add a compute shader to a render pass
        pass.add_shader(&compute_shader);

        // Assert: no change
        assert_eq!(pass.object.shaders.read().len(), count_before);
        assert_eq!(*pass.object.required_buffer_size.read(), bytes_before);
        assert!(!pass.object.is_compute());
    }

    // Story: set_clear_color changes PassInput to a clear op; load_previous flips back to load.
    #[test]
    fn toggles_input_between_clear_and_load() {
        // Arrange
        let pass = Pass::new("p");

        // Act: set a clear color
        pass.set_clear_color([0.1, 0.2, 0.3, 0.4]);
        let after_clear = pass.get_input();

        // Act: switch to load previous contents
        pass.load_previous();
        let after_load = pass.get_input();

        // Assert
        assert!(!after_clear.load);
        assert_eq!(after_clear.color, Color::from([0.1, 0.2, 0.3, 0.4]));
        assert!(after_load.load);
    }

    // Story: setting a viewport is stored and can be read back.
    #[test]
    fn sets_viewport_rect() {
        // Arrange
        let pass = Pass::new("p");
        let vp = Region::from_region(2, 4, 8, 6);

        // Act
        pass.set_viewport(vp);

        // Assert
        assert_eq!(pass.object.viewport.read().as_ref(), Some(vp).as_ref());
    }
}
