//! Scene module — higher-level renderables on top of Mesh + Material.
//!
//! Currently houses [`Model`] (Mesh + Material + transform). A full `Scene`
//! object that owns many Models with traversal / sort hooks is on the
//! roadmap; this module is the landing spot for it when it ships.

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
mod platform;

pub use camera::*;
pub use light::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/model/model.md")]
pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) material: Material,
    // Arc-shared so the Pass can hold a *live* reference to the transform
    // through `Pass::add_model`; the renderer reads the current value at draw
    // time. Mutating `Model::translate` after `Pass::add_model` is picked up on
    // the next render — no re-add needed.
    pub(crate) transform: Arc<RwLock<Mat4>>,
}

crate::impl_fc_kind!(Model, "Model");

impl Model {
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self {
            mesh,
            material,
            transform: Arc::new(RwLock::new(Mat4::IDENTITY)),
        }
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
        self.transform.read().to_cols_array_2d()
    }

    /// Replace the model matrix wholesale, in column-major order.
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform(&self, matrix: [[f32; 4]; 4]) {
        *self.transform.write() = Mat4::from_cols_array_2d(&matrix);
    }

    /// Pre-multiply by a world-space translation. Result: the model moves by
    /// `offset` in world coordinates.
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate(&self, offset: [f32; 3]) {
        let mut t = self.transform.write();
        *t = Mat4::from_translation(Vec3::from(offset)) * *t;
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
        let mut t = self.transform.write();
        *t = *t * Mat4::from_axis_angle(axis_vec / length, radians);
    }

    /// Post-multiply by a per-axis scale (in local space). Result: the model
    /// grows or shrinks around its local origin without moving its origin.
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale(&self, factor: [f32; 3]) {
        let mut t = self.transform.write();
        *t = *t * Mat4::from_scale(Vec3::from(factor));
    }
}

/// A single Model queued on a Pass: the Material's Shader, the Model's Mesh,
/// and a live Arc reference to the Model's transform. The renderer groups
/// entries by (Shader, Mesh) and builds a per-Pass instance buffer (4 vec4
/// columns per entry, taken at draw time so transform mutations between
/// `add_model` and `render` are picked up live).
#[derive(Debug, Clone)]
pub(crate) struct ModelEntry {
    pub(crate) shader: Arc<ShaderObject>,
    pub(crate) mesh: Arc<MeshObject>,
    pub(crate) transform: Arc<RwLock<Mat4>>,
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
            mesh.add_vertex(
                Vertex::new(p)
                    .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                    .set(Vertex::UV0, uv),
            );
        }
        mesh
    }

    fn pbr_material() -> Material {
        pollster::block_on(Material::pbr(&crate::Renderer::new())).expect("pbr")
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
        pass.add_model(&m).expect("add_model");
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
        pass.add_model(&a).expect("a");
        pass.add_model(&b).expect("b");

        // Two Models sharing one Material → one shader queued on the pass,
        // two model entries.
        assert_eq!(pass.object.shaders.read().len(), 1);
        assert_eq!(pass.object.model_entries.read().len(), 2);
    }

    #[test]
    fn pass_add_model_carries_live_transform() {
        let pass = crate::Pass::new("scene");
        let m = Model::new(pbr_triangle_mesh(), pbr_material());
        pass.add_model(&m).expect("add_model");

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
        let live = entries[0].transform.read().to_cols_array_2d();
        assert_eq!(live[3], [9.0, 0.0, 0.0, 1.0]);
    }
}
