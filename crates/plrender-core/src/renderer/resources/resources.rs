use crate::renderer::resources::{
    mesh::{Mesh, MeshId},
    texture::{Texture, TextureId},
};

pub struct Resources {
    textures: Vec<Texture>,
    meshes: Vec<Mesh>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub fn get_mesh(&self, id: MeshId) -> &Mesh {
        &self.meshes[id.0 as usize]
    }

    pub fn get_texture(&self, id: TextureId) -> &Texture {
        &self.textures[id.0 as usize]
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        let index = self.meshes.len();
        self.meshes.push(mesh);

        return MeshId(index as u32);
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureId {
        let index = self.textures.len();
        self.textures.push(texture);

        return TextureId(index as u32);
    }
}
