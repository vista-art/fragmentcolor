//! Logs each GPU error that no error scope caught,
//! and appends it to the file that `FC_GPU_ERROR_LOG` names.

#[cfg(not(wasm))]
use std::{fs::OpenOptions, io::Write, path::Path};

/// The variable that names the file collecting uncaptured GPU errors.
#[cfg(not(wasm))]
const ERROR_LOG: &str = "FC_GPU_ERROR_LOG";

/// Reports a GPU error that no error scope caught.
/// The logger gets it with the build and runner that hit it,
/// and the file that `FC_GPU_ERROR_LOG` names gets it too, so the
/// iOS, Android and Python healthchecks fail the test that raised it.
pub(in crate::core::renderer) fn report_error(error: wgpu::Error) {
    // Build metadata (compile-time)
    let pkg = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
    let git = option_env!("FC_GIT_HASH").unwrap_or("unknown");
    let built = option_env!("FC_BUILD_TIME").unwrap_or("unknown");
    // Runtime context set by runners like healthchecks
    let runner = std::env::var("FC_RUNNER").unwrap_or_else(|_| "".into());
    let current = std::env::var("FC_CURRENT_TEST").unwrap_or_else(|_| "".into());

    // log::error! reaches every platform's sink, including the browser
    // console on wasm where stdout is discarded.
    log::error!(
        "\n\n==== GPU error ({pkg} v{ver} | {profile} | git {git} | built {built}) ====\nRunner: {runner}\nContext: {current}\n\n{error:#?}\n"
    );

    #[cfg(not(wasm))]
    if let Some(path) = std::env::var_os(ERROR_LOG) {
        append_error(Path::new(&path), &error);
    }
}

/// Appends `error` to the file at `path`, creating it when missing.
#[cfg(not(wasm))]
fn append_error(path: &Path, error: &wgpu::Error) {
    let appended = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{error}"));
    if let Err(problem) = appended {
        log::warn!(
            "Failed to append a GPU error to {}: {problem}",
            path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    // Story: a healthcheck reads the GPU errors of a test from the file FC_GPU_ERROR_LOG names.
    #[cfg(not(wasm))]
    #[test]
    fn appends_each_error_to_the_log() {
        // Arrange
        let dir = tempfile::tempdir().expect("tempdir");
        let log = dir.path().join("gpu-errors.log");
        let error = |description: &str| wgpu::Error::Validation {
            source: description.into(),
            description: description.into(),
        };

        // Act
        super::append_error(&log, &error("first"));
        super::append_error(&log, &error("second"));

        // Assert
        let text = std::fs::read_to_string(&log).expect("read the log");
        assert_eq!(text, "first\nsecond\n");
    }
}
