use crate::{
    controllers::{fixation::FixationOptions, gaze::GazeOptions},
    events::VipEvent,
    renderer::Renderables,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// The controller owns a set of Renderables
/// and is responsible for updating their state
/// after receiving events from the event manager.
pub trait Controller<E> {
    fn handle(&mut self, event: E);
    fn renderables(&self) -> &Renderables;
    fn should_update(&self) -> bool;
}

pub type VipController = Box<dyn Controller<VipEvent>>;
pub type Controllers = Vec<VipController>;

impl Debug for VipController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Controller{{handle: {}, renderables: {}}}",
            "handle formatter not implemented", "renderables formatter not implemented"
        )
    }
}

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ControllerOptions {
    pub gaze: Option<GazeOptions>,
    pub _fixation: Option<FixationOptions>,
}
