use crate::{
    enrichments::{fixation::FixationOptions, gaze::GazeOptions},
    renderer::Renderables,
};
use serde::{Deserialize, Serialize};
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// The enrichment will own a set of Renderables
/// and are responsible for updating their state
/// after receiving events from the event manager.
pub trait Enrichment<T>: Clone {
    fn handle(&mut self, event: T);
    fn renderables(&self) -> Renderables;
}

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnrichmentOptions {
    pub gaze: Option<GazeOptions>,
    pub _fixation: Option<FixationOptions>,
}
