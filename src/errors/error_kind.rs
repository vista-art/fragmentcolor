//! `ErrorKind`: the kind of GPU error a device error carries.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[lsp_doc("docs/api/errors/error_kind/error_kind.md")]
pub enum ErrorKind {
    /// The GPU refused a call that breaks a rule of the API.
    Validation,
    /// The GPU failed for a reason of its own, such as a driver bug.
    Internal,
    /// The device ran out of memory.
    Memory,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ErrorKind::Validation => "validation",
            ErrorKind::Internal => "internal",
            ErrorKind::Memory => "memory",
        };
        formatter.write_str(name)
    }
}
