include!(concat!(env!("OUT_DIR"), "/built.rs"));

impl AppMetadata for crate::app::App {}
impl AppMetadata for crate::app::AppState {}

pub trait AppMetadata {
    /// Prints the build information to the standard output.
    fn print_build_info(&self) {
        print!("{}", self.build_info());
    }

    /// Returns the build information as a string.
    fn build_info(&self) -> String {
        format!(
            "
            ⭕  FragmentColor Version {}
                {}
                Pupil Labs GmbH - All rights reserved

                {}

                {} Build created on {}
                - Host machine: {}
                - Target machine: {}

                Enabled features: {:?}
            ",
            self.version(),
            self.description(),
            self.repository(),
            capitalize_first(self.profile()),
            self.built_time(),
            self.host(),
            self.target(),
            self.features()
        )
    }

    /// The library's name as a string.
    fn name(&self) -> &'static str {
        PKG_NAME
    }

    /// The library's package description as a string.
    fn description(&self) -> &'static str {
        PKG_DESCRIPTION
    }

    /// The library's repository URL as a string.
    fn repository(&self) -> &'static str {
        PKG_REPOSITORY
    }

    /// The full semantic version of this library
    /// as a string in the form of `major.minor.patch`.
    ///
    /// Example: `0.1.0`
    fn version(&self) -> &'static str {
        PKG_VERSION
    }

    /// The major version of this library as a string.
    ///
    /// Example: `0`
    fn version_major(&self) -> &'static str {
        PKG_VERSION_MAJOR
    }

    /// The minor version of this library as a string.
    ///
    /// Example: `1`
    fn version_minor(&self) -> &'static str {
        PKG_VERSION_MINOR
    }

    /// The patch version of this library as a string.
    ///
    /// Example: `0`
    fn version_patch(&self) -> &'static str {
        PKG_VERSION_PATCH
    }

    /// Whether or not this build was a debug build.
    ///
    /// All builds that are not release builds are considered debug builds.
    fn is_debug(&self) -> bool {
        PROFILE != "release"
    }

    /// Whether or not this build was a release build.
    fn is_release(&self) -> bool {
        PROFILE == "release"
    }

    /// The profile that this library has been compiled with.
    ///
    /// Example: `release`
    fn profile(&self) -> &'static str {
        PROFILE
    }

    /// The target triple that this library has been compiled for.
    ///
    /// Example: `x86_64-unknown-linux-gnu`
    fn target(&self) -> &'static str {
        TARGET
    }

    /// The host triple of the machine that compiled this library.
    ///
    /// Example: `x86_64-unknown-linux-gnu`
    fn host(&self) -> &'static str {
        HOST
    }

    /// The features that were enabled during compilation.
    fn features(&self) -> &'static [&'static str] {
        &FEATURES_LOWERCASE
    }

    /// The build time in RFC2822, UTC.
    ///
    /// Example: `Thu, 07 May 2020 21:18:02 GMT`
    fn built_time(&self) -> &'static str {
        BUILT_TIME_UTC
    }
}

/// Capitalizes the first letter of a string
fn capitalize_first(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

impl Logger for crate::app::App {}
impl Logger for crate::app::AppState {}

pub trait Logger {
    /// Logs a message to the App's main logger.
    fn log(&self, level: log::Level, message: &str) {
        log::log!(level, "{}", message);
    }

    /// Logs an error message to the App's main logger.
    fn error(&self, message: &str) {
        self.log(log::Level::Error, message);
    }

    /// Logs a warning message to the App's main logger.
    fn warn(&self, message: &str) {
        self.log(log::Level::Warn, message);
    }

    /// Logs an info message to the App's main logger.
    fn info(&self, message: &str) {
        self.log(log::Level::Info, message);
    }

    /// Logs a debug message to the App's main logger.
    fn debug(&self, message: &str) {
        self.log(log::Level::Debug, message);
    }

    /// Logs a trace message to the App's main logger.
    fn trace(&self, message: &str) {
        self.log(log::Level::Trace, message);
    }
}
