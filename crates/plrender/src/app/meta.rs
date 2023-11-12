include!(concat!(env!("OUT_DIR"), "/built.rs"));

/// Prints the build information to the standard output.
pub fn print_build_info() {
    print!("{}", build_info());
}

/// Returns the build information as a string.
pub fn build_info() -> String {
    format!(
        "
⭕  PLRender Version {}
    {}
    Pupil Labs GmbH - All rights reserved

    {}

    {} Build created on {}
    - Host machine: {}
    - Target machine: {}

    Enabled features: {:?}
",
        version(),
        description(),
        repository(),
        capitalize_first(profile()),
        built_time(),
        host(),
        target(),
        features()
    )
}

/// The library's name as a string.
pub fn name() -> &'static str {
    PKG_NAME
}

/// The library's package description as a string.
pub fn description() -> &'static str {
    PKG_DESCRIPTION
}

/// The library's repository URL as a string.
pub fn repository() -> &'static str {
    PKG_REPOSITORY
}

/// The full semantic version of this library
/// as a string in the form of `major.minor.patch`.
///
/// Example: `0.1.0`
pub fn version() -> &'static str {
    PKG_VERSION
}

/// The major version of this library as a string.
///
/// Example: `0`
pub fn version_major() -> &'static str {
    PKG_VERSION_MAJOR
}

/// The minor version of this library as a string.
///
/// Example: `1`
pub fn version_minor() -> &'static str {
    PKG_VERSION_MINOR
}

/// The patch version of this library as a string.
///
/// Example: `0`
pub fn version_patch() -> &'static str {
    PKG_VERSION_PATCH
}

/// Whether or not this build was a debug build.
///
/// All builds that are not release builds are considered debug builds.
pub fn is_debug() -> bool {
    PROFILE != "release"
}

/// Whether or not this build was a release build.
pub fn is_release() -> bool {
    PROFILE == "release"
}

/// The profile that this library has been compiled with.
///
/// Example: `release`
pub fn profile() -> &'static str {
    PROFILE
}

/// The target triple that this library has been compiled for.
///
/// Example: `x86_64-unknown-linux-gnu`
pub fn target() -> &'static str {
    TARGET
}

/// The host triple of the machine that compiled this library.
///
/// Example: `x86_64-unknown-linux-gnu`
pub fn host() -> &'static str {
    HOST
}

/// The features that were enabled during compilation.
pub fn features() -> &'static [&'static str] {
    &FEATURES_LOWERCASE
}

/// The build time in RFC2822, UTC.
///
/// Example: `Thu, 07 May 2020 21:18:02 GMT`
pub fn built_time() -> &'static str {
    BUILT_TIME_UTC
}

/// Capitalizes the first letter of a string
fn capitalize_first(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
