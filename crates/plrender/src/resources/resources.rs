use std::collections::HashMap;

use crate::resources::{
    mesh::{MeshData, MeshId},
    texture::{Texture, TextureId},
};

#[derive(Debug)]
pub struct Resources {
    textures: HashMap<TextureId, Texture>,
    meshes: Vec<MeshData>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            meshes: Vec::new(),
        }
    }

    pub fn get_mesh(&self, id: MeshId) -> &MeshData {
        &self.meshes[id.0 as usize]
    }

    pub fn get_texture(&self, id: TextureId) -> &Texture {
        &self.textures.get(&id).expect("Texture not found")
    }

    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshId {
        let index = self.meshes.len();
        self.meshes.push(mesh);

        return MeshId(index as u32);
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureId {
        let index = texture.id;
        self.textures.insert(texture.id, texture);

        index
    }
}
