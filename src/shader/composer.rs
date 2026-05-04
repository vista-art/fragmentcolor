//! Input classification and value type for `Shader::new`.
//!
//! `ShaderInput` is the unified value passed to the shader builder. A single
//! string is classified into one of four `ShaderPart` variants by a heuristic
//! over its trimmed contents; an array of strings becomes one `ShaderPart` per
//! element, each classified independently. The resolver in `input.rs` walks the
//! parts, fetches/reads/looks up each one, deduplicates by source hash, and
//! concatenates the results into a single shader source.
//!
//! Detection order for `&str -> ShaderPart` (after trimming):
//!   1. `http://` or `https://` prefix                     -> Url
//!   2. Starts with `./`, `../`, `/`, `~/`                  -> Path
//!   3. Single-line, no whitespace, ends with a known shader extension -> Path
//!   4. Single-line, no whitespace, matches `[a-z][a-z0-9_]*/[a-z0-9_]+` -> Slug
//!   5. Otherwise                                            -> Source

use std::path::PathBuf;

const SHADER_PATH_EXTENSIONS: &[&str] = &[".wgsl", ".glsl", ".frag", ".vert"];
const SLUG_MAX_LEN: usize = 128;

#[derive(Debug, Clone)]
pub struct ShaderInput {
    pub(crate) parts: Vec<ShaderPart>,
}

#[derive(Debug, Clone)]
pub enum ShaderPart {
    Source(String),
    Slug(String),
    Url(String),
    Path(PathBuf),
}

impl ShaderInput {
    pub(crate) fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub(crate) fn parts(&self) -> &[ShaderPart] {
        &self.parts
    }
}

impl ShaderPart {
    /// Classify a single string into a `ShaderPart` using the documented heuristic.
    pub(crate) fn classify(raw: &str) -> Self {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return ShaderPart::Source(String::new());
        }

        if let Some(rest) = strip_http_prefix(trimmed) {
            let _ = rest;
            return ShaderPart::Url(trimmed.to_string());
        }

        let single_token = is_single_token(trimmed);

        // Path detection requires a single-line, whitespace-free string. Without
        // this guard, a multi-line WGSL source whose first non-whitespace bytes
        // are `//` (a comment) would match `starts_with('/')` and get classified
        // as a path. Real paths are single tokens.
        if single_token && starts_with_path_anchor(trimmed) {
            return ShaderPart::Path(PathBuf::from(trimmed));
        }

        if single_token && has_shader_extension(trimmed) {
            return ShaderPart::Path(PathBuf::from(trimmed));
        }

        if single_token && trimmed.len() < SLUG_MAX_LEN && is_slug(trimmed) {
            return ShaderPart::Slug(trimmed.to_string());
        }

        ShaderPart::Source(raw.to_string())
    }
}

fn strip_http_prefix(s: &str) -> Option<&str> {
    s.strip_prefix("http://")
        .or_else(|| s.strip_prefix("https://"))
}

fn starts_with_path_anchor(s: &str) -> bool {
    // `//` is the line-comment marker in WGSL/JS/Rust/Swift/Kotlin and never
    // a valid path prefix in any environment we target. Reject early so that
    // single-token strings beginning with a comment (rare but possible: a
    // minified shader, a copy-paste with a leading `//directive`) aren't
    // mis-classified as paths.
    if s.starts_with("//") {
        return false;
    }
    s.starts_with("./") || s.starts_with("../") || s.starts_with('/') || s.starts_with("~/")
}

fn has_shader_extension(s: &str) -> bool {
    SHADER_PATH_EXTENSIONS.iter().any(|ext| s.ends_with(ext))
}

fn is_single_token(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(|c| c.is_whitespace())
}

/// Slug grammar: `^[a-z][a-z0-9_]*\/[a-z0-9_]+$` with exactly one slash.
fn is_slug(s: &str) -> bool {
    let mut parts = s.split('/');
    let category = match parts.next() {
        Some(c) if !c.is_empty() => c,
        _ => return false,
    };
    let name = match parts.next() {
        Some(n) if !n.is_empty() => n,
        _ => return false,
    };
    if parts.next().is_some() {
        return false;
    }

    let mut chars = category.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl From<&str> for ShaderInput {
    fn from(s: &str) -> Self {
        Self {
            parts: vec![ShaderPart::classify(s)],
        }
    }
}

impl From<String> for ShaderInput {
    fn from(s: String) -> Self {
        Self {
            parts: vec![ShaderPart::classify(&s)],
        }
    }
}

impl From<&String> for ShaderInput {
    fn from(s: &String) -> Self {
        Self {
            parts: vec![ShaderPart::classify(s)],
        }
    }
}

impl From<Vec<String>> for ShaderInput {
    fn from(v: Vec<String>) -> Self {
        Self {
            parts: v.iter().map(|s| ShaderPart::classify(s)).collect(),
        }
    }
}

impl From<&[&str]> for ShaderInput {
    fn from(v: &[&str]) -> Self {
        Self {
            parts: v.iter().map(|s| ShaderPart::classify(s)).collect(),
        }
    }
}

impl From<&[String]> for ShaderInput {
    fn from(v: &[String]) -> Self {
        Self {
            parts: v.iter().map(|s| ShaderPart::classify(s)).collect(),
        }
    }
}

impl<const N: usize> From<[&str; N]> for ShaderInput {
    fn from(v: [&str; N]) -> Self {
        Self {
            parts: v.iter().map(|s| ShaderPart::classify(s)).collect(),
        }
    }
}

impl<const N: usize> From<[String; N]> for ShaderInput {
    fn from(v: [String; N]) -> Self {
        Self {
            parts: v.iter().map(|s| ShaderPart::classify(s)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_source(p: &ShaderPart, expected: &str) {
        match p {
            ShaderPart::Source(s) => assert_eq!(s, expected),
            other => panic!("expected Source, got {other:?}"),
        }
    }

    fn assert_slug(p: &ShaderPart, expected: &str) {
        match p {
            ShaderPart::Slug(s) => assert_eq!(s, expected),
            other => panic!("expected Slug, got {other:?}"),
        }
    }

    fn assert_url(p: &ShaderPart, expected: &str) {
        match p {
            ShaderPart::Url(u) => assert_eq!(u, expected),
            other => panic!("expected Url, got {other:?}"),
        }
    }

    fn assert_path(p: &ShaderPart, expected: &str) {
        match p {
            ShaderPart::Path(pb) => assert_eq!(pb.as_os_str(), expected),
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn classifier_url_https() {
        let p = ShaderPart::classify("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl");
        assert_url(&p, "https://fragmentcolor.org/shaders/sdf2d/circle.wgsl");
    }

    #[test]
    fn classifier_url_http() {
        let p = ShaderPart::classify("http://localhost:8000/shader.wgsl");
        assert_url(&p, "http://localhost:8000/shader.wgsl");
    }

    #[test]
    fn classifier_url_trims_whitespace() {
        let p = ShaderPart::classify("   https://example.com/x.wgsl  \n");
        assert_url(&p, "https://example.com/x.wgsl");
    }

    #[test]
    fn classifier_path_relative_dot() {
        let p = ShaderPart::classify("./shader.wgsl");
        assert_path(&p, "./shader.wgsl");
    }

    #[test]
    fn classifier_path_relative_dotdot() {
        let p = ShaderPart::classify("../shaders/main.wgsl");
        assert_path(&p, "../shaders/main.wgsl");
    }

    #[test]
    fn classifier_path_absolute() {
        let p = ShaderPart::classify("/usr/local/share/shader.wgsl");
        assert_path(&p, "/usr/local/share/shader.wgsl");
    }

    #[test]
    fn classifier_path_home() {
        let p = ShaderPart::classify("~/shaders/main.wgsl");
        assert_path(&p, "~/shaders/main.wgsl");
    }

    #[test]
    fn classifier_path_by_extension_wgsl() {
        let p = ShaderPart::classify("shader.wgsl");
        assert_path(&p, "shader.wgsl");
    }

    #[test]
    fn classifier_path_by_extension_glsl() {
        let p = ShaderPart::classify("shader.glsl");
        assert_path(&p, "shader.glsl");
    }

    #[test]
    fn classifier_path_by_extension_frag() {
        let p = ShaderPart::classify("shader.frag");
        assert_path(&p, "shader.frag");
    }

    #[test]
    fn classifier_path_by_extension_vert() {
        let p = ShaderPart::classify("shader.vert");
        assert_path(&p, "shader.vert");
    }

    // Regression: a multi-line shader source whose first non-whitespace bytes
    // are a `//` line comment must not be classified as a Path just because
    // the comment marker happens to start with `/`. This was the bug that broke
    // the front-page swirl shader: Shader.fetch fetched the URL successfully,
    // but the resulting WGSL body was re-classified as a path on the second
    // pass through Shader::new and resolution then tried to read it as a file.
    #[test]
    fn classifier_source_with_leading_line_comment_is_source() {
        let src = "// Fullscreen swirl palette demo (shader-only)\n\
                   // Ported from a Shadertoy-style fragment to WGSL\n\
                   \n\
                   struct VOut { @builtin(position) pos: vec4<f32> };\n";
        let p = ShaderPart::classify(src);
        assert_source(&p, src);
    }

    // Even a single-line `// foo` (no newline) shouldn't be a path — `//` is a
    // comment marker in every language we target, never a path prefix.
    #[test]
    fn classifier_single_line_starting_with_double_slash_is_source() {
        let p = ShaderPart::classify("//something");
        assert_source(&p, "//something");
    }

    #[test]
    fn classifier_slug_two_segments() {
        let p = ShaderPart::classify("sdf2d/circle");
        assert_slug(&p, "sdf2d/circle");
    }

    #[test]
    fn classifier_slug_with_underscores() {
        let p = ShaderPart::classify("post_fx/film_grain");
        assert_slug(&p, "post_fx/film_grain");
    }

    #[test]
    fn classifier_slug_with_digits_in_category() {
        let p = ShaderPart::classify("sdf2d/box");
        assert_slug(&p, "sdf2d/box");
    }

    #[test]
    fn classifier_slug_trimmed() {
        let p = ShaderPart::classify("  sdf2d/circle  \n");
        assert_slug(&p, "sdf2d/circle");
    }

    #[test]
    fn classifier_rejects_slug_with_uppercase() {
        let p = ShaderPart::classify("SDF/Circle");
        assert_source(&p, "SDF/Circle");
    }

    #[test]
    fn classifier_rejects_slug_with_three_segments() {
        let p = ShaderPart::classify("a/b/c");
        assert_source(&p, "a/b/c");
    }

    #[test]
    fn classifier_rejects_slug_starting_with_digit() {
        let p = ShaderPart::classify("2d/circle");
        assert_source(&p, "2d/circle");
    }

    #[test]
    fn classifier_rejects_slug_with_spaces() {
        let p = ShaderPart::classify("foo / bar");
        assert_source(&p, "foo / bar");
    }

    #[test]
    fn classifier_rejects_slug_too_long() {
        let long_name: String = std::iter::repeat_n('a', SLUG_MAX_LEN).collect();
        let candidate = format!("cat/{long_name}");
        let p = ShaderPart::classify(&candidate);
        match p {
            ShaderPart::Source(_) => {}
            other => panic!("expected Source for over-long slug, got {other:?}"),
        }
    }

    #[test]
    fn classifier_source_multiline() {
        let src =
            "@vertex\nfn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }";
        let p = ShaderPart::classify(src);
        assert_source(&p, src);
    }

    #[test]
    fn classifier_source_with_division_operator() {
        let src = "fn f(a: f32, b: f32) -> f32 { return a / b; }";
        let p = ShaderPart::classify(src);
        assert_source(&p, src);
    }

    #[test]
    fn classifier_source_starting_with_slug_shape_but_multiline() {
        // Ensure a multi-line string that happens to start with slug-looking text
        // is treated as Source, not Slug.
        let src = "foo/bar\n@vertex fn vs() {}";
        let p = ShaderPart::classify(src);
        assert_source(&p, src);
    }

    #[test]
    fn classifier_empty_string_is_empty_source() {
        let p = ShaderPart::classify("");
        assert_source(&p, "");
    }

    #[test]
    fn classifier_whitespace_only_is_empty_source() {
        let p = ShaderPart::classify("   \n\t");
        assert_source(&p, "");
    }

    #[test]
    fn from_str_single_part() {
        let input: ShaderInput = "sdf2d/circle".into();
        assert_eq!(input.parts.len(), 1);
        assert_slug(&input.parts[0], "sdf2d/circle");
    }

    #[test]
    fn from_string_single_part() {
        let s: String = "sdf2d/circle".to_string();
        let input: ShaderInput = s.into();
        assert_eq!(input.parts.len(), 1);
        assert_slug(&input.parts[0], "sdf2d/circle");
    }

    #[test]
    fn from_string_ref_single_part() {
        let s: String = "sdf2d/circle".to_string();
        let input: ShaderInput = (&s).into();
        assert_eq!(input.parts.len(), 1);
        assert_slug(&input.parts[0], "sdf2d/circle");
    }

    #[test]
    fn from_array_mixed() {
        let input: ShaderInput = [
            "sdf2d/circle",
            "https://example.com/x.wgsl",
            "fn f() {}",
            "./local.wgsl",
        ]
        .into();
        assert_eq!(input.parts.len(), 4);
        assert_slug(&input.parts[0], "sdf2d/circle");
        assert_url(&input.parts[1], "https://example.com/x.wgsl");
        assert_source(&input.parts[2], "fn f() {}");
        assert_path(&input.parts[3], "./local.wgsl");
    }

    #[test]
    fn from_vec_string_mixed() {
        let v: Vec<String> = vec!["sdf2d/circle".into(), "noise/simplex2".into()];
        let input: ShaderInput = v.into();
        assert_eq!(input.parts.len(), 2);
        assert_slug(&input.parts[0], "sdf2d/circle");
        assert_slug(&input.parts[1], "noise/simplex2");
    }

    #[test]
    fn from_slice_str_ref() {
        let v = ["sdf2d/circle", "noise/simplex2"];
        let input: ShaderInput = v.as_slice().into();
        assert_eq!(input.parts.len(), 2);
    }

    #[test]
    fn from_slice_string_ref() {
        let v: Vec<String> = vec!["sdf2d/circle".into(), "noise/simplex2".into()];
        let input: ShaderInput = v.as_slice().into();
        assert_eq!(input.parts.len(), 2);
    }
}
