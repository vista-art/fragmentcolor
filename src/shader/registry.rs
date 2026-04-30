//! Process-wide shader registry base URL with a thread-local override stack
//! for test isolation.
//!
//! `Shader::set_registry(base)` overrides the default `https://fragmentcolor.org/shaders/`.
//! `slug_to_url("sdf2d/circle")` -> `<registry-base>/sdf2d/circle.wgsl`.

use parking_lot::RwLock;
use std::cell::RefCell;
use std::sync::LazyLock;

const DEFAULT_REGISTRY: &str = "https://fragmentcolor.org/shaders/";

static REGISTRY: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(DEFAULT_REGISTRY.to_string()));

thread_local! {
    static REGISTRY_OVERRIDE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Override the registry base URL used to resolve slugs (e.g. `sdf2d/circle`).
///
/// The base may end with or without a trailing slash; both are normalised at lookup time.
/// This setting is process-wide; tests should use `with_registry` for isolation.
pub(crate) fn set_registry(base_url: &str) {
    *REGISTRY.write() = base_url.to_string();
}

/// Return the active registry base, preferring any thread-local override.
pub(crate) fn registry_base() -> String {
    REGISTRY_OVERRIDE.with(|stack| stack.borrow().last().cloned()).unwrap_or_else(|| REGISTRY.read().clone())
}

/// Convert a slug like `sdf2d/circle` into a URL `<base>/sdf2d/circle.wgsl`.
pub(crate) fn slug_to_url(slug: &str) -> String {
    let base = registry_base();
    let trimmed_base = base.trim_end_matches('/');
    let trimmed_slug = slug.trim_start_matches('/');
    format!("{trimmed_base}/{trimmed_slug}.wgsl")
}

/// Run a closure with a thread-local registry override; restores the previous value on drop.
/// Used in tests to avoid races on the process-wide global.
#[cfg(test)]
pub(crate) fn with_registry<F, R>(base: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    REGISTRY_OVERRIDE.with(|stack| stack.borrow_mut().push(base.to_string()));
    let result = f();
    REGISTRY_OVERRIDE.with(|stack| {
        stack.borrow_mut().pop();
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_to_url_default_base() {
        with_registry("https://fragmentcolor.org/shaders/", || {
            assert_eq!(
                slug_to_url("sdf2d/circle"),
                "https://fragmentcolor.org/shaders/sdf2d/circle.wgsl"
            );
        });
    }

    #[test]
    fn slug_to_url_strips_trailing_slash() {
        with_registry("https://example.com/shaders/", || {
            assert_eq!(
                slug_to_url("sdf2d/circle"),
                "https://example.com/shaders/sdf2d/circle.wgsl"
            );
        });
    }

    #[test]
    fn slug_to_url_handles_no_trailing_slash() {
        with_registry("https://example.com/shaders", || {
            assert_eq!(
                slug_to_url("noise/simplex2"),
                "https://example.com/shaders/noise/simplex2.wgsl"
            );
        });
    }

    #[test]
    fn slug_to_url_strips_leading_slash_on_slug() {
        with_registry("https://example.com/shaders/", || {
            assert_eq!(
                slug_to_url("/sdf2d/circle"),
                "https://example.com/shaders/sdf2d/circle.wgsl"
            );
        });
    }

    #[test]
    fn registry_override_stacks_and_restores() {
        // Verify only override-stack semantics; do not depend on the global REGISTRY,
        // which other tests may mutate in parallel (the override stack is thread-local).
        with_registry("https://outer.example.com/", || {
            assert_eq!(registry_base(), "https://outer.example.com/");
            with_registry("https://inner.example.com/", || {
                assert_eq!(registry_base(), "https://inner.example.com/");
            });
            assert_eq!(registry_base(), "https://outer.example.com/");
        });
    }

    #[test]
    fn set_registry_writes_to_global() {
        // Snapshot and restore the global so this test plays nicely under cargo's
        // default parallel runner. The override stack is thread-local, so it does
        // not interfere with other threads running this test.
        let snapshot = REGISTRY.read().clone();
        set_registry("https://persist.example.com/");
        assert_eq!(REGISTRY.read().clone(), "https://persist.example.com/");
        set_registry(&snapshot);
    }
}
