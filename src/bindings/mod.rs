//! What every language binding shares: the Python module,
//! the wasm entry point, the number rules and the JavaScript brand.

#[cfg(python)]
pub mod python;

#[cfg(wasm)]
pub mod web;

#[cfg(any(wasm, mobile, python, test))]
pub(crate) mod numbers;

pub(crate) mod brand;

/// A value a binding reads that its call can't take, raised as `InvalidInputError`.
/// Its message names the call first, as in `Texture.writeRegion: a region takes 2, 4 or 6 numbers, got 3`.
#[cfg(any(wasm, python, test))]
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct InputError(pub(crate) String);

#[cfg(any(wasm, python, test))]
impl crate::errors::Classify for InputError {
    fn classify(&self) -> crate::errors::ErrorClass {
        crate::errors::ErrorClass::InvalidInput
    }
}

#[cfg(wasm)]
impl From<InputError> for crate::errors::WebError {
    fn from(error: InputError) -> Self {
        crate::errors::WebError::from_error(&error)
    }
}

#[cfg(python)]
impl From<InputError> for pyo3::PyErr {
    fn from(error: InputError) -> Self {
        crate::errors::raise_error(&error)
    }
}

/// One key of a JS object or a Python dict, in its camelCase and its snake_case spelling.
#[cfg(any(wasm, python, test))]
pub(crate) type Key = (&'static str, &'static str);

/// A class name with its article, such as `a Mesh` or `an AlphaMode`, as a type error names it.
#[cfg(any(wasm, python, test))]
pub(crate) fn name_class(class: &str) -> String {
    let vowel = class.starts_with(['A', 'E', 'I', 'O', 'U']);
    let article = if vowel { "an" } else { "a" };
    format!("{article} {class}")
}

#[cfg(test)]
mod tests {
    use super::name_class;

    // Story: a type error reads "an AlphaMode" and "a Mesh", as English writes them.
    #[test]
    fn name_class_picks_the_article() {
        assert_eq!(name_class("Mesh"), "a Mesh");
        assert_eq!(name_class("AlphaMode"), "an AlphaMode");
        assert_eq!(name_class("UnlitMaterial"), "an UnlitMaterial");
        assert_eq!(name_class("TextureFormat"), "a TextureFormat");
    }
}
