use serde::{Deserialize, Serialize};

use crate::ShaderError;

use super::Shader;

#[derive(Debug, Serialize, Deserialize)]
// Compute Pipeline
pub struct Compute {
    shader: Shader,
}

impl Compute {
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        let shader = Shader::new(source)?;
        Ok(Self { shader })
    }
}
