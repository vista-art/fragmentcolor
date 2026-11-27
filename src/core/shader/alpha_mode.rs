//! `AlphaMode`, the blend setting a shader keeps.
//! The renderer keys each pipeline by it, since wgpu
//! fixes the blend and depth state when it builds one.

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// How the renderer treats the alpha channel a shader writes.
///
/// The values follow glTF 2.0's `alphaMode` (`OPAQUE`, `MASK`, `BLEND`):
///
/// - `Opaque`: depth-test on, blending off. Alpha bits go to the framebuffer
///   but the blend equation ignores them. Cheapest path; the only one where
///   the depth buffer fully describes the geometry.
/// - `Mask`: the opaque pipeline, with the cut-out left to the fragment shader.
///   The built-in materials `discard` a fragment whose alpha falls below
///   `material.alpha_cutoff`. Hard-edged cut-out; perf profile matches opaque.
/// - `Blend`: depth-test on but depth-write **off**, color target uses
///   standard `SrcAlpha / OneMinusSrcAlpha` over-blend. The renderer sorts
///   Models with this mode back-to-front by eye-space Z before drawing,
///   using the Camera attached via `Pass::add(&camera)`, with no caller-side
///   sorting required. Sort granularity is per-Model (not per-fragment),
///   so self-intersecting translucent meshes can still show artifacts.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlphaMode {
    /// Depth-test on, blending off, no alpha cutoff. Default.
    #[default]
    Opaque,
    /// Depth-test on, blending off, and the fragment shader discards the cut-out.
    Mask,
    /// Depth-test on (depth-write off), standard `SrcAlpha / OneMinusSrcAlpha`
    /// over-blend.
    Blend,
}

impl AlphaMode {
    /// The mode's number in variant order: `Opaque` is 0, `Mask` 1 and `Blend` 2.
    /// The material shaders read it from `material.alpha_mode_flag` to gate the discard.
    pub(crate) fn flag(self) -> u32 {
        match self {
            AlphaMode::Opaque => 0,
            AlphaMode::Mask => 1,
            AlphaMode::Blend => 2,
        }
    }
}
