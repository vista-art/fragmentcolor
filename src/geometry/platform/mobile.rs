//! Mobile (Swift / Kotlin) uniffi bindings for `Vertex`, `Instance`, and `Mesh`.
//!
//! Uniffi cannot marshal generic parameters (`impl Into<T>`, `<P: Trait>`), so
//! each method gets a concrete mobile entry point. The naming convention mirrors
//! the web and Python platform modules: mobile-specific methods carry a
//! `_mobile` suffix on the Rust side so the build-time doc scanner can keep
//! them separate from the Rust-only API, and every uniffi export carries an
//! explicit `name = "..."` attribute to expose idiomatic camelCase names in
//! Swift and Kotlin.
//!
//! Hidden per-language docs under `docs/api/geometry/{object}/hidden/<method>_mobile.md`
//! satisfy the build-time documentation validator without polluting the
//! main website.

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::bindings::numbers::{VectorError, read_floats, read_position, read_vector};
use crate::core::renderer::platform::mobile::FragmentColorError;
use crate::{Instance, Mesh, Quad, Vertex, VertexData};

// -----------------------------------------------------------------
// Vertex (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Vertex {
    /// Create a new `Vertex` from a position vector of 1 to 4 components.
    /// Any other length returns an error. Mobile shim because uniffi cannot
    /// marshal the generic `impl IntoVertexPositionFull`.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/geometry/vertex/hidden/new_mobile.md")]
    pub fn new_mobile(position: Vec<f32>) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(read_position(position, "Vertex: position")?))
    }

    /// Construct a `Vertex` with PBR-shader defaults seeded for the
    /// optional attributes (`NORMAL`, `UV0`, `COLOR0`, `UV1`, `TANGENT`).
    /// Mobile shim because uniffi cannot marshal the generic position
    /// type.
    #[uniffi::constructor(name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_mobile(position: Vec<f32>) -> Result<Arc<Self>, FragmentColorError> {
        let position = read_floats(position, "Vertex.pbr: position")?;
        let v = match position.as_slice() {
            [x] => Vertex::pbr(*x),
            [x, y] => Vertex::pbr([*x, *y]),
            [x, y, z] => Vertex::pbr([*x, *y, *z]),
            [x, y, z, w] => Vertex::pbr([*x, *y, *z, *w]),
            other => return Err(position_error(other.len())),
        };
        Ok(Arc::new(v))
    }

    /// Set a vertex attribute to 1 to 4 numbers.
    /// The shader that reads it decides their type when the mesh attaches.
    #[uniffi::method(name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/hidden/set_mobile.md")]
    pub fn set_mobile(
        self: Arc<Self>,
        key: String,
        value: Vec<f64>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = VertexData::try_from(value.as_slice())?;
        Ok(Arc::new((*self).clone().set(&key, value)))
    }

    /// Create an `Instance` seeded from this vertex's per-vertex attributes.
    #[uniffi::method(name = "createInstance")]
    #[lsp_doc("docs/api/geometry/vertex/hidden/create_instance_mobile.md")]
    pub fn create_instance_mobile(self: Arc<Self>) -> Arc<Instance> {
        Arc::new(self.create_instance())
    }
}

// -----------------------------------------------------------------
// Instance (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Instance {
    /// Create an empty `Instance`. Chain `.set(key, value)` calls to populate
    /// per-instance attributes.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/geometry/instance/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Set a per-instance attribute to 1 to 4 numbers.
    /// The shader that reads it decides their type when the mesh attaches.
    #[uniffi::method(name = "set")]
    #[lsp_doc("docs/api/geometry/instance/hidden/set_mobile.md")]
    pub fn set_mobile(
        self: Arc<Self>,
        key: String,
        value: Vec<f64>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let value = VertexData::try_from(value.as_slice())?;
        Ok(Arc::new((*self).clone().set(&key, value)))
    }
}

// -----------------------------------------------------------------
// Mesh (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Mesh {
    /// Create an empty `Mesh`.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/geometry/mesh/new.md")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Mesh::new())
    }

    /// Create a `Mesh` pre-populated from an array of vertices.
    #[uniffi::constructor(name = "fromVertices")]
    #[lsp_doc("docs/api/geometry/mesh/from_vertices.md")]
    pub fn from_vertices_mobile(vertices: Vec<Arc<Vertex>>) -> Arc<Self> {
        let mesh = Mesh::new();
        mesh.add_vertices(vertices.iter().map(|vertex| (**vertex).clone()));
        Arc::new(mesh)
    }

    /// Append a single vertex to the mesh.
    #[uniffi::method(name = "addVertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_mobile(&self, v: Arc<Vertex>) {
        self.add_vertex((*v).clone());
    }

    /// Append multiple vertices to the mesh.
    #[uniffi::method(name = "addVertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_mobile(&self, vertices: Vec<Arc<Vertex>>) {
        self.add_vertices(vertices.iter().map(|vertex| (**vertex).clone()));
    }

    /// Append a single instance data record to the mesh.
    #[uniffi::method(name = "addInstance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_mobile(&self, instance: Arc<Instance>) {
        self.add_instance((*instance).clone());
    }

    /// Append multiple instance data records to the mesh.
    #[uniffi::method(name = "addInstances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_mobile(&self, instances: Vec<Arc<Instance>>) {
        self.add_instances(instances.iter().map(|instance| (**instance).clone()));
    }

    /// Remove all per-instance attribute data, resetting to single-instance rendering.
    #[uniffi::method(name = "clearInstances")]
    #[lsp_doc("docs/api/geometry/mesh/clear_instances.md")]
    pub fn clear_instances_mobile(&self) {
        self.clear_instances();
    }

    /// Override how many instances to draw (when not using per-instance attributes).
    #[uniffi::method(name = "setInstanceCount")]
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count_mobile(&self, n: u32) {
        self.set_instance_count(n);
    }

    /// Supply explicit indices, bypassing the auto-derived dedup pass.
    #[uniffi::method(name = "setIndices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_mobile(&self, indices: Vec<u32>) {
        self.set_indices(indices);
    }

    /// Drop user-supplied indices and return to auto-derived dedup indexing.
    #[uniffi::method(name = "clearIndices")]
    #[lsp_doc("docs/api/geometry/mesh/clear_indices.md")]
    pub fn clear_indices_mobile(&self) {
        self.clear_indices();
    }
}

// -----------------------------------------------------------------
// Quad (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Quad {
    /// Create a `Quad` from two corners (min.xy, max.xy) in clip-space coordinates.
    /// Accepts exactly 2 components per argument; any other length returns an error.
    /// Mobile shim because uniffi cannot marshal fixed-size arrays.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/geometry/quad/hidden/new_mobile.md")]
    pub fn new_mobile(min: Vec<f32>, max: Vec<f32>) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Quad::new(
            read_vector::<2>(min, "Quad: min")?,
            read_vector::<2>(max, "Quad: max")?,
        )))
    }

    /// Get the `Mesh` built by this `Quad`.
    #[uniffi::method(name = "getMesh")]
    #[lsp_doc("docs/api/geometry/quad/hidden/get_mesh_mobile.md")]
    pub fn get_mesh_mobile(self: Arc<Self>) -> Arc<Mesh> {
        Arc::new(self.get_mesh())
    }
}

/// The error for a `Vertex.pbr` position that isn't 1 to 4 numbers long.
fn position_error(count: usize) -> FragmentColorError {
    VectorError::Length {
        label: "Vertex.pbr: position".to_string(),
        expected: "1 to 4".to_string(),
        count,
    }
    .into()
}
