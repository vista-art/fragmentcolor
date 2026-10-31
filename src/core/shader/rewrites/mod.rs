//! The rewrites a shader's module takes for a device that can't run it as written.

pub(super) mod immediates;
pub(super) mod struct_layouts;
pub(super) mod uniform_padding;
