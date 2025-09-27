use crate::{Color, Mesh, Region, Renderable, Shader, ShaderObject};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod platform;

pub mod error;
pub use error::*;

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

impl From<Shader> for PassType {
    fn from(shader: Shader) -> Self {
        shader.object.into()
    }
}

impl From<Arc<ShaderObject>> for PassType {
    fn from(shader: Arc<ShaderObject>) -> Self {
        match shader.is_compute() {
            true => PassType::Compute,
            false => PassType::Render,
        }
    }
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
    pub fn add_mesh(&self, mesh: &Mesh) -> Result<(), PassError> {
        self.object.add_mesh(mesh)
    }

    #[lsp_doc("docs/api/core/pass/add_mesh_to_shader.md")]
    pub fn add_mesh_to_shader(&self, mesh: &Mesh, shader: &Shader) -> Result<(), PassError> {
        Ok(shader.add_mesh(mesh)?)
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

    #[lsp_doc("docs/api/core/pass/add_target.md")]
    pub fn add_target<T>(&self, target: T) -> Result<(), PassError>
    where
        T: TryInto<ColorTarget, Error = PassError>,
    {
        let ct = target.try_into()?;
        self.object.set_color_target_id(ct.id);
        Ok(())
    }

    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute(&self) -> bool {
        self.object.is_compute()
    }
}

impl Renderable for Pass {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        vec![self.object.as_ref()]
    }
}

/// A reference to a color render target texture. Expects a stable texture created by the Renderer
/// (same device/context) with usage including RENDER_ATTACHMENT and a color-capable format.
#[derive(Clone, Debug)]
pub struct ColorTarget {
    pub(crate) id: crate::texture::TextureId,
}

impl TryFrom<&crate::texture::Texture> for ColorTarget {
    type Error = PassError;
    fn try_from(tex: &crate::texture::Texture) -> Result<Self, Self::Error> {
        // Validate usage and format for color attachment
        let usage = tex.object.usage;
        if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            return Err(PassError::InvalidColorTarget(
                "texture was not created with RENDER_ATTACHMENT usage".into(),
            ));
        }
        let fmt = tex.object.format;
        match fmt {
            wgpu::TextureFormat::Depth32Float
            | wgpu::TextureFormat::Depth16Unorm
            | wgpu::TextureFormat::Depth24Plus
            | wgpu::TextureFormat::Depth24PlusStencil8
            | wgpu::TextureFormat::Depth32FloatStencil8 => {
                return Err(PassError::InvalidColorTarget(
                    "depth/stencil formats are not valid color targets".into(),
                ));
            }
            _ => {}
        }
        Ok(ColorTarget {
            id: tex.id().clone(),
        })
    }
}

impl TryFrom<&crate::target::TextureTarget> for ColorTarget {
    type Error = PassError;
    fn try_from(target: &crate::target::TextureTarget) -> Result<Self, Self::Error> {
        // Convert to a Texture handle and reuse validation
        let texture = target.texture();
        ColorTarget::try_from(&texture)
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Pass, crate::pass::error::PassError);

#[derive(Debug)]
pub struct PassObject {
    pub pass_type: PassType,
    pub(crate) name: Arc<str>,
    pub(crate) input: RwLock<PassInput>,
    pub(crate) shaders: RwLock<Vec<Arc<ShaderObject>>>,
    pub(crate) viewport: RwLock<Option<Region>>,
    pub(crate) required_buffer_size: RwLock<u64>,
    // For compute passes: dispatch size (defaults to 1,1,1)
    pub(crate) compute_dispatch: RwLock<(u32, u32, u32)>,
    // Milestone A: optional per-pass color target
    pub(crate) color_target: RwLock<Option<crate::texture::TextureId>>,
    // Milestone B (placeholders): depth attachment/state and storage alias map
    pub(crate) _depth_target: RwLock<Option<crate::texture::TextureId>>,
    pub(crate) _storage_alias: RwLock<HashMap<String, String>>,
    pub(crate) present_to_target: RwLock<bool>,
}

impl PassObject {
    /// Create a new Pass object with the given name and type.
    pub fn new(name: &str, pass_type: PassType) -> Self {
        Self {
            name: Arc::from(name),
            shaders: RwLock::new(Vec::new()),
            viewport: RwLock::new(None),
            input: RwLock::new(PassInput::clear(Color::transparent())),
            required_buffer_size: RwLock::new(0),
            pass_type,
            compute_dispatch: RwLock::new((1, 1, 1)),
            color_target: RwLock::new(None),
            _depth_target: RwLock::new(None),
            _storage_alias: RwLock::new(HashMap::new()),
            present_to_target: RwLock::new(false),
        }
    }

    /// Set the clear color for this pass; changes input to a clear operation.
    pub fn set_clear_color(&self, color: impl Into<Color>) {
        *self.input.write() = PassInput::clear(color.into());
    }

    /// Set compute dispatch size for compute passes.
    pub fn set_compute_dispatch(&self, x: u32, y: u32, z: u32) {
        *self.compute_dispatch.write() = (x.max(1), y.max(1), z.max(1));
    }

    /// Get the compute dispatch size for compute passes.
    pub fn compute_dispatch(&self) -> (u32, u32, u32) {
        *self.compute_dispatch.read()
    }

    /// Get the current PassInput (load or clear).
    pub fn get_input(&self) -> PassInput {
        self.input.read().clone()
    }

    /// Add a shader to this pass. The shader must be compatible with the pass type.
    pub fn add_shader(&self, shader: &Shader) {
        self.add_shader_object(shader.object.clone())
    }

    pub fn set_color_target_id(&self, id: crate::texture::TextureId) {
        *self.color_target.write() = Some(id);
        // Selecting a color target marks this pass as intermediate by default
        *self.present_to_target.write() = false;
    }

    /// Internal method to add a shader object to this pass.
    fn add_shader_object(&self, shader: Arc<ShaderObject>) {
        if shader.is_compute() == self.is_compute() {
            *self.required_buffer_size.write() += shader.total_bytes;
            self.shaders.write().push(shader.clone());
        } else {
            log::warn!("Cannot add a compute shader to a render pass or vice versa");
        }
    }

    /// Add a mesh to the last compatible shader in this pass.
    /// If no compatible shader is found, an error is returned.
    pub fn add_mesh(&self, mesh: &Mesh) -> Result<(), PassError> {
        let shaders = self.shaders.read();

        // loop backwards and find a compatible shader
        for shader in shaders.iter().rev() {
            match shader.add_mesh(mesh.object.clone()) {
                Ok(mesh_added) => return Ok(mesh_added),
                Err(_) => continue,
            }
        }

        Err(PassError::NoCompatibleShader)
    }

    pub fn set_viewport(&self, viewport: Region) {
        *self.viewport.write() = Some(viewport);
    }

    pub fn is_compute(&self) -> bool {
        matches!(self.pass_type, PassType::Compute)
    }

    pub(crate) fn from_shader_object(name: &str, shader: Arc<ShaderObject>) -> Self {
        let pass_type = match shader.is_compute() {
            true => PassType::Compute,
            false => PassType::Render,
        };
        let object = Self::new(name, pass_type);
        object.add_shader_object(shader);
        object
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

impl From<Shader> for Pass {
    fn from(shader: Shader) -> Self {
        Self::from_shader("Default Pass from Shader", &shader)
    }
}

impl From<Arc<ShaderObject>> for PassObject {
    fn from(shader: Arc<ShaderObject>) -> Self {
        Self::from_shader_object("Default Pass from ShaderObject", shader)
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
