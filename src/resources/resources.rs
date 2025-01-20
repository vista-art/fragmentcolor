use crate::resources::{
    mesh::{MeshData, MeshId},
    texture::{Texture, TextureId},
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

static MESH_ID: AtomicU32 = AtomicU32::new(1);
#[derive(Debug, Default)]
pub struct Resources {
    pub(crate) textures: HashMap<TextureId, Texture>,
    pub(crate) meshes: HashMap<MeshId, MeshData>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    pub fn get_mesh(&self, id: &MeshId) -> Option<&MeshData> {
        self.meshes.get(id)
    }

    pub fn get_texture(&self, id: &TextureId) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshId {
        let index = MeshId(MESH_ID.fetch_add(1, Ordering::Relaxed));
        self.meshes.insert(index, mesh);
        index
    }

    pub fn remove_mesh(&mut self, id: &MeshId) -> Option<MeshData> {
        self.meshes.remove(id)
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureId {
        let index = texture.id;
        self.textures.insert(texture.id, texture);
        index
    }

    pub fn remove_texture(&mut self, id: &TextureId) -> Option<Texture> {
        self.textures.remove(id)
    }
}
