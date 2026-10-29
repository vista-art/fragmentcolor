use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Failed to parse shader: {0}")]
    ParseError(String),
    #[error("Planned feature not yet implemented: {0}")]
    PlannedFeature(String),
    #[error("Uniform not found: {0}")]
    UniformNotFound(String),
    #[error("Type mismatch for uniform {0}")]
    TypeMismatch(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Field not found in struct: {0}")]
    FieldNotFound(String),
    /// A module naga's writer can't write as WGSL.
    #[error("WGSL error: {0}")]
    WgslError(Diagnostic),
    /// A WGSL source naga's parser refuses.
    #[error("WGSL Parse error: {0}")]
    WgslParseError(Diagnostic),
    /// A type whose layout naga can't compute, such as one past its size limit,
    /// or a struct whose explicit layout the WGSL the browser gets can't keep.
    #[error("Layout error: {0}")]
    LayoutError(String),
    /// A module naga's validator refuses, from WGSL or GLSL.
    #[error("Validation error: {0}")]
    ValidationError(Diagnostic),
    /// A GLSL source naga's parser refuses.
    #[error("GLSL Parse errors: {0}")]
    GlslParseErrors(Diagnostic),
    #[cfg(not(wasm))]
    #[error("URL Request Error: {0}")]
    RequestError(#[from] crate::net::NetworkError),
    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("Index out of bounds for {key}: index {index} >= len {len}")]
    IndexOutOfBounds {
        key: String,
        index: usize,
        len: usize,
    },
    #[error("Shader busy: {0}")]
    Busy(String),
    /// A mesh value that doesn't fit the type its attribute takes.
    #[error("Mesh attribute {name}: expected {expected}, got {found}")]
    AttributeMismatch {
        name: String,
        expected: String,
        found: String,
    },
    /// An attribute one shader typed, which this shader declares another way.
    #[error("Mesh attribute {name} is {fixed} on this mesh, but the shader declares {declared}")]
    AttributeConflict {
        name: String,
        fixed: String,
        declared: String,
    },
    #[cfg(wasm)]
    #[error("WASM Shader Error: {0}")]
    Error(String),
}

/// What naga reports about a shader: its message, with the source lines it points at
/// when it has the source, and each place in the source it labels.
///
/// The types are FragmentColor's own, so a naga release that reworks its errors
/// changes no API. The message is naga's text as it emits it, so each binding
/// shows the same message, and `locations` holds the same places as numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// naga's message, with the lines of source it points at.
    pub message: String,
    /// The places naga labels, in its order. Empty when it labels none, or has no source.
    pub locations: Vec<SourceLocation>,
}

/// A place naga labels in a shader source, with its label.
/// `line` and `column` count from 1, and `length` counts bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub length: u32,
    pub label: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message.trim_end())
    }
}

impl Diagnostic {
    /// naga's report on WGSL `source` that its parser refuses.
    pub(crate) fn from_parse(error: &naga::front::wgsl::ParseError, source: &str) -> Self {
        let labels = error
            .labels()
            .map(|(span, label)| (span, label.to_string()));
        Self {
            message: error.emit_to_string(source),
            locations: locate(labels, source),
        }
    }

    /// naga's report on a module its validator refuses, with the places located
    /// in `source` when the module came from it.
    pub(crate) fn from_validation(
        error: &naga::WithSpan<naga::valid::ValidationError>,
        source: Option<&str>,
    ) -> Self {
        match source {
            Some(source) => Self {
                message: error.emit_to_string(source),
                locations: locate(error.spans().cloned(), source),
            },
            None => Self::from_error(error),
        }
    }

    /// naga's report on GLSL `source` that its parser refuses.
    pub(crate) fn from_glsl(errors: &naga::front::glsl::ParseErrors, source: &str) -> Self {
        let labels = errors
            .errors
            .iter()
            .map(|error| (error.meta, error.kind.to_string()));
        Self {
            message: errors.emit_to_string(source),
            locations: locate(labels, source),
        }
    }

    /// A report with `error`'s message and no places, for an error with no source to point at.
    pub(crate) fn from_error(error: &dyn std::error::Error) -> Self {
        Self {
            message: describe(error),
            locations: Vec::new(),
        }
    }
}

/// Each labeled span in `source`, skipping a span naga leaves undefined.
fn locate(labels: impl Iterator<Item = (naga::Span, String)>, source: &str) -> Vec<SourceLocation> {
    labels
        .filter(|(span, _)| span.is_defined())
        .map(|(span, label)| {
            let place = span.location(source);
            SourceLocation {
                line: place.line_number,
                column: place.line_position,
                length: place.length,
                label,
            }
        })
        .collect()
}

/// `error` and each error under it, joined into one line.
pub(crate) fn describe(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        cause = inner.source();
    }
    text
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for ShaderError {
    fn from(e: pyo3::PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for ShaderError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::Base
    }
}

#[cfg(python)]
impl From<ShaderError> for pyo3::PyErr {
    fn from(e: ShaderError) -> Self {
        crate::error::raise_error(&e)
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for ShaderError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        ShaderError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<ShaderError> for wasm_bindgen::JsValue {
    fn from(error: ShaderError) -> Self {
        crate::error::WebError::from(error).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: ShaderError formats carry details for various failure kinds.
    #[test]
    fn formats_shader_error_variants() {
        let a = ShaderError::ParseError("p".into());
        let b = ShaderError::PlannedFeature("pf".into());
        let c = ShaderError::UniformNotFound("u".into());
        let d = ShaderError::TypeMismatch("k".into());
        let e = ShaderError::InvalidKey("k".into());
        let f = ShaderError::IndexOutOfBounds {
            key: "arr".into(),
            index: 3,
            len: 2,
        };
        let g = ShaderError::FieldNotFound("x".into());
        let h = ShaderError::AttributeMismatch {
            name: "id".into(),
            expected: "u32".into(),
            found: "-1 at vertex 2".into(),
        };
        let i = ShaderError::AttributeConflict {
            name: "id".into(),
            fixed: "u32".into(),
            declared: "f32".into(),
        };

        assert!(a.to_string().contains("parse"));
        assert!(b.to_string().contains("Planned feature"));
        assert!(c.to_string().contains("Uniform not found"));
        assert!(d.to_string().contains("Type mismatch"));
        assert!(e.to_string().contains("Invalid key"));
        assert!(f.to_string().contains("Index out of bounds"));
        assert!(g.to_string().contains("Field not found"));
        assert_eq!(
            h.to_string(),
            "Mesh attribute id: expected u32, got -1 at vertex 2"
        );
        assert_eq!(
            i.to_string(),
            "Mesh attribute id is u32 on this mesh, but the shader declares f32"
        );
    }

    /// The diagnostic in `result`, which the test expects to be the error `pick` reads.
    fn take_diagnostic<T>(
        result: Result<T, ShaderError>,
        pick: impl FnOnce(ShaderError) -> Option<Diagnostic>,
    ) -> Diagnostic {
        let Err(error) = result else {
            panic!("expected an error");
        };
        let text = error.to_string();
        let Some(diagnostic) = pick(error) else {
            panic!("expected another error, got {text}");
        };
        diagnostic
    }

    // Story: a WGSL source naga's parser refuses keeps naga's own message, with the source
    // line it points at, and the place it labels as a line, a column and a length.
    #[test]
    fn wgsl_parse_errors_keep_naga_message_and_place() {
        let source =
            "@fragment fn fs_main() -> @location(0) vec4<f32> {\n  return vec4<f32>(1.0;\n}";
        let naga = naga::front::wgsl::parse_str(source)
            .err()
            .map(|error| error.emit_to_string(source));

        let diagnostic = take_diagnostic(crate::Shader::new(source), |error| match error {
            ShaderError::WgslParseError(diagnostic) => Some(diagnostic),
            _ => None,
        });

        assert_eq!(Some(diagnostic.message.clone()), naga);
        assert!(
            diagnostic.message.contains("wgsl:2:"),
            "{}",
            diagnostic.message
        );
        let place = diagnostic.locations.first().expect("a place");
        assert_eq!(place.line, 2);
        assert!(place.column > 1 && place.length > 0, "{place:?}");
    }

    // Story: a module naga's validator refuses keeps naga's own message, emitted
    // with the source, and the places it labels, which are none for this error.
    #[test]
    fn validation_errors_keep_naga_message() {
        let source = "@fragment fn fs_main() -> @location(0) vec4<f32> {\n  let x = 1.0;\n}";
        let module = naga::front::wgsl::parse_str(source).expect("parses");
        let naga = super::super::create_validator()
            .validate(&module)
            .err()
            .map(|error| (error.emit_to_string(source), error.spans().len()));

        let diagnostic = take_diagnostic(crate::Shader::new(source), |error| match error {
            ShaderError::ValidationError(diagnostic) => Some(diagnostic),
            _ => None,
        });

        let found = (diagnostic.message.clone(), diagnostic.locations.len());
        assert_eq!(Some(found), naga);
    }

    // Story: a GLSL source naga's parser refuses keeps naga's message and the place it labels.
    #[test]
    fn glsl_errors_keep_naga_message_and_place() {
        let vertex = "#version 450\nvoid main() {\n  gl_Position = vec4(1.0)\n}";
        let fragment = "#version 450\nlayout(location = 0) out vec4 color;\nvoid main() { color = vec4(1.0); }";
        let options = naga::front::glsl::Options::from(naga::ShaderStage::Vertex);
        let naga = naga::front::glsl::Frontend::default()
            .parse(&options, vertex)
            .err()
            .map(|errors| errors.emit_to_string(vertex));

        let result = crate::ShaderObject::glsl(vertex, fragment);

        let diagnostic = take_diagnostic(result, |error| match error {
            ShaderError::GlslParseErrors(diagnostic) => Some(diagnostic),
            _ => None,
        });
        assert_eq!(Some(diagnostic.message.clone()), naga);
        let place = diagnostic.locations.first().expect("a place");
        assert!(place.line >= 3, "{place:?}");
    }
}
