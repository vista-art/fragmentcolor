// DRAFT

use crate::shader::Shader;
use crate::Color;

enum RenderTarget {
    Texture,
    Surface,
}

struct RenderPass {
    shader: Shader,
    target: RenderTarget,
    clear_color: Option<Color>,
}

struct ComputePass {
    shader: Shader,
    dispatch: (u32, u32, u32),
}

/// A Pass can be a Render Pass or a Compute Pass.
pub enum Pass {
    Render(RenderPass),
    Compute(ComputePass),
}
