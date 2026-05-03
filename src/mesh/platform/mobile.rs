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

use crate::renderer::platform::mobile::FragmentColorError;
use crate::mesh::{Instance, Mesh, Vertex, VertexValue};

// -----------------------------------------------------------------
// Vertex (uniffi bindings)
// -----------------------------------------------------------------

#[uniffi::export]
impl Vertex {
    /// Create a new `Vertex` from a position vector. Accepts 2, 3, or 4
    /// components; any other length returns an error. Mobile shim because
    /// uniffi cannot marshal the generic `impl IntoVertexPositionFull`.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/geometry/vertex/hidden/new_mobile.md")]
    pub fn new_mobile(position: Vec<f32>) -> Result<Arc<Self>, FragmentColorError> {
        let v = match position.len() {
            2 => Vertex::new([position[0], position[1]]),
            3 => Vertex::new([position[0], position[1], position[2]]),
            4 => Vertex::new([position[0], position[1], position[2], position[3]]),
            n => {
                return Err(FragmentColorError::Render(format!(
                    "Vertex position must have 2, 3, or 4 components; got {}",
                    n
                )));
            }
        };
        Ok(Arc::new(v))
    }

    /// Set a vertex attribute by key. `VertexValue` is a `uniffi::Enum` so
    /// Swift/Kotlin callers construct the variant directly.
    ///
    /// Returns the updated `Vertex` (builder style).
    #[uniffi::method(name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/hidden/set_mobile.md")]
    pub fn set_mobile(self: Arc<Self>, key: String, value: VertexValue) -> Arc<Self> {
        Arc::new((*self).clone().set(&key, value))
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
    /// Create an empty `Instance`. Chain `.set(key:value:)` calls to populate
    /// per-instance attributes.
    #[uniffi::constructor(name = "new")]
    pub fn new_mobile() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Set a per-instance attribute by key.
    ///
    /// Returns the updated `Instance` (builder style).
    #[uniffi::method(name = "set")]
    pub fn set_mobile(self: Arc<Self>, key: String, value: VertexValue) -> Arc<Self> {
        Arc::new((*self).clone().set(&key, value))
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
        for v in vertices {
            mesh.add_vertex((*v).clone());
        }
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
        for v in vertices {
            self.add_vertex((*v).clone());
        }
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
        for inst in instances {
            self.add_instance((*inst).clone());
        }
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
}
