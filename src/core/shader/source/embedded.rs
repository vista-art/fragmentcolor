//! Embedded shader registry — a feature-gated lookup from registry slugs
//! (e.g. `"postfx/vignette"`) to the shader source compiled into the binary.
//!
//! `scripts/embedded_shaders.rs` generates the `lookup` function below into OUT_DIR
//! from the contents of `docs/website/public/shaders/`. Each match arm is
//! gated on `shaders-<category>`, so only the categories the consumer opts
//! into are baked into the binary; everything else falls through to the
//! URL-fetch path in `source::fetch_url`.

include!(concat!(env!("OUT_DIR"), "/embedded_shaders.rs"));
