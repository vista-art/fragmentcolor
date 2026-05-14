//! Material alpha mode — glTF 2.0 PBR-MR `alphaMode` semantics for the pipeline state.
//!
//! `AlphaMode` is a pipeline-state flag carried on `Material`. The renderer
//! bakes it into the `RenderPipelineKey` so different modes for the same
//! shader cache to distinct pipelines (wgpu requires the depth/blend state
//! at pipeline-build time).

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// How the renderer interprets the alpha channel of a Material's output.
///
/// Mirrors the glTF 2.0 `alphaMode` field — `OPAQUE`, `MASK`, `BLEND` — with
/// the same semantics:
///
/// - `Opaque`: depth-test on, blending off. Alpha bits go to the framebuffer
///   but the blend equation ignores them. Cheapest path; the only one where
///   the depth buffer fully describes the geometry.
/// - `Mask`: depth-test on, blending off, but fragments with
///   `material.base_color.a < material.alpha_cutoff` are `discard`ed in the
///   fragment shader. Hard-edged cut-out; perf profile matches opaque.
/// - `Blend`: depth-test on but depth-write **off**, color target uses
///   standard `SrcAlpha / OneMinusSrcAlpha` over-blend. The renderer sorts
///   Models with this mode back-to-front by eye-space Z before drawing,
///   using the Camera attached via `Pass::add(&camera)` — no caller-side
///   sorting required. Sort granularity is per-Model (not per-fragment),
///   so self-intersecting translucent meshes can still show artifacts.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlphaMode {
    /// Depth-test on, blending off, no alpha cutoff. Default.
    Opaque,
    /// Depth-test on, blending off, fragment discarded if
    /// `material.base_color.a < material.alpha_cutoff`.
    Mask,
    /// Depth-test on (depth-write off), standard `SrcAlpha / OneMinusSrcAlpha`
    /// over-blend.
    Blend,
}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

impl AlphaMode {
    /// Numeric flag the WGSL fragment shader uses to gate the mask discard.
    /// `Opaque = 0`, `Mask = 1`, `Blend = 2`. Stored in the `alpha_mode_flag`
    /// field of the `PbrMaterial` uniform.
    pub(crate) fn flag(self) -> u32 {
        match self {
            AlphaMode::Opaque => 0,
            AlphaMode::Mask => 1,
            AlphaMode::Blend => 2,
        }
    }
}
