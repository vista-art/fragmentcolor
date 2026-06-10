//! Scene module — higher-level abstractions on top of Mesh + Material.
//!
//! Houses [`Scene`] (the top-level container), [`Model`] (Mesh + Material +
//! transform), [`Camera`], the unified [`Light`] type with three
//! constructors (`Light::directional` / `Light::point` / `Light::spot`),
//! and the [`SceneObject`] trait that ties them together. The split
//! mirrors glTF / USD: a Scene is a flat list of nodes (geometry,
//! viewpoints, lights), and the renderer walks the scene to produce a
//! frame.

use glam::{Mat4, Vec3};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::mesh::MeshObject;
use crate::shader::ShaderObject;
use crate::{Material, Mesh};

mod camera;
mod light;
mod loader;
mod object;
mod platform;
#[allow(clippy::module_inception)]
mod scene;

pub use camera::*;
pub use light::*;
pub use loader::{GltfSource, SceneLoadError, SceneSource};
pub use object::SceneObject;
pub use scene::{PassRef, Scene};

/// Per-Model live state read by the renderer's draw-queue build. Folded
/// into one lock so the per-frame `build_pass_draws` walk takes a single
/// acquisition per entry. Hidden Models skip the queue entirely, so
/// reading `visible` and `transform` together under one lock is the
/// natural shape.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelState {
    pub(crate) transform: Mat4,
    pub(crate) visible: bool,
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/model/model.md")]
pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) material: Material,
    // Arc-shared so the Pass can hold a *live* reference through
    // `Pass::add`; the renderer reads the current values at draw time.
    // Mutating `Model::translate`, `Model::set_visible`, etc., after
    // `pass.add(&model)` is picked up on the next render — no re-add
    // needed. One lock covers transform + visibility so the renderer's
    // per-Model walk halves its lock acquisitions.
    pub(crate) state: Arc<RwLock<ModelState>>,
}

impl SceneObject for Model {
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError> {
        let shader_arc = &self.material.shader.object;

        // Declare the model-instance vertex schema on the Mesh first, so the
        // shader's mesh-validation (inside `add_mesh` below) sees `model_0..3`
        // at the per-instance slot and accepts the layout. The Mesh's `insts`
        // stays empty; per-Model transforms ride the Pass's `model_entries`
        // queue and the renderer builds the buffer at draw time.
        if self.mesh.object.instance_schema.read().is_none() {
            self.mesh.object.declare_model_instance_schema();
        }

        // Dedupe shader-attach: many Models that share one Material's Shader
        // should result in one entry on the pass. The renderer iterates
        // shaders linearly and a doubled entry would set pipeline + bind
        // groups twice for the same draws. When a new shader joins, replay
        // every scene object on the pass against it so order-of-add (Camera
        // before Model, lights after Model, …) doesn't matter.
        let shader_present = pass
            .object
            .shaders
            .read()
            .iter()
            .any(|s| Arc::ptr_eq(s, shader_arc));
        if !shader_present {
            pass.add_shader(&self.material.shader);
            pass.replay_scene_objects(&self.material.shader);
        }

        // Dedupe mesh-attach onto the shader. Without this the renderer
        // would iterate the doubly-attached mesh and draw it twice.
        let mesh_attached = shader_arc
            .meshes
            .read()
            .iter()
            .any(|m| Arc::ptr_eq(m, &self.mesh.object));
        if !mesh_attached {
            self.material.shader.add_mesh(&self.mesh)?;
        }

        pass.object.model_entries.write().push(ModelEntry {
            shader: shader_arc.clone(),
            mesh: self.mesh.object.clone(),
            state: self.state.clone(),
        });
        Ok(())
    }
}

crate::impl_fc_kind!(Model, "Model");

impl Model {
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self {
            mesh,
            material,
            state: Arc::new(RwLock::new(ModelState {
                transform: Mat4::IDENTITY,
                visible: true,
            })),
        }
    }

    /// Read the current visibility flag.
    #[lsp_doc("docs/api/scene/model/visible.md")]
    pub fn visible(&self) -> bool {
        self.state.read().visible
    }

    /// Toggle the Model in or out of the next render. Hidden Models are
    /// skipped by the renderer in both the opaque-batched and
    /// blend-sorted draw paths — no re-attach needed. Useful for
    /// LOD switches, level transitions, or temporarily hiding helpers
    /// without rebuilding the Scene.
    #[lsp_doc("docs/api/scene/model/set_visible.md")]
    pub fn set_visible(&self, visible: bool) {
        self.state.write().visible = visible;
    }

    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material(&self) -> &Material {
        &self.material
    }

    /// Read the current model matrix in column-major order, matching WGSL
    /// `mat4x4<f32>` storage and glam `to_cols_array_2d()`.
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform(&self) -> [[f32; 4]; 4] {
        self.state.read().transform.to_cols_array_2d()
    }

    /// Replace the model matrix wholesale, in column-major order.
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform(&self, matrix: [[f32; 4]; 4]) {
        self.state.write().transform = Mat4::from_cols_array_2d(&matrix);
    }

    /// Pre-multiply by a world-space translation. Result: the model moves by
    /// `offset` in world coordinates.
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate(&self, offset: [f32; 3]) {
        let mut s = self.state.write();
        s.transform = Mat4::from_translation(Vec3::from(offset)) * s.transform;
    }

    /// Post-multiply by a rotation around the given axis (in local space).
    /// Result: the model spins in place when it sits at the world origin and
    /// orbits its local-origin offset otherwise.
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate(&self, axis: [f32; 3], radians: f32) {
        let axis_vec = Vec3::from(axis);
        let length = axis_vec.length();
        if !length.is_finite() || length < 1.0e-6 {
            log::warn!("Model::rotate ignored: axis is zero-length");
            return;
        }
        let mut s = self.state.write();
        s.transform *= Mat4::from_axis_angle(axis_vec / length, radians);
    }

    /// Post-multiply by a per-axis scale (in local space). Result: the model
    /// grows or shrinks around its local origin without moving its origin.
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale(&self, factor: [f32; 3]) {
        let mut s = self.state.write();
        s.transform *= Mat4::from_scale(Vec3::from(factor));
    }
}

/// A single Model queued on a Pass: the Material's Shader, the Model's Mesh,
/// and a live Arc reference to the Model's transform + visibility. The
/// renderer groups entries by (Shader, Mesh) and builds a per-Pass instance
/// buffer (4 vec4 columns per entry, taken at draw time so transform
/// mutations between `add_model` and `render` are picked up live). One
/// lock covers both fields so the per-frame walk takes a single read per
/// Model.
#[derive(Debug, Clone)]
pub(crate) struct ModelEntry {
    pub(crate) shader: Arc<ShaderObject>,
    pub(crate) mesh: Arc<MeshObject>,
    pub(crate) state: Arc<RwLock<ModelState>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Vertex;

    fn pbr_triangle_mesh() -> Mesh {
        let mesh = Mesh::new();
        for (p, uv) in [
            ([0.0, 0.5, 0.0], [0.5, 1.0]),
            ([-0.5, -0.5, 0.0], [0.0, 0.0]),
            ([0.5, -0.5, 0.0], [1.0, 0.0]),
        ] {
            mesh.add_vertex(Vertex::pbr(p).set(Vertex::UV0, uv));
        }
        mesh
    }

    fn pbr_material() -> Material {
        Material::pbr()
    }

    #[test]
    fn new_starts_at_identity() {
        let model = Model::new(pbr_triangle_mesh(), pbr_material());
        let m = model.transform();
        assert_eq!(m[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(m[2], [0.0, 0.0, 1.0, 0.0]);
        assert_eq!(m[3], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn translate_moves_in_world_space() {
        let model = Model::new(pbr_triangle_mesh(), pbr_material());
        model.translate([5.0, 0.0, -2.0]);
        let m = model.transform();
        assert_eq!(m[3], [5.0, 0.0, -2.0, 1.0]);
    }

    #[test]
    fn rotate_then_translate_translates_in_world_space() {
        let model = Model::new(pbr_triangle_mesh(), pbr_material());
        model.rotate([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
        model.translate([1.0, 0.0, 0.0]);
        let m = model.transform();
        assert!((m[3][0] - 1.0).abs() < 1.0e-5, "got {m:?}");
        assert!((m[3][2]).abs() < 1.0e-5, "got {m:?}");
    }

    #[test]
    fn scale_is_local_post_multiply() {
        let model = Model::new(pbr_triangle_mesh(), pbr_material());
        model.translate([3.0, 0.0, 0.0]);
        model.scale([2.0, 2.0, 2.0]);
        let m = model.transform();
        assert_eq!(m[3][0], 3.0);
        assert!((m[0][0] - 2.0).abs() < 1.0e-5);
        assert!((m[1][1] - 2.0).abs() < 1.0e-5);
        assert!((m[2][2] - 2.0).abs() < 1.0e-5);
    }

    #[test]
    fn rotate_ignores_zero_axis() {
        let model = Model::new(pbr_triangle_mesh(), pbr_material());
        let before = model.transform();
        model.rotate([0.0, 0.0, 0.0], 1.57);
        let after = model.transform();
        assert_eq!(before, after, "zero axis must be ignored");
    }

    #[test]
    fn clone_shares_transform_live() {
        // Model::clone is a shallow Arc-share; mutating one handle's transform
        // is visible on the clone (and on any Pass that already holds a live
        // entry — that's how batched instancing picks up updates between
        // `Pass::add_model` and `Renderer::render`).
        let m1 = Model::new(pbr_triangle_mesh(), pbr_material());
        let m2 = m1.clone();
        m1.translate([7.0, 0.0, 0.0]);
        assert_eq!(m2.transform()[3], [7.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn pass_add_model_queues_one_entry() {
        let pass = crate::Pass::new("test");
        let m = Model::new(pbr_triangle_mesh(), pbr_material());
        pass.add(&m).expect("add_model");
        assert_eq!(pass.object.model_entries.read().len(), 1);
        assert_eq!(pass.object.shaders.read().len(), 1);
        assert!(!pass.is_compute());
    }

    #[test]
    fn pass_add_model_dedupes_shared_shader_and_mesh() {
        let pass = crate::Pass::new("scene");
        let template = pbr_material();

        let a = Model::new(pbr_triangle_mesh(), template.clone());
        let b = Model::new(pbr_triangle_mesh(), template);
        pass.add(&a).expect("a");
        pass.add(&b).expect("b");

        // Two Models sharing one Material → one shader queued on the pass,
        // two model entries.
        assert_eq!(pass.object.shaders.read().len(), 1);
        assert_eq!(pass.object.model_entries.read().len(), 2);
    }

    #[test]
    fn pass_add_model_carries_live_transform() {
        let pass = crate::Pass::new("scene");
        let m = Model::new(pbr_triangle_mesh(), pbr_material());
        pass.add(&m).expect("add_model");

        // Mutate the Model AFTER add_model. The Pass's entry holds an Arc to
        // the same RwLock<Mat4>, so the new value is observable through the
        // entry.
        m.set_transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [9.0, 0.0, 0.0, 1.0],
        ]);
        let entries = pass.object.model_entries.read();
        let live = entries[0].state.read().transform.to_cols_array_2d();
        assert_eq!(live[3], [9.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn visible_defaults_true_and_toggles() {
        let m = Model::new(pbr_triangle_mesh(), pbr_material());
        assert!(m.visible(), "default visibility must be true");
        m.set_visible(false);
        assert!(!m.visible());
        m.set_visible(true);
        assert!(m.visible());
    }

    #[test]
    fn pass_entry_shares_visibility_flag_with_model() {
        // The Pass holds an Arc-clone of the Model's visibility flag — same
        // share semantics as the transform — so toggling on the Model
        // shows up in the renderer's view of the entry immediately.
        let pass = crate::Pass::new("scene");
        let m = Model::new(pbr_triangle_mesh(), pbr_material());
        pass.add(&m).expect("add_model");

        m.set_visible(false);
        let entries = pass.object.model_entries.read();
        assert!(
            !entries[0].state.read().visible,
            "visibility flag should propagate live through the Arc"
        );
    }

    #[test]
    fn hidden_model_does_not_render() {
        // Sanity check that the renderer's draw-queue build honours the
        // visibility flag. We can't directly observe the queue (PassDraws
        // is private to the renderer module), but a hidden Model + a
        // visible Model on the same Pass should produce the same image as
        // just the visible Model alone — and a fully-hidden Pass should
        // produce the clear-color-only image.
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");
            let model = Model::new(
                pbr_triangle_mesh(),
                Material::pbr().base_color([0.6, 0.2, 0.8, 1.0]),
            );
            model.set_visible(false);
            let scene = crate::Scene::new();
            scene.add(&model).expect("add");
            renderer.render(&scene, &target).expect("render");
            use crate::Target;
            let image = target.get_image().await;
            // Hidden Model + clear-color background. Every pixel's RGB
            // should be the clear color's RGB (default black, [0, 0, 0]);
            // the triangle's purple [0.6, 0.2, 0.8] never lands in the
            // buffer. The alpha component depends on the clear color's
            // default — we don't constrain it here.
            for px in image.chunks_exact(4) {
                assert_eq!(
                    [px[0], px[1], px[2]],
                    [0, 0, 0],
                    "hidden model should not contribute to the rendered pixels"
                );
            }
        });
    }
}
