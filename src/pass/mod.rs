use crate::texture::Registration;
use crate::{Color, Mesh, Renderable, ScreenRegion, Shader, ShaderObject};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod platform;

pub mod error;
pub use error::*;

/// What a render pass does with its target before drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PassInput {
    /// Keep what the target holds.
    Load,
    /// Clear the target to this color.
    Clear(Color),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
    Compute,
    Render,
}

impl std::fmt::Display for PassType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PassType::Compute => formatter.write_str("compute"),
            PassType::Render => formatter.write_str("render"),
        }
    }
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
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
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
        let obj = Arc::new(PassObject::from_shader(name, shader.object.clone()));
        PassObject::ensure_flat_current(&obj);
        Self { object: obj }
    }

    #[lsp_doc("docs/api/core/pass/name.md")]
    pub fn name(&self) -> String {
        self.object.name.to_string()
    }

    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous(&self) {
        self.object.load_previous();
    }

    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader(&self, shader: &Shader) -> Result<(), PassError> {
        self.object.add_shader(shader.object.clone())
    }

    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh(&self, mesh: &Mesh) -> Result<(), PassError> {
        self.object.add_mesh(mesh)
    }

    /// Absorb a [`SceneObject`](crate::scene::SceneObject) — Model, Camera,
    /// Light, or any custom node — into this pass. Each implementation owns
    /// its own attach behavior: Model queues a draw, Camera and Light wire
    /// their uniforms into every shader the pass renders (current and
    /// future). Chainable; errors short-circuit the chain.
    #[lsp_doc("docs/api/core/pass/add.md")]
    pub fn add<O: crate::scene::SceneObject>(&self, object: &O) -> Result<&Self, PassError> {
        object.attach(self)?;
        Ok(self)
    }

    /// Re-invoke `apply_to_shader` on every absorbed scene object for the
    /// given shader. Called by `Model::attach` when a Model brings a new
    /// shader to the pass, so a Camera writes its state into it. The pass's
    /// lights reach the shader through `add_shader`.
    pub(crate) fn replay_scene_objects(&self, shader: &crate::Shader) {
        for object in self.object.scene_objects.read().iter() {
            object.apply_to_shader(shader);
        }
    }

    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport<R>(&self, viewport: R) -> Result<(), PassError>
    where
        R: TryInto<ScreenRegion>,
        PassError: From<R::Error>,
    {
        self.object.set_viewport(viewport.try_into()?)
    }

    #[lsp_doc("docs/api/core/pass/set_clear_color.md")]
    pub fn set_clear_color(&self, color: impl Into<Color>) -> Result<(), PassError> {
        self.object.set_clear_color(color)
    }

    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch(&self, x: u32, y: u32, z: u32) {
        self.object.set_compute_dispatch(x, y, z);
    }

    #[lsp_doc("docs/api/core/pass/set_target.md")]
    pub fn set_target<T>(&self, target: T) -> Result<(), PassError>
    where
        T: TryInto<ColorTarget, Error = PassError>,
    {
        let ct = target.try_into()?;
        self.object.set_color_target(ct.registration);
        Ok(())
    }

    #[lsp_doc("docs/api/core/pass/set_depth_target.md")]
    pub fn set_depth_target<T>(&self, target: T) -> Result<(), PassError>
    where
        T: TryInto<DepthTarget, Error = PassError>,
    {
        let dt = target.try_into()?;
        self.object.set_depth_target(dt.registration);
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
        let required = self.object.flat.read().clone();
        required
            .iter()
            .cloned()
            .chain([self.object.clone()])
            .collect()
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
    pub(crate) registration: Arc<Registration>,
}

/// A reference to a depth render target texture (Depth32Float or similar).
/// Must be a stable texture created by the Renderer (same device/context) with RENDER_ATTACHMENT usage and a depth/stencil format.
#[derive(Clone, Debug)]
pub struct DepthTarget {
    pub(crate) registration: Arc<Registration>,
}

impl TryFrom<&crate::texture::Texture> for ColorTarget {
    type Error = PassError;

    /// Validate usage and format for color attachment
    fn try_from(tex: &crate::texture::Texture) -> Result<Self, Self::Error> {
        let object = tex.object();
        let usage = object.usage;
        if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            return Err(PassError::InvalidColorTarget(
                "texture was not created with RENDER_ATTACHMENT usage".into(),
            ));
        }
        let fmt = object.format;
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
        check_plane(&object, "color")?;
        Ok(ColorTarget {
            registration: tex.registration.clone(),
        })
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
        let object = tex.object();
        let usage = object.usage;
        if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            return Err(PassError::InvalidColorTarget(
                "depth texture was not created with RENDER_ATTACHMENT usage".into(),
            ));
        }
        let format = object.format;
        match format {
            wgpu::TextureFormat::Depth32Float
            | wgpu::TextureFormat::Depth16Unorm
            | wgpu::TextureFormat::Depth24Plus
            | wgpu::TextureFormat::Depth24PlusStencil8
            | wgpu::TextureFormat::Depth32FloatStencil8 => {}
            _ => {
                return Err(PassError::InvalidColorTarget(
                    "texture format is not a depth/stencil format".into(),
                ));
            }
        }
        check_plane(&object, "depth")?;
        Ok(DepthTarget {
            registration: tex.registration.clone(),
        })
    }
}

/// Refuses a texture with more than one mip level or layer as a `role` target.
/// A pass draws into one level of one layer and has no way to pick it,
/// so the other levels and layers would keep what they held.
fn check_plane(object: &crate::TextureObject, role: &str) -> Result<(), PassError> {
    let levels = object.inner.mip_level_count();
    if levels != 1 {
        return Err(PassError::InvalidColorTarget(format!(
            "a {role} target has one mip level, and this texture has {levels}; create it with mipmaps set to false"
        )));
    }
    let layers = object.size.depth_or_array_layers;
    if layers != 1 {
        return Err(PassError::InvalidColorTarget(format!(
            "a {role} target has one layer, and this texture has {layers}"
        )));
    }
    Ok(())
}

#[cfg(wasm)]
crate::impl_js_bridge!(Pass, PassError);

static GRAPH_VERSION: AtomicU64 = AtomicU64::new(1);

/// World-space camera state cached on a Pass via `Camera::attach`. Carried
/// here (not just inside the shader uniform) so the renderer can read the
/// eye position and view matrix without round-tripping through the shader's
/// `Shader::get("camera.position")` path — which would only work for
/// shaders that happen to expose those uniforms under canonical names.
///
/// Used by `process_render_pass` to sort the `Pass::model_entries` whose
/// Material declares `alpha_mode: Blend` back-to-front by eye-space depth
/// before submitting the blend phase. Updated whenever the Camera's
/// `propagate` runs (i.e. on `Camera::look_at` after `pass.add(&camera)`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraSnapshot {
    /// View matrix (world → camera), column-major. Mirrors the `view`
    /// component of `Camera::view_proj`. The renderer needs this for the
    /// sort — eye-space Z is `(view * world)[2]`, not just `length(eye -
    /// world)`. World-space distance ties incorrectly for objects spread
    /// laterally on a vertical plane (a row of glass panels viewed
    /// edge-on); eye-Z resolves them in the right order.
    pub(crate) view: [[f32; 4]; 4],
    /// The camera's `proj * view`, column-major. The renderer bins the
    /// pass's lights in its clip space.
    pub(crate) view_proj: [[f32; 4]; 4],
}

/// Hands each pass a number of its own, which keys its light sets.
static NEXT_PASS_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct PassObject {
    /// Unique in the process.
    pub(crate) id: u64,
    pub pass_type: RwLock<PassType>,
    pub(crate) name: Arc<str>,
    pub(crate) input: RwLock<PassInput>,
    pub(crate) shaders: RwLock<Vec<Arc<ShaderObject>>>,
    // Per-Pass live references to Models queued via `Pass::add_model`. Each entry
    // carries an Arc-shared transform so the renderer reads the current value at
    // draw time. Grouped by (shader, mesh) the entries become the rows of a
    // Pass-owned instance buffer — that's the batched-instancing path: many
    // Models sharing a (Material, Mesh) pair collapse to one draw with N
    // instances, with no shader duplication and no mutation of the Mesh's own
    // instance buffer (so the same Mesh in another Pass with different Models
    // doesn't clobber state).
    pub(crate) model_entries: RwLock<Vec<crate::scene::ModelEntry>>,
    // Scene-level objects (Camera, Light, ...) absorbed via `Pass::add`.
    // Each one's `apply_to_shader` hook is re-invoked when a new shader
    // joins the pass via `Model::attach`, so order of attachment doesn't
    // matter. Implementors that need live propagation (Camera, Light)
    // track their own `Weak<ShaderObject>` list so subsequent mutations
    // on the source value reach every absorbed shader without a second
    // `add` call. Model doesn't go in this list (its work is one-shot).
    pub(crate) scene_objects: RwLock<Vec<Box<dyn crate::scene::SceneObject>>>,
    /// The lights added to this pass, in add order. Holding them keeps them
    /// lighting after the caller drops its handles, and a shader that joins
    /// the pass later takes every one.
    pub(crate) lights: RwLock<crate::scene::PassLights>,
    pub(crate) viewport: RwLock<Option<ScreenRegion>>,
    pub(crate) required_buffer_size: RwLock<u64>,
    /// Snapshot of the most recently attached Camera's `(view, position)`,
    /// taken when `Camera::attach` runs (and refreshed by `Camera::look_at`
    /// through `Camera::propagate`). The renderer reads this to sort
    /// translucent draws (`alpha_mode: Blend`) back-to-front by per-Model
    /// eye-space depth. `None` means no Camera was attached to this pass
    /// — the renderer falls back to insertion order, which is correct for
    /// opaque-only passes and the best it can do otherwise.
    ///
    /// Storing the view matrix (not just the eye position) lets the sort
    /// account for non-axis-aligned cameras — eye-Z depth in camera space
    /// is what matters for over-blend correctness, not world-space
    /// distance from the eye. Stored as a column-major `[[f32; 4]; 4]` to
    /// match `Camera::view_proj` and `glam::Mat4::to_cols_array_2d`.
    pub(crate) camera_snapshot: RwLock<Option<CameraSnapshot>>,
    // For compute passes: dispatch size (defaults to 1,1,1)
    pub(crate) compute_dispatch: RwLock<(u32, u32, u32)>,
    // Optional per-pass color target. Holding its registration keeps the
    // texture in the registry after the caller drops its handle.
    pub(crate) color_target: RwLock<Option<Arc<Registration>>>,
    // Optional per-pass depth attachment (Depth32Float for now)
    pub(crate) depth_target: RwLock<Option<Arc<Registration>>>,
    // DAG metadata and cached traversal
    pub(crate) dependencies: RwLock<Vec<Arc<PassObject>>>,
    pub(crate) dependency_names: RwLock<std::collections::HashSet<Arc<str>>>,
    // Every pass this one requires, deps first. Never this pass:
    // an Arc to itself would keep the pass and its shaders alive forever.
    pub(crate) flat: RwLock<Arc<[Arc<PassObject>]>>,
    pub(crate) built_version: AtomicU64,
}

impl PassObject {
    /// Create a new Pass object with the given name and type.
    pub fn new(name: &str, pass_type: PassType) -> Self {
        Self {
            id: NEXT_PASS_ID.fetch_add(1, Ordering::Relaxed),
            name: Arc::from(name),
            shaders: RwLock::new(Vec::new()),
            model_entries: RwLock::new(Vec::new()),
            scene_objects: RwLock::new(Vec::new()),
            lights: RwLock::new(crate::scene::PassLights::default()),
            viewport: RwLock::new(None),
            input: RwLock::new(PassInput::Clear(Color::transparent())),
            required_buffer_size: RwLock::new(0),
            camera_snapshot: RwLock::new(None),
            pass_type: RwLock::new(pass_type),
            compute_dispatch: RwLock::new((1, 1, 1)),
            color_target: RwLock::new(None),
            depth_target: RwLock::new(None),
            dependencies: RwLock::new(Vec::new()),
            dependency_names: RwLock::new(std::collections::HashSet::new()),
            flat: RwLock::new(Arc::from(Vec::<Arc<PassObject>>::new().into_boxed_slice())),
            built_version: AtomicU64::new(0),
        }
    }

    /// Makes the pass clear to `color`, and keeps its input when a channel isn't finite.
    pub fn set_clear_color(&self, color: impl Into<Color>) -> Result<(), PassError> {
        *self.input.write() = PassInput::Clear(color.into().check_finite()?);
        Ok(())
    }

    /// Keep what the target holds instead of clearing it.
    pub(crate) fn load_previous(&self) {
        *self.input.write() = PassInput::Load;
    }

    /// Sets the workgroup counts of a compute pass, as given. A 0 dispatches nothing.
    pub fn set_compute_dispatch(&self, x: u32, y: u32, z: u32) {
        *self.compute_dispatch.write() = (x, y, z);
    }

    /// Get the compute dispatch size for compute passes.
    pub fn compute_dispatch(&self) -> (u32, u32, u32) {
        *self.compute_dispatch.read()
    }

    /// Adds a shader to this pass. The first shader sets the pass's kind, and
    /// a later one must match it. A lit shader takes every light the pass holds,
    /// and when they don't fit, the pass stays as it was.
    pub(crate) fn add_shader(&self, shader: Arc<ShaderObject>) -> Result<(), PassError> {
        let lights = self.lights.read();
        self.check_shader(&shader, &lights)?;
        for light in lights.iter() {
            shader.add_light(&light.object);
        }
        self.push_shader(shader);
        Ok(())
    }

    /// Checks that `shader` can join this pass, changing nothing: its kind
    /// matches the pass's, and it has room for every light the pass holds.
    pub(crate) fn admit(&self, shader: &Arc<ShaderObject>) -> Result<(), PassError> {
        self.check_shader(shader, &self.lights.read())
    }

    /// The checks of `add_shader`, against the pass's `lights`.
    fn check_shader(
        &self,
        shader: &Arc<ShaderObject>,
        lights: &crate::scene::PassLights,
    ) -> Result<(), PassError> {
        let kind = PassType::from(shader.clone());
        if !self.shaders.read().is_empty() {
            let pass = *self.pass_type.read();
            if pass != kind {
                return Err(PassError::ShaderKind { pass, shader: kind });
            }
        }
        let incoming: Vec<u64> = lights.iter().map(|light| light.object.id).collect();
        shader.check_light_room(&incoming)
    }

    /// Adds a shader with no checks, for a pass that has no shader and no light.
    pub(crate) fn push_shader(&self, shader: Arc<ShaderObject>) {
        let mut shaders = self.shaders.write();
        if shaders.is_empty() {
            *self.pass_type.write() = PassType::from(shader.clone());
        }
        *self.required_buffer_size.write() += shader.total_bytes;
        shaders.push(shader);
    }

    pub(crate) fn set_color_target(&self, target: Arc<Registration>) {
        *self.color_target.write() = Some(target);
    }

    pub(crate) fn set_depth_target(&self, target: Arc<Registration>) {
        *self.depth_target.write() = Some(target);
    }

    /// Add a mesh to the last shader in this pass that accepts it.
    /// When none does, the error carries the reason the last shader gave.
    pub fn add_mesh(&self, mesh: &Mesh) -> Result<(), PassError> {
        let shaders = self.shaders.read();
        let mut first_error = None;
        for shader in shaders.iter().rev() {
            match shader.add_mesh(mesh.object.clone()) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        match first_error {
            Some(error) => Err(PassError::NoCompatibleShader(error)),
            None => Err(PassError::NoShader),
        }
    }

    /// Keeps the viewport, or refuses a region with no width or no height.
    pub fn set_viewport(&self, viewport: ScreenRegion) -> Result<(), PassError> {
        *self.viewport.write() = Some(viewport.check()?);
        Ok(())
    }

    pub fn is_compute(&self) -> bool {
        matches!(*self.pass_type.read(), PassType::Compute)
    }

    pub(crate) fn from_shader(name: &str, shader: Arc<ShaderObject>) -> Self {
        let object = Self::new(name, PassType::from(shader.clone()));
        object.push_shader(shader);
        object
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

    // Ensure cached deps-first slice of required passes is up-to-date with global graph version
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
        let deps = self_arc.dependencies.read().clone();
        for dep in deps.iter() {
            dfs(dep, &mut seen, &mut out);
        }
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
        Self::from_shader("Default Pass from ShaderObject", shader)
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
        pass.add_shader(&s1).expect("a render shader");
        pass.add_shader(&s2).expect("a render shader");
        let after = *pass.object.required_buffer_size.read();

        // Assert
        assert!(after >= before);
        assert_eq!(pass.object.shaders.read().len(), 2);
        assert!(!pass.object.is_compute());
    }

    // Story: A compute shader on a render pass, or the reverse, returns ShaderKind
    // naming both kinds, and the pass doesn't change.
    #[test]
    fn rejects_mismatched_shader_kinds() {
        // Arrange: a render pass with one render shader, and a compute pass with one compute shader
        let render_shader = Shader::default();
        let compute_src = r#"
            @group(0) @binding(0) var<uniform> u: vec4<f32>;
            @compute @workgroup_size(1)
            fn cs() { _ = u; }
        "#;
        let compute_shader = Shader::new(compute_src).expect("compute shader");
        let render = Pass::from_shader("render pass", &render_shader);
        let compute = Pass::from_shader("compute pass", &compute_shader);
        let bytes_before = *render.object.required_buffer_size.read();

        // Act
        let on_render = render.add_shader(&compute_shader);
        let on_compute = compute.add_shader(&render_shader);

        // Assert: each names both kinds, and neither pass changed
        assert!(
            matches!(
                on_render,
                Err(PassError::ShaderKind {
                    pass: PassType::Render,
                    shader: PassType::Compute,
                })
            ),
            "{on_render:?}"
        );
        assert!(
            matches!(
                on_compute,
                Err(PassError::ShaderKind {
                    pass: PassType::Compute,
                    shader: PassType::Render,
                })
            ),
            "{on_compute:?}"
        );
        assert_eq!(render.object.shaders.read().len(), 1);
        assert_eq!(compute.object.shaders.read().len(), 1);
        assert_eq!(*render.object.required_buffer_size.read(), bytes_before);
        assert!(!render.object.is_compute());
        assert!(compute.object.is_compute());
    }

    // Story: set_clear_color makes the pass clear its target; load_previous makes it keep it.
    #[test]
    fn toggles_input_between_clear_and_load() {
        let pass = Pass::new("p");
        let input = |pass: &Pass| *pass.object.input.read();

        assert!(pass.set_clear_color([0.1, 0.2, 0.3, 0.4]).is_ok());
        assert_eq!(
            input(&pass),
            PassInput::Clear(Color::from([0.1, 0.2, 0.3, 0.4]))
        );

        pass.load_previous();
        assert_eq!(input(&pass), PassInput::Load);

        // Any Color form works: three channels are opaque, and a Color passes through.
        assert!(pass.set_clear_color([1.0, 0.0, 0.0]).is_ok());
        assert_eq!(
            input(&pass),
            PassInput::Clear(Color::from([1.0, 0.0, 0.0, 1.0]))
        );
        assert!(pass.set_clear_color(Color::white()).is_ok());
        assert_eq!(input(&pass), PassInput::Clear(Color::white()));
    }

    // Story: a clear color keeps values past 0 to 1 for float targets, and a
    // channel that is NaN or infinite fails and leaves the pass as it was.
    #[test]
    fn keeps_the_clear_color_and_refuses_nan() {
        let pass = Pass::new("p");
        let input = |pass: &Pass| *pass.object.input.read();

        assert!(pass.set_clear_color([4.0, -1.0, 0.25, 1.0]).is_ok());
        assert_eq!(
            input(&pass),
            PassInput::Clear(Color::new(4.0, -1.0, 0.25, 1.0))
        );

        let error = pass.set_clear_color([f32::NAN, 0.0, 0.0]).err();
        assert!(matches!(error, Some(PassError::Color(_))), "{error:?}");
        assert!(pass.set_clear_color([0.0, 0.0, f32::INFINITY]).is_err());
        assert_eq!(
            input(&pass),
            PassInput::Clear(Color::new(4.0, -1.0, 0.25, 1.0))
        );
    }

    // Story: setting a viewport is stored and can be read back.
    #[test]
    fn sets_viewport_rect() {
        // Arrange
        let pass = Pass::new("p");
        let vp = ScreenRegion::from_region(2, 4, 8, 6).expect("a region with an area");

        // Act
        assert!(pass.set_viewport(vp).is_ok());

        // Assert
        assert_eq!(pass.object.viewport.read().as_ref(), Some(vp).as_ref());
    }

    // Story: a viewport with no width or no height fails and keeps the old
    // one, whether it comes as an array or as a region built field by field.
    #[test]
    fn refuses_an_empty_viewport() {
        let pass = Pass::new("p");
        assert!(pass.set_viewport([2, 4, 8, 6]).is_ok());

        let error = pass.set_viewport([0, 0, 0, 4]).err();
        assert!(matches!(error, Some(PassError::Viewport(_))), "{error:?}");
        assert!(pass.set_viewport((4, 0)).is_err());
        let flat = ScreenRegion {
            min_x: 3,
            min_y: 3,
            max_x: 9,
            max_y: 3,
        };
        assert!(pass.set_viewport(flat).is_err());

        let kept = ScreenRegion::from_region(2, 4, 8, 6).ok();
        assert_eq!(*pass.object.viewport.read(), kept);
    }

    // Story: compute dispatch keeps a 0 in any dimension, which dispatches nothing.
    #[test]
    fn compute_dispatch_keeps_zeros() {
        let p = Pass::compute("c");
        p.set_compute_dispatch(0, 2, 0);
        assert_eq!(*p.object.compute_dispatch.read(), (0, 2, 0));
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
                .create_storage_texture(([4u32, 4u32], crate::TextureFormat::default()))
                .await
                .expect("color tex");
            let dt_bad = DepthTarget::try_from(&color_tex);
            assert!(dt_bad.is_err());
        });
    }

    // Story: a pass draws into one mip level of one layer, so a texture with a
    // mip chain or with layers is refused as a target, and the pass keeps none.
    #[test]
    fn targets_refuse_mip_chains_and_layers() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let usage = crate::TextureUsage::RENDER_ATTACHMENT;
            let options = |size: [u32; 3], mipmaps: bool| {
                crate::TextureOptions {
                    size: Some(size.into()),
                    mipmaps,
                    ..Default::default()
                }
                .with_usage(usage)
            };
            let chain = renderer
                .create_texture((vec![255; 64], options([4, 4, 1], true)))
                .await
                .expect("a texture with a mip chain");
            let layered = renderer
                .create_storage_texture(crate::TextureInput {
                    data: crate::TextureData::Empty,
                    options: options([4, 4, 2], false),
                })
                .await
                .expect("a texture with two layers");
            let flat = renderer
                .create_texture((vec![255; 64], options([4, 4, 1], false)))
                .await
                .expect("a texture with one level");

            let pass = Pass::new("p");
            for texture in [&chain, &layered] {
                let error = pass.set_target(texture).err();
                assert!(
                    matches!(error, Some(PassError::InvalidColorTarget(_))),
                    "{error:?}"
                );
            }
            let error = pass.set_target(&chain).err().map(|error| error.to_string());
            assert!(
                error.is_some_and(|text| text.contains("mipmaps set to false")),
                "the error names the fix"
            );
            assert!(pass.object.color_target.read().is_none());
            pass.set_target(&flat).expect("one level of one layer");
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

    // Story: a pass runs after every pass it requires, and each pass runs once.
    #[test]
    fn require_chain_runs_dependencies_first() {
        let color = Pass::new("color");
        let blur_x = Pass::new("blur_x");
        blur_x.require(&color).expect("blur_x requires color");
        let blur_y = Pass::new("blur_y");
        blur_y.require(&blur_x).expect("blur_y requires blur_x");
        let compose = Pass::new("compose");
        compose.require(&color).expect("compose requires color");
        compose.require(&blur_y).expect("compose requires blur_y");

        let names: Vec<String> = compose
            .passes()
            .iter()
            .map(|pass| pass.name.to_string())
            .collect();

        assert_eq!(names, ["color", "blur_x", "blur_y", "compose"]);
    }

    // Story: dropping a rendered pass chain frees every pass and the shaders only they held.
    #[test]
    fn dropped_pass_frees_its_shaders() {
        pollster::block_on(async move {
            // Arrange: a draw pass that requires a compute pass
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([4u32, 4u32])
                .await
                .expect("texture target");
            let source = r#"
                @group(0) @binding(0) var<storage, read_write> particles: array<f32, 64>;
                @compute @workgroup_size(64)
                fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
                    particles[id.x] += 1.0;
                }
            "#;
            let simulation = Shader::new(source).expect("compute shader");
            let drawing = Shader::default();
            let simulate = Pass::from_shader("simulate", &simulation);
            let draw = Pass::from_shader("draw", &drawing);
            draw.require(&simulate).expect("draw requires simulate");
            let passes = [
                Arc::downgrade(&simulate.object),
                Arc::downgrade(&draw.object),
            ];
            let shaders = [
                Arc::downgrade(&simulation.object),
                Arc::downgrade(&drawing.object),
            ];

            // Act
            renderer.render(&draw, &target).expect("render");
            let bytes = renderer
                .read_storage(&simulation, "particles")
                .await
                .expect("read_storage");
            drop((simulation, drawing, simulate, draw));

            // Assert: the required pass ran, then nothing outlived its handle
            assert!(
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|chunk| f32::from_le_bytes(*chunk) == 1.0),
                "the required pass did not run"
            );
            assert!(
                passes.iter().all(|pass| pass.upgrade().is_none()),
                "a pass leaked"
            );
            assert!(
                shaders.iter().all(|shader| shader.upgrade().is_none()),
                "a shader leaked"
            );
        });
    }
}
