use lsp_doc::lsp_doc;
use std::sync::Arc;

use super::{Mesh, MeshObject, Vertex};

#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/mesh/primitives/quad/quad.md")]
pub struct Quad {
    object: Arc<MeshObject>,
}

impl Quad {
    #[lsp_doc("docs/api/core/mesh/primitives/quad/new.md")]
    pub fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        // Build 2 triangles with position (vec2) and uv (vec2)
        let mut mesh = Mesh::new();
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

    #[lsp_doc("docs/api/core/mesh/primitives/quad/get_mesh.md")]
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

impl From<Quad> for Mesh {
    fn from(q: Quad) -> Self {
        q.get_mesh()
    }
}
