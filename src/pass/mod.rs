use crate::{Color, Mesh, Region, Renderable, Shader, ShaderObject};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

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

crate::impl_fc_kind!(Pass, "Pass");

impl Pass {
    #[lsp_doc("docs/api/core/pass/new.md")]
    pub fn new(name: &str) -> Self {
        let obj = Arc::new(PassObject::new(name, PassType::Render));
        PassObject::ensure_flat_current(&obj);
        Self { object: obj }
    }

    #[lsp_doc("docs/api/core/pass/compute.md")]
    pub fn compute(name: &str) -> Self {
        let obj = Arc::new(PassObject::new(name, PassType::Compute));
        PassObject::ensure_flat_current(&obj);
        Self { object: obj }
    }

    #[lsp_doc("docs/api/core/pass/from_shader.md")]
    pub fn from_shader(name: &str, shader: &Shader) -> Self {
        let obj = Arc::new(PassObject::from_shader_object(name, shader.object.clone()));
        PassObject::ensure_flat_current(&obj);
        Self { object: obj }
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
    pub fn set_viewport(&self, viewport: impl Into<Region>) {
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

    #[lsp_doc("docs/api/core/pass/add_depth_target.md")]
    pub fn add_depth_target<T>(&self, target: T) -> Result<(), PassError>
    where
        T: TryInto<DepthTarget, Error = PassError>,
    {
        let dt = target.try_into()?;
        self.object.set_depth_target_id(dt.id);
        Ok(())
    }

    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute(&self) -> bool {
        self.object.is_compute()
    }

    #[lsp_doc("docs/api/core/pass/require.md")]
    pub fn require<R: Renderable>(&self, dependencies: &R) -> Result<(), PassError> {
        let roots = dependencies.roots();
        PassObject::add_dependencies_arc(&self.object, roots.iter().cloned().collect::<Vec<_>>())
    }
}

impl Renderable for Pass {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        PassObject::ensure_flat_current(&self.object);
        self.object.flat.read().clone()
    }
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.object.clone()].into()
    }
}

// Sequential lists: do not expand dependencies; return the listed passes in order.
impl Renderable for &[Pass] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for Vec<Pass> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for &[&Pass] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for Vec<&Pass> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

// Provide convenience for direct Arc containers if needed.
impl Renderable for &[Arc<PassObject>] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.to_vec();
        v.into()
    }
}

impl Renderable for Vec<Arc<PassObject>> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.clone().into()
    }
}

/// A reference to a color render target texture. Expects a stable texture created by the Renderer
/// (same device/context) with usage including RENDER_ATTACHMENT and a color-capable format.
#[derive(Clone, Debug)]
pub struct ColorTarget {
    pub(crate) id: crate::texture::TextureId,
}

/// A reference to a depth render target texture (Depth32Float or similar).
/// Must be a stable texture created by the Renderer (same device/context) with RENDER_ATTACHMENT usage and a depth/stencil format.
#[derive(Clone, Debug)]
pub struct DepthTarget {
    pub(crate) id: crate::texture::TextureId,
}

impl TryFrom<&crate::texture::Texture> for ColorTarget {
    type Error = PassError;

    /// Validate usage and format for color attachment
    fn try_from(tex: &crate::texture::Texture) -> Result<Self, Self::Error> {
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
        Ok(ColorTarget { id: *tex.id() })
    }
}

impl TryFrom<&crate::target::TextureTarget> for ColorTarget {
    type Error = PassError;

    // Convert to a Texture handle and reuse validation
    fn try_from(target: &crate::target::TextureTarget) -> Result<Self, Self::Error> {
        let texture = target.texture();
        ColorTarget::try_from(&texture)
    }
}

impl TryFrom<&crate::target::TextureTarget> for DepthTarget {
    type Error = PassError;

    // Convert to a Texture handle and reuse validation for depth targets
    fn try_from(target: &crate::target::TextureTarget) -> Result<Self, Self::Error> {
        let texture = target.texture();
        DepthTarget::try_from(&texture)
    }
}

impl TryFrom<&crate::texture::Texture> for DepthTarget {
    type Error = PassError;

    /// Validate usage and format for depth/stencil attachment
    fn try_from(tex: &crate::texture::Texture) -> Result<Self, Self::Error> {
        let usage = tex.object.usage;
        if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            return Err(PassError::InvalidColorTarget(
                "depth texture was not created with RENDER_ATTACHMENT usage".into(),
            ));
        }
        let format = tex.object.format;
        match format {
            wgpu::TextureFormat::Depth32Float
            | wgpu::TextureFormat::Depth16Unorm
            | wgpu::TextureFormat::Depth24Plus
            | wgpu::TextureFormat::Depth24PlusStencil8
            | wgpu::TextureFormat::Depth32FloatStencil8 => Ok(DepthTarget { id: *tex.id() }),
            _ => Err(PassError::InvalidColorTarget(
                "texture format is not a depth/stencil format".into(),
            )),
        }
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Pass, PassError);

static GRAPH_VERSION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct PassObject {
    pub pass_type: RwLock<PassType>,
    pub(crate) name: Arc<str>,
    pub(crate) input: RwLock<PassInput>,
    pub(crate) shaders: RwLock<Vec<Arc<ShaderObject>>>,
    pub(crate) viewport: RwLock<Option<Region>>,
    pub(crate) required_buffer_size: RwLock<u64>,
    // For compute passes: dispatch size (defaults to 1,1,1)
    pub(crate) compute_dispatch: RwLock<(u32, u32, u32)>,
    // Milestone A: optional per-pass color target
    pub(crate) color_target: RwLock<Option<crate::texture::TextureId>>,
    // Optional per-pass depth attachment (Depth32Float for now)
    pub(crate) depth_target: RwLock<Option<crate::texture::TextureId>>,
    // Milestone B (placeholders): storage alias map
    pub(crate) _storage_alias: RwLock<HashMap<String, String>>,
    pub(crate) present_to_target: RwLock<bool>,
    // DAG metadata and cached traversal
    pub(crate) dependencies: RwLock<Vec<Arc<PassObject>>>,
    pub(crate) dependency_names: RwLock<std::collections::HashSet<Arc<str>>>,
    pub(crate) flat: RwLock<Arc<[Arc<PassObject>]>>,
    pub(crate) built_version: AtomicU64,
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
            pass_type: RwLock::new(pass_type),
            compute_dispatch: RwLock::new((1, 1, 1)),
            color_target: RwLock::new(None),
            depth_target: RwLock::new(None),
            _storage_alias: RwLock::new(HashMap::new()),
            present_to_target: RwLock::new(false),
            dependencies: RwLock::new(Vec::new()),
            dependency_names: RwLock::new(std::collections::HashSet::new()),
            flat: RwLock::new(Arc::from(Vec::<Arc<PassObject>>::new().into_boxed_slice())),
            built_version: AtomicU64::new(0),
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

    /// Set per-pass depth attachment by TextureId.
    pub fn set_depth_target_id(&self, id: crate::texture::TextureId) {
        *self.depth_target.write() = Some(id);
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

    pub fn set_viewport(&self, viewport: impl Into<Region>) {
        *self.viewport.write() = Some(viewport.into());
    }

    pub fn is_compute(&self) -> bool {
        matches!(*self.pass_type.read(), PassType::Compute)
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

    /// Internal method to add a shader object to this pass.
    fn add_shader_object(&self, shader: Arc<ShaderObject>) {
        // If this is the first shader added, adopt the shader kind for the pass.
        {
            let shaders = self.shaders.read();
            if shaders.is_empty() {
                let pass_type = match shader.is_compute() {
                    true => PassType::Compute,
                    false => PassType::Render,
                };
                *self.pass_type.write() = pass_type;
            }
        }

        if shader.is_compute() == self.is_compute() {
            *self.required_buffer_size.write() += shader.total_bytes;
            self.shaders.write().push(shader.clone());
        } else {
            log::warn!("Cannot add a compute shader to a render pass or vice versa");
        }
    }

    // Link a list of dependencies into this node, maintain sibling order, rebuild caches, and propagate.
    pub(crate) fn add_dependencies_arc(
        self_arc: &Arc<PassObject>,
        deps: impl IntoIterator<Item = Arc<PassObject>>,
    ) -> Result<(), PassError> {
        for dep in deps {
            Self::link_one(self_arc, dep.clone())?;
        }
        // Maintain sibling order among direct dependencies
        Self::resort_dependencies(self_arc);
        // Bump global graph version (lazy finalize will observe this)
        GRAPH_VERSION.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn link_one(self_arc: &Arc<PassObject>, dep: Arc<PassObject>) -> Result<(), PassError> {
        let this = &**self_arc;
        // Self-dependency by name
        if dep.name.as_ref() == this.name.as_ref() {
            return Err(PassError::SelfDependency);
        }
        // Duplicate by name
        {
            let mut names = this.dependency_names.write();
            if names.contains(&dep.name) {
                return Err(PassError::DuplicateDependency(dep.name.to_string()));
            }
            // Cycle: if dep reaches self, linking would close a cycle
            if dep.reaches(this.name.as_ref()) {
                return Err(PassError::DependencyCycle {
                    via: dep.name.to_string(),
                });
            }
            names.insert(dep.name.clone());
        }
        // Link
        this.dependencies.write().push(dep.clone());
        Ok(())
    }

    // Does this node reach target_name through dependencies edges? (BFS)
    fn reaches(&self, target_name: &str) -> bool {
        use std::collections::{HashSet, VecDeque};
        let mut seen: HashSet<*const PassObject> = HashSet::new();
        let mut q: VecDeque<*const PassObject> = VecDeque::new();
        q.push_back(self as *const _);
        while let Some(ptr) = q.pop_front() {
            if !seen.insert(ptr) {
                continue;
            }
            // Safety: all pointers originate from &self or Arcs we hold; read-only
            let node = unsafe { &*ptr };
            if node.name.as_ref() == target_name {
                return true;
            }
            // Clone arcs locally to avoid holding the lock across iteration
            let deps: Vec<Arc<PassObject>> = { node.dependencies.read().clone() };
            for d in deps.into_iter() {
                q.push_back(&*d as *const _);
            }
        }
        false
    }

    // Ensure cached deps-first flat slice is up-to-date with global graph version
    pub(crate) fn ensure_flat_current(self_arc: &Arc<PassObject>) {
        let global = GRAPH_VERSION.load(Ordering::Relaxed);
        let built = self_arc.built_version.load(Ordering::Relaxed);
        if built == global {
            return;
        }
        use std::collections::HashSet;
        let mut out: Vec<Arc<PassObject>> = Vec::new();
        let mut seen: HashSet<*const PassObject> = HashSet::new();
        fn dfs(
            n_arc: &Arc<PassObject>,
            seen: &mut std::collections::HashSet<*const PassObject>,
            out: &mut Vec<Arc<PassObject>>,
        ) {
            let k = &**n_arc as *const _;
            if !seen.insert(k) {
                return;
            }
            let deps = n_arc.dependencies.read().clone();
            for d in deps.iter() {
                dfs(d, seen, out);
            }
            out.push(Arc::clone(n_arc));
        }
        dfs(self_arc, &mut seen, &mut out);
        let slice: Arc<[Arc<PassObject>]> = out.into_boxed_slice().into();
        *self_arc.flat.write() = slice;
        self_arc.built_version.store(global, Ordering::Relaxed);
    }
    // Stable topological sort among direct dependencies using reachability.
    fn resort_dependencies(self_arc: &Arc<PassObject>) {
        let deps = self_arc.dependencies.read().clone();
        let n = deps.len();
        if n <= 1 {
            return;
        }
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut indeg: Vec<usize> = vec![0; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let a = &deps[i];
                let b = &deps[j];
                if a.reaches(b.name.as_ref()) {
                    adj[i].push(j);
                    indeg[j] += 1;
                } else if b.reaches(a.name.as_ref()) {
                    adj[j].push(i);
                    indeg[i] += 1;
                }
            }
        }
        // Kahn with stable seed order
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for (i, _) in indeg.iter().enumerate().take(n) {
            if indeg[i] == 0 {
                queue.push_back(i);
            }
        }
        let mut out_idx = Vec::with_capacity(n);
        while let Some(i) = queue.pop_front() {
            out_idx.push(i);
            for &j in &adj[i] {
                indeg[j] -= 1;
                if indeg[j] == 0 {
                    queue.push_back(j);
                }
            }
        }
        if out_idx.len() == n {
            let mut new_deps = Vec::with_capacity(n);
            for i in out_idx {
                new_deps.push(deps[i].clone());
            }
            *self_arc.dependencies.write() = new_deps;
        } else {
            // Fallback: keep insertion order if something went wrong
        }
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

    // Story: compute dispatch clamps zeros to 1 per dimension.
    #[test]
    fn compute_dispatch_clamps_zeros() {
        let p = Pass::compute("c");
        p.set_compute_dispatch(0, 2, 0);
        assert_eq!(*p.object.compute_dispatch.read(), (1, 2, 1));
    }

    // Story: adding a mesh explicitly to a shader via pass helper succeeds when compatible.
    #[test]
    fn add_mesh_to_shader_happy_path() {
        // Shader that expects @location(0) position (vec2)
        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(position, 0.0, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
        "#;
        let shader = Shader::new(wgsl).expect("shader with pos input");
        let pass = Pass::from_shader("p", &shader);
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5f32, -0.5f32]),
            Vertex::new([0.5f32, -0.5f32]),
            Vertex::new([0.0f32, 0.5f32]),
        ]);
        let res = pass.add_mesh_to_shader(&mesh, &shader);
        assert!(res.is_ok(), "mesh should be compatible: {:?}", res);
    }

    // Story: ColorTarget/DepthTarget validations pass/fail as per format and usage.
    #[test]
    fn color_and_depth_target_tryfrom_validate() {
        pollster::block_on(async move {
            let r = crate::Renderer::new();
            // A color-capable texture target
            let tex_target = r
                .create_texture_target([4u32, 4u32])
                .await
                .expect("texture target");
            // ColorTarget from TextureTarget should succeed
            let ct = ColorTarget::try_from(&tex_target);
            assert!(ct.is_ok());

            // Depth texture should not be valid as a ColorTarget
            let depth = r.create_depth_texture([4u32, 4u32]).await.expect("depth");
            let ct_bad = ColorTarget::try_from(&depth);
            assert!(ct_bad.is_err());

            // DepthTarget from depth texture is Ok
            let dt = DepthTarget::try_from(&depth);
            assert!(dt.is_ok());

            // Non-depth texture (storage) should not be valid as DepthTarget
            let color_tex = r
                .create_storage_texture([4u32, 4u32], crate::TextureFormat::default(), None)
                .await
                .expect("color tex");
            let dt_bad = DepthTarget::try_from(&color_tex);
            assert!(dt_bad.is_err());
        });
    }

    // Story: dependency linking rejects self, duplicate and cycles.
    #[test]
    fn dependencies_self_duplicate_cycle_errors() {
        // self-dependency
        let a = Pass::new("A");
        let e1 = a.require(&a).unwrap_err();
        let s1 = format!("{}", e1);
        assert!(s1.contains("Self"), "expected self-dependency error: {s1}");

        // duplicate (fresh pair)
        let x = Pass::new("X");
        let y = Pass::new("Y");
        x.require(&y).expect("first ok");
        let e2 = x.require(&y).unwrap_err();
        let s2 = format!("{}", e2);
        assert!(s2.contains("Duplicate"), "expected duplicate error: {s2}");

        // cycle (fresh pair): M -> N then N -> M should error
        let m = Pass::new("M");
        let n = Pass::new("N");
        m.require(&n).expect("m->n ok");
        let e3 = n.require(&m).unwrap_err();
        let s3 = format!("{}", e3);
        assert!(s3.contains("cycle"), "expected cycle error: {s3}");
    }
}
