use crate::enrichments::fixation::Fixation;
use crate::enrichments::gaze::Gaze;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Clone)]
pub struct Enrichments {
    pub gaze: Option<Gaze>,
    pub _fixation: Option<Fixation>,
}

impl Default for Enrichments {
    fn default() -> Self {
        Self {
            gaze: None,
            _fixation: None,
        }
    }
}
