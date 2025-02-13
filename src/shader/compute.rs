use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Uniform;

#[derive(Debug, Serialize, Deserialize)]
// Compute Pipeline
pub struct Compute {
    source: String,
    uniforms: HashMap<String, Uniform>,
    workgroup_size: [u32; 3],
}

impl Compute {
    pub fn new() -> Self {
        unimplemented!("Compute pass is not implemented yet")
    }
}
