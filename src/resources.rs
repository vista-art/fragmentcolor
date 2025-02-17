use crate::Texture;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceId(String);

#[derive(Debug, Default)]
pub struct ResourceRegistry {
    textures: HashMap<ResourceId, Texture>,
    buffers: HashMap<ResourceId, wgpu::Buffer>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            buffers: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, id: ResourceId, texture: Texture) {
        self.textures.insert(id, texture);
    }

    pub fn add_buffer(&mut self, id: ResourceId, buffer: wgpu::Buffer) {
        self.buffers.insert(id, buffer);
    }

    pub fn get_texture(&self, id: &ResourceId) -> Option<&Texture> {
        self.textures.get(id)
    }

    pub fn get_buffer(&self, id: &ResourceId) -> Option<&wgpu::Buffer> {
        self.buffers.get(id)
    }
}
