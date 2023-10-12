use crate::events::handler::EventHandler;
use std::sync::{Arc, RwLock};
use winit::event_loop::{EventLoop, EventLoopClosed, EventLoopProxy};

/// The base Event type.
///
pub enum Event {
    SceneEvent,
    RenderTargetEvent,
    RendererEvent,
    ResourceEvent,
    UserCommand,
}

#[derive(Debug, Clone)]
pub enum SceneEvent {
    EntityAdded,
    EntityRemoved,
    EntityUpdated,
    AddComponent,
    RemoveComponent,
    UpdateComponent,
}

pub enum EntityEvent {
    ComponentAdded,
    ComponentRemoved,
    ComponentUpdated,
}
