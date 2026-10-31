#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(from_py_object, get_all, set_all, eq))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SamplerOptions {
    // wasm-bindgen gives each pub field an unchecked setter.
    // JavaScript uses the checked `repeat_x`, `repeat_y`, `smooth` and `compare` accessors instead.
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    #[cfg_attr(mobile, uniffi(default = false))]
    pub repeat_x: bool,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    #[cfg_attr(mobile, uniffi(default = false))]
    pub repeat_y: bool,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    #[cfg_attr(mobile, uniffi(default = true))]
    pub smooth: bool,
    #[cfg_attr(wasm, wasm_bindgen(skip))]
    #[cfg_attr(mobile, uniffi(default = None))]
    pub compare: Option<CompareFunction>,
}

// SamplerOptions is a value type. The parity audit tracks the methods that take
// it, so its constructor has no doc page.
#[cfg(wasm)]
#[wasm_bindgen]
impl SamplerOptions {
    /// Each option left out keeps its default: no repeat, smooth, no compare.
    #[doc(hidden)]
    #[wasm_bindgen(constructor)]
    pub fn new_js(
        #[wasm_bindgen(unchecked_optional_param_type = "boolean | null")] repeat_x: Option<JsValue>,
        #[wasm_bindgen(unchecked_optional_param_type = "boolean | null")] repeat_y: Option<JsValue>,
        #[wasm_bindgen(unchecked_optional_param_type = "boolean | null")] smooth: Option<JsValue>,
        #[wasm_bindgen(unchecked_optional_param_type = "CompareFunction | null")] compare: Option<
            JsValue,
        >,
    ) -> Result<SamplerOptions, crate::error::WebError> {
        use crate::bindings::web::{read_flag, read_given};
        let compare = match compare {
            Some(compare) => super::parse_compare(&compare)?,
            None => None,
        };
        let defaults = SamplerOptions::default();
        let flag = |value: Option<JsValue>, label: &str, default: bool| match read_given(value) {
            Some(value) => read_flag(&value, label),
            None => Ok(default),
        };
        Ok(SamplerOptions {
            repeat_x: flag(repeat_x, "SamplerOptions: repeat_x", defaults.repeat_x)?,
            repeat_y: flag(repeat_y, "SamplerOptions: repeat_y", defaults.repeat_y)?,
            smooth: flag(smooth, "SamplerOptions: smooth", defaults.smooth)?,
            compare: compare.or(defaults.compare),
        })
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = repeat_x)]
    pub fn repeat_x_js(&self) -> bool {
        self.repeat_x
    }

    /// A value that isn't a boolean throws a `TypeError` and keeps the old one.
    #[doc(hidden)]
    #[wasm_bindgen(setter = repeat_x)]
    pub fn set_repeat_x_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: JsValue,
    ) -> Result<(), crate::error::WebError> {
        self.repeat_x = crate::bindings::web::read_flag(&value, "SamplerOptions.repeat_x")?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = repeat_y)]
    pub fn repeat_y_js(&self) -> bool {
        self.repeat_y
    }

    /// A value that isn't a boolean throws a `TypeError` and keeps the old one.
    #[doc(hidden)]
    #[wasm_bindgen(setter = repeat_y)]
    pub fn set_repeat_y_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: JsValue,
    ) -> Result<(), crate::error::WebError> {
        self.repeat_y = crate::bindings::web::read_flag(&value, "SamplerOptions.repeat_y")?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = smooth)]
    pub fn smooth_js(&self) -> bool {
        self.smooth
    }

    /// A value that isn't a boolean throws a `TypeError` and keeps the old one.
    #[doc(hidden)]
    #[wasm_bindgen(setter = smooth)]
    pub fn set_smooth_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: JsValue,
    ) -> Result<(), crate::error::WebError> {
        self.smooth = crate::bindings::web::read_flag(&value, "SamplerOptions.smooth")?;
        Ok(())
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter = compare)]
    pub fn compare_js(&self) -> Option<CompareFunction> {
        self.compare
    }

    /// `undefined` or `null` clears the compare function, and any other value
    /// that isn't one throws and keeps the old one.
    #[doc(hidden)]
    #[wasm_bindgen(setter = compare)]
    pub fn set_compare_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "CompareFunction | null | undefined")] value: JsValue,
    ) -> Result<(), crate::error::WebError> {
        self.compare = super::parse_compare(&value)?;
        Ok(())
    }
}

#[cfg_attr(python, pyo3::pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct SamplerInfo {
    pub comparison: bool,
}

impl Default for SamplerOptions {
    fn default() -> Self {
        Self {
            repeat_x: false,
            repeat_y: false,
            smooth: true,
            compare: None,
        }
    }
}

pub(crate) fn create_default_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    create_sampler(device, SamplerOptions::default())
}

pub(crate) fn create_sampler(device: &wgpu::Device, options: SamplerOptions) -> wgpu::Sampler {
    let label = format!("{:?}", options);
    let address_mode_u = match options.repeat_x {
        true => wgpu::AddressMode::Repeat,
        false => wgpu::AddressMode::ClampToEdge,
    };
    let address_mode_v = match options.repeat_y {
        true => wgpu::AddressMode::Repeat,
        false => wgpu::AddressMode::ClampToEdge,
    };
    let filter = match options.smooth {
        true => wgpu::FilterMode::Linear,
        false => wgpu::FilterMode::Nearest,
    };
    let mipmap_filter = match options.smooth {
        true => wgpu::MipmapFilterMode::Linear,
        false => wgpu::MipmapFilterMode::Nearest,
    };

    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&label),
        address_mode_u,
        address_mode_v,
        address_mode_w: address_mode_v,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: options.compare.map(CompareFunction::to_wgpu),
        anisotropy_clamp: 1,
        border_color: None,
    })
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompareFunction {
    /// Function never passes
    Never = 1,
    /// Function passes if new value less than existing value
    Less = 2,
    /// Function passes if new value is equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    Equal = 3,
    /// Function passes if new value is less than or equal to existing value
    LessEqual = 4,
    /// Function passes if new value is greater than existing value
    Greater = 5,
    /// Function passes if new value is not equal to existing value. When using
    /// this compare function, make sure to mark your Vertex Shader's `@builtin(position)`
    /// output as `@invariant` to prevent artifacting.
    NotEqual = 6,
    /// Function passes if new value is greater than or equal to existing value
    GreaterEqual = 7,
    /// Function always passes
    Always = 8,
}

impl CompareFunction {
    /// The wgpu `CompareFunction` this `CompareFunction` stands for.
    pub(crate) fn to_wgpu(self) -> wgpu::CompareFunction {
        match self {
            CompareFunction::Never => wgpu::CompareFunction::Never,
            CompareFunction::Less => wgpu::CompareFunction::Less,
            CompareFunction::Equal => wgpu::CompareFunction::Equal,
            CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            CompareFunction::Greater => wgpu::CompareFunction::Greater,
            CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            CompareFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}
