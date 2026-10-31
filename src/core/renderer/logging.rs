//! How much FragmentColor logs, set for the whole process with `Renderer::set_log_level`.

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// How much FragmentColor logs, from nothing to every step.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    /// No messages.
    Off,
    /// Errors only.
    Error,
    /// Errors and warnings.
    Warn,
    /// Errors, warnings and notes on progress. The web build starts here.
    Info,
    /// Everything above, and the detail of each step, such as naga's.
    Debug,
    /// Every message.
    Trace,
}

impl LogLevel {
    /// The `log` crate's filter for this level.
    pub(crate) fn to_filter(self) -> log::LevelFilter {
        match self {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Renderer;

    // Story: the level a program sets is the filter the `log` crate applies,
    // so `Debug` lets naga's notes through and `Off` silences everything.
    #[test]
    fn sets_the_process_filter() {
        let before = log::max_level();
        Renderer::set_log_level(LogLevel::Debug);
        assert_eq!(log::max_level(), log::LevelFilter::Debug);
        Renderer::set_log_level(LogLevel::Off);
        assert_eq!(log::max_level(), log::LevelFilter::Off);
        log::set_max_level(before);
    }
}
