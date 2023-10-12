use crate::events::SceneEvent;
use hecs::World;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

// This represents a 3D model in a scene.
// It contains the 3D model, instance data, and a parent ID (TBD)
pub struct Node {
    // ID of parent Node
    pub parent: u32,
    // Local position of model (for relative calculations)
    pub locals: Locals,
    // The vertex buffers and texture data
    pub model: super::Model,
    // An array of positional data for each instance (can just pass 1 instance)
    pub instances: Vec<Instance>,
}

/// The scene owns a set of Renderables
/// and is responsible for updating their state
/// after receiving events from the event manager.

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Default)]
pub struct Scene {
    world: hecs::World,
    entities: Vec<Entity>,
}

#[cfg_attr(wasm, wasm_bindgen)]
impl Scene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            entities: Vec::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) {
        let components = entity.components();
        self.world.spawn(entity);
    }

    pub fn update_entity(&mut self, entity: Entity) {
        let components = entity.components();
        self.world.spawn(entity);
    }

    pub fn update(&mut self) {
        for renderable in self.renderables() {
            let mut renderable = renderable.write().unwrap();
            if renderable.should_update() {
                renderable.update();
                let label = renderable.label();
                let buffer = &state.buffers[&label];

                state
                    .queue
                    .write_buffer(buffer, 0, renderable.uniform_bytes().as_slice());
            }
        }
    }

    pub fn renderables(&self) -> Vec<Entity> {
        // query entities that have a renderable component
        let renderables = self
            .world
            .query::<&Renderable>()
            .iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
    }

    pub fn handle(&self, event: SceneEvent) {}
}
