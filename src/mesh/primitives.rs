use lsp_doc::lsp_doc;
use std::sync::Arc;

use super::{Mesh, MeshObject, Vertex};

#[cfg(python)]
use pyo3::prelude::*;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(wasm, wasm_bindgen)]
#[lsp_doc("docs/api/geometry/quad/quad.md")]
pub struct Quad {
    object: Arc<MeshObject>,
}

impl Quad {
    #[lsp_doc("docs/api/geometry/quad/new.md")]
    pub fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        // Build 2 triangles with position (vec2) and uv (vec2)
        let mesh = Mesh::new();
        let (minx, miny) = (min[0], min[1]);
        let (maxx, maxy) = (max[0], max[1]);

        // Quad vertices (two triangles):
        // v0 (minx, miny) -> uv(0,0)
        // v1 (maxx, miny) -> uv(1,0)
        // v2 (maxx, maxy) -> uv(1,1)
        // v3 (minx, maxy) -> uv(0,1)
        let v0 = Vertex::new([minx, miny]).set("uv", [0.0f32, 0.0]);
        let v1 = Vertex::new([maxx, miny]).set("uv", [1.0f32, 0.0]);
        let v2 = Vertex::new([maxx, maxy]).set("uv", [1.0f32, 1.0]);
        let v3 = Vertex::new([minx, maxy]).set("uv", [0.0f32, 1.0]);

        // Triangle list: v0, v1, v2, v0, v2, v3
        mesh.add_vertices([v0.clone(), v1.clone(), v2.clone(), v0, v2, v3]);

        Self {
            object: mesh.object,
        }
    }

    #[lsp_doc("docs/api/geometry/quad/get_mesh.md")]
    pub fn get_mesh(&self) -> Mesh {
        Mesh {
            object: self.object.clone(),
            pass: Arc::new(crate::pass::PassObject::new(
                "Quad Mesh Debug Pass",
                crate::pass::PassType::Render,
            )),
        }
    }
}

#[cfg(python)]
#[pymethods]
impl Quad {
    #[new]
    #[lsp_doc("docs/api/geometry/quad/new.md")]
    pub fn new_py(min: [f32; 2], max: [f32; 2]) -> Self {
        Self::new(min, max)
    }

    #[pyo3(name = "get_mesh")]
    #[lsp_doc("docs/api/geometry/quad/get_mesh.md")]
    pub fn get_mesh_py(&self) -> Mesh {
        self.get_mesh()
    }
}

impl From<Quad> for Mesh {
    fn from(q: Quad) -> Self {
        q.get_mesh()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VertexValue;

    #[test]
    fn quad_vertices_and_uv_layout() {
        let q = Quad::new([0.0, 0.0], [1.0, 1.0]);
        let mesh = q.get_mesh();

        // Access the underlying vertices (crate-visible)
        let verts = mesh.object.verts.read().clone();
        assert_eq!(verts.len(), 6, "quad should emit 6 vertices (2 triangles)");

        // Expected triangle list: v0,v1,v2, v0,v2,v3
        let p = |i: usize| -> [f32; 4] {
            let v = &verts[i];
            [
                v.position.0.x,
                v.position.0.y,
                v.position.0.z,
                v.position.0.w,
            ]
        };
        let uv = |i: usize| -> Option<[f32; 2]> {
            match verts[i].properties.get("uv") {
                Some(VertexValue::F32x2(v)) => Some(*v),
                _ => None,
            }
        };

        assert_eq!(p(0), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(p(1), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(p(2), [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(p(3), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(p(4), [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(p(5), [0.0, 1.0, 0.0, 1.0]);

        assert_eq!(uv(0), Some([0.0, 0.0]));
        assert_eq!(uv(1), Some([1.0, 0.0]));
        assert_eq!(uv(2), Some([1.0, 1.0]));
        assert_eq!(uv(3), Some([0.0, 0.0]));
        assert_eq!(uv(4), Some([1.0, 1.0]));
        assert_eq!(uv(5), Some([0.0, 1.0]));

        // Location map: position at 0, uv at 1
        let (pos_loc, rev) = mesh.object.first_vertex_location_map();
        assert_eq!(pos_loc, 0);
        assert_eq!(rev.get(&1).map(|s| s.as_str()), Some("uv"));
    }

    #[test]
    fn get_mesh_uses_named_pass() {
        let q = Quad::new([0.0, 0.0], [1.0, 1.0]);
        let mesh = q.get_mesh();
        let pass_name = mesh.pass.name.clone();
        assert_eq!(pass_name.as_ref(), "Quad Mesh Debug Pass");
    }
}
