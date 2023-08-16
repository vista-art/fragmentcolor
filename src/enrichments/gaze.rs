use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GazeConfig {
    pub size: u32,
    pub color: String,
    pub opacity: f32,
}
