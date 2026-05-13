//! Scene module — higher-level renderables on top of Mesh + Material.
//!
//! Currently houses [`Model`] (Mesh + Material + transform). A full `Scene`
//! object that owns many Models with traversal / sort hooks is on the
//! roadmap; this module is the landing spot for it when it ships.

use glam::{Mat4, Vec3};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::mesh::Instance;
use crate::{Material, Mesh};

mod platform;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug)]
#[lsp_doc("docs/api/scene/model/model.md")]
pub struct Model {
    pub(crate) mesh: Mesh,
    pub(crate) material: Material,
    pub(crate) transform: RwLock<Mat4>,
}

crate::impl_fc_kind!(Model, "Model");

impl Model {
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new(mesh: Mesh, material: Material) -> Self {
        let model = Self {
            mesh,
            material,
            transform: RwLock::new(Mat4::IDENTITY),
        };
        model.sync_transform();
        model
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
        self.sync_transform();
    }

    /// Pre-multiply by a world-space translation. Result: the model moves by
    /// `offset` in world coordinates.
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate(&self, offset: [f32; 3]) {
        {
            let mut t = self.transform.write();
            *t = Mat4::from_translation(Vec3::from(offset)) * *t;
        }
        self.sync_transform();
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
        {
            let mut t = self.transform.write();
            *t = *t * Mat4::from_axis_angle(axis_vec / length, radians);
        }
        self.sync_transform();
    }

    /// Post-multiply by a per-axis scale (in local space). Result: the model
    /// grows or shrinks around its local origin without moving its origin.
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale(&self, factor: [f32; 3]) {
        {
            let mut t = self.transform.write();
            *t = *t * Mat4::from_scale(Vec3::from(factor));
        }
        self.sync_transform();
    }

    /// Push the current model matrix into the mesh's per-instance attribute
    /// stream as a single instance (4 vec4 columns).
    ///
    /// Why instance attributes instead of a `mesh.model` uniform: the
    /// renderer batches by shader-pipeline hash and issues one draw per
    /// attached mesh, but a uniform is per-shader, so many Models sharing a
    /// Material would collide on the slot. The instance-attribute path is
    /// per-mesh state by design and rides the existing FC instancing
    /// machinery — no renderer changes, no shader duplication, no collisions.
    ///
    /// The first three vertex `@location` slots (0..2) are reserved for
    /// position, normal, and uv0 — the layout `Material::pbr` expects — so
    /// the instance's auto-location counter starts at 3 to avoid colliding
    /// with them in WGSL's shared location namespace.
    ///
    /// **Caveat:** the Mesh's instance buffer is shared across Arc-clones of
    /// the Mesh handle. Two Models that share a Mesh (`Mesh::clone` is an
    /// Arc-clone) collide on the same instance buffer — the most recent
    /// `set_transform` / `translate` / `rotate` / `scale` wins. For batched
    /// instancing (one shared Mesh, many instances), drop down to the
    /// `Mesh::add_instance(...)` + `Material::custom(...)` API directly.
    fn sync_transform(&self) {
        let m = self.transform.read().to_cols_array_2d();
        self.mesh.clear_instances();
        let mut inst = Instance::new();
        inst.next_location = 3;
        self.mesh.add_instance(
            inst.set("model_0", m[0])
                .set("model_1", m[1])
                .set("model_2", m[2])
                .set("model_3", m[3]),
        );
    }
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

    fn instance_row(mesh: &Mesh, key: &str) -> [f32; 4] {
        // Peek at the most recent instance written to the mesh; tests use this
        // to verify Model::sync_transform routed the matrix into the per-
        // instance attribute stream.
        let insts = mesh.object.insts.read();
        let inst = insts.last().expect("mesh has at least one instance");
        match inst.properties.get(key).expect("instance property") {
            crate::mesh::VertexValue::F32x4(v) => *v,
            other => panic!("expected F32x4 for '{key}', got {other:?}"),
        }
    }

    #[test]
    fn new_starts_at_identity_and_writes_one_instance() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        let m = model.transform();
        assert_eq!(m[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m[3], [0.0, 0.0, 0.0, 1.0]);

        assert_eq!(model.mesh.object.insts.read().len(), 1);
        assert_eq!(instance_row(&model.mesh, "model_0"), m[0]);
        assert_eq!(instance_row(&model.mesh, "model_3"), m[3]);
    }

    #[test]
    fn translate_moves_in_world_space() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        model.translate([5.0, 0.0, -2.0]);
        let m = model.transform();
        // Column-major: translation lives in column 3.
        assert_eq!(m[3], [5.0, 0.0, -2.0, 1.0]);
        assert_eq!(instance_row(&model.mesh, "model_3"), [5.0, 0.0, -2.0, 1.0]);
    }

    #[test]
    fn rotate_then_translate_translates_in_world_space() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        model.rotate([0.0, 1.0, 0.0], std::f32::consts::FRAC_PI_2);
        model.translate([1.0, 0.0, 0.0]);
        let m = model.transform();
        // Translation is pre-multiplied (world-space), so column 3 always
        // matches the world offset regardless of prior rotation.
        assert!((m[3][0] - 1.0).abs() < 1.0e-5, "got {m:?}");
        assert!((m[3][2]).abs() < 1.0e-5, "got {m:?}");
    }

    #[test]
    fn scale_is_local_post_multiply() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        model.translate([3.0, 0.0, 0.0]);
        model.scale([2.0, 2.0, 2.0]);
        let m = model.transform();
        // Post-multiplied scale does not move the origin.
        assert_eq!(m[3][0], 3.0);
        // Diagonal scaled.
        assert!((m[0][0] - 2.0).abs() < 1.0e-5);
        assert!((m[1][1] - 2.0).abs() < 1.0e-5);
        assert!((m[2][2] - 2.0).abs() < 1.0e-5);
    }

    #[test]
    fn rotate_ignores_zero_axis() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        let before = model.transform();
        model.rotate([0.0, 0.0, 0.0], 1.57);
        let after = model.transform();
        assert_eq!(before, after, "zero axis must be ignored");
    }

    #[test]
    fn set_transform_replaces_wholesale_and_writes_instance() {
        let model = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        let cols = [
            [2.0_f32, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [4.0, 5.0, 6.0, 1.0],
        ];
        model.set_transform(cols);
        assert_eq!(instance_row(&model.mesh, "model_0"), cols[0]);
        assert_eq!(instance_row(&model.mesh, "model_3"), cols[3]);
    }

    #[test]
    fn two_models_with_distinct_meshes_keep_independent_transforms() {
        let template = Material::pbr().expect("pbr").base_color([0.5, 0.5, 0.5, 1.0]);

        let m1 = Model::new(pbr_triangle_mesh(), template.clone());
        m1.set_transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [10.0, 0.0, 0.0, 1.0],
        ]);

        let m2 = Model::new(pbr_triangle_mesh(), template);
        m2.set_transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-10.0, 0.0, 0.0, 1.0],
        ]);

        // Distinct Meshes → distinct instance buffers; transforms don't
        // collide. Shared Material → shared Shader Arc, single pipeline.
        assert_eq!(instance_row(&m1.mesh, "model_3"), [10.0, 0.0, 0.0, 1.0]);
        assert_eq!(instance_row(&m2.mesh, "model_3"), [-10.0, 0.0, 0.0, 1.0]);
        assert!(std::sync::Arc::ptr_eq(
            &m1.material.shader.object,
            &m2.material.shader.object
        ));
    }

    #[test]
    fn pass_add_model_pushes_shader_and_attaches_mesh() {
        let pass = crate::Pass::new("test");
        let m = Model::new(pbr_triangle_mesh(), Material::pbr().expect("pbr"));
        pass.add_model(&m)
            .expect("Pass::add_model succeeds with PBR-compatible mesh");
        assert!(!pass.is_compute());
    }

    #[test]
    fn pass_add_model_dedupes_shared_shader() {
        let pass = crate::Pass::new("scene");
        let template = Material::pbr().expect("pbr");

        let a = Model::new(pbr_triangle_mesh(), template.clone());
        let b = Model::new(pbr_triangle_mesh(), template);
        pass.add_model(&a).expect("a");
        pass.add_model(&b).expect("b");

        // Two Models sharing one Material → one shader queued on the pass,
        // not two. The renderer iterates shaders once, hits the cached
        // pipeline once, and draws both meshes under it.
        assert_eq!(pass.object.shaders.read().len(), 1);
    }
}
