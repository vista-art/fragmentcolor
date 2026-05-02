use super::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER, ShaderInput, ShaderPart};
use crate::ShaderObject;
use crate::shader::error::ShaderError;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
#[cfg(not(wasm))]
use std::path::PathBuf;
use std::sync::Arc;

/// Resolve a `ShaderInput` into a single `ShaderObject`:
/// fetch URLs / read paths / look up slugs, dedup by source hash, concat in order,
/// then dispatch to `ShaderObject::wgsl` (or the GLSL path for a single GLSL part).
pub(super) fn resolve(input: ShaderInput) -> Result<Arc<ShaderObject>, ShaderError> {
    if input.is_empty() {
        return Ok(Arc::new(ShaderObject::default()));
    }

    let resolved: Vec<Resolved> = input
        .parts()
        .iter()
        .map(resolve_part)
        .collect::<Result<_, _>>()?;

    let non_empty: Vec<&Resolved> = resolved.iter().filter(|r| !r.body.is_empty()).collect();

    if non_empty.is_empty() {
        return Ok(Arc::new(ShaderObject::default()));
    }

    let glsl_kind: Option<GlslKind> = non_empty.iter().find_map(|r| r.glsl);
    if let Some(kind) = glsl_kind {
        if non_empty.len() > 1 {
            return Err(ShaderError::ParseError(
                "GLSL composition not supported: pass exactly one .glsl/.frag/.vert part \
                 (mixing WGSL with GLSL is not allowed)"
                    .into(),
            ));
        }
        let only = non_empty[0];
        let object = match kind {
            GlslKind::Vertex => ShaderObject::glsl(&only.body, DEFAULT_FRAGMENT_SHADER)?,
            GlslKind::Fragment => ShaderObject::glsl(DEFAULT_VERTEX_SHADER, &only.body)?,
        };
        return Ok(Arc::new(object));
    }

    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    let mut bodies: Vec<&str> = Vec::with_capacity(non_empty.len());
    for r in &non_empty {
        let mut hasher = Sha256::new();
        hasher.update(r.body.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        if seen.insert(hash) {
            bodies.push(r.body.as_str());
        }
    }

    let merged = bodies.join("\n\n");
    Ok(Arc::new(ShaderObject::wgsl(&merged)?))
}

#[cfg(test)]
fn load_shader(source: &str) -> Result<Arc<ShaderObject>, ShaderError> {
    resolve(ShaderInput::from(source))
}

#[derive(Clone, Copy)]
enum GlslKind {
    Vertex,
    Fragment,
}

struct Resolved {
    body: String,
    glsl: Option<GlslKind>,
}

fn resolve_part(part: &ShaderPart) -> Result<Resolved, ShaderError> {
    match part {
        ShaderPart::Source(s) => Ok(Resolved {
            body: s.clone(),
            glsl: None,
        }),
        ShaderPart::Path(p) => read_path(p),
        ShaderPart::Url(u) => fetch_url(u),
        ShaderPart::Slug(slug) => {
            // If the slug's category was compiled in (via a `shaders-<cat>`
            // feature), serve the embedded copy and skip the network
            // entirely. Otherwise fall back to fetching the URL form.
            if let Some(body) = crate::shader::embedded::lookup(slug) {
                return Ok(Resolved {
                    body: body.to_string(),
                    glsl: None,
                });
            }
            let url = crate::shader::registry::slug_to_url(slug);
            fetch_url(&url)
        }
    }
}

fn read_path(path: &Path) -> Result<Resolved, ShaderError> {
    let body = std::fs::read_to_string(path)?;
    Ok(Resolved {
        body,
        glsl: glsl_kind_from_extension(path),
    })
}

#[cfg(not(wasm))]
fn fetch_url(url: &str) -> Result<Resolved, ShaderError> {
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;
    Ok(Resolved {
        body,
        glsl: glsl_kind_from_url(url),
    })
}

#[cfg(wasm)]
fn fetch_url(_url: &str) -> Result<Resolved, ShaderError> {
    Err(ShaderError::Error(
        "HTTP requests in the constructor are not supported in WASM. \
         Use `await Shader.fetch(input)` to compose shaders from URLs or registry slugs."
            .into(),
    ))
}

fn glsl_kind_from_extension(path: &Path) -> Option<GlslKind> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    match ext {
        "vert" => Some(GlslKind::Vertex),
        "frag" | "glsl" => Some(GlslKind::Fragment),
        _ => None,
    }
}

#[cfg(not(wasm))]
fn glsl_kind_from_url(url: &str) -> Option<GlslKind> {
    let path = url.split('?').next().unwrap_or(url);
    let pb = PathBuf::from(path);
    glsl_kind_from_extension(&pb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::registry::with_registry;

    // Story: Load minimal WGSL from a file path with .wgsl extension.
    #[test]
    fn loads_minimal_wgsl_from_file() {
        let wgsl = r#"
@group(0) @binding(0) var<uniform> u: vec4<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i) - 1);
  let y = f32(i32(i & 1u) * 2 - 1);
  return vec4<f32>(x, y, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0, 1.0, 1.0, 1.0); }
        "#;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("min.wgsl");
        std::fs::write(&path, wgsl).expect("write");
        let res = load_shader(path.to_str().unwrap());
        assert!(res.is_ok());
    }

    // Story: Invalid GLSL produces error
    #[test]
    fn glsl_file_without_feature_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("m.frag");
        std::fs::write(&path, "void main() {}").expect("write");
        let res = load_shader(path.to_str().unwrap());

        // With GLSL enabled, invalid GLSL should still error (validation error path)
        assert!(res.is_err());
    }

    // Story: A short non-WGSL source returns a naga WGSL parse error.
    #[test]
    fn short_source_string_rejected() {
        let err = load_shader("x").expect_err("invalid short source");
        match err {
            ShaderError::WgslParseError(_) | ShaderError::ParseError(_) => {}
            other => panic!("unexpected error kind: {other:?}"),
        }
    }

    // Story: Multiple WGSL parts concatenate cleanly.
    #[test]
    fn resolves_multiple_wgsl_parts() {
        let helper = "fn util() -> f32 { return 1.0; }";
        let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(util()); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;
        let input = ShaderInput::from([helper, main]);
        let res = resolve(input);
        assert!(res.is_ok(), "{res:?}");
    }

    // Story: Identical parts are deduplicated; the same helper included twice still validates.
    #[test]
    fn dedups_identical_parts() {
        let helper = "fn util() -> f32 { return 1.0; }";
        let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(util()); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;
        let input = ShaderInput::from([helper, helper, main]);
        let res = resolve(input);
        assert!(res.is_ok(), "{res:?}");
    }

    // Story: Empty parts are dropped after resolution.
    #[test]
    fn drops_empty_source_parts() {
        let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;
        let input = ShaderInput::from(["", main, "   "]);
        let res = resolve(input);
        assert!(res.is_ok(), "{res:?}");
    }

    // Story: A fully-empty input falls back to the default shader.
    #[test]
    fn empty_input_falls_back_to_default() {
        let input = ShaderInput::from("");
        let res = resolve(input).expect("default fallback");
        let _ = res; // construction succeeded
    }

    // Story: Mixing GLSL paths with other parts is rejected.
    #[test]
    fn rejects_mixed_glsl_and_wgsl_composition() {
        let dir = tempfile::tempdir().expect("tempdir");
        let frag = dir.path().join("m.frag");
        std::fs::write(
            &frag,
            "#version 450\nlayout(location=0) out vec4 c; void main() { c = vec4(1.0); }",
        )
        .expect("write");
        let frag_path = frag.to_str().unwrap().to_string();

        let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#
        .to_string();

        let input = ShaderInput::from(vec![frag_path, main]);
        let err = resolve(input).expect_err("must reject mixed GLSL+WGSL");
        match err {
            ShaderError::ParseError(_) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    // Story: A slug resolves through the registry to a fetchable URL.
    // We can't hit the network in unit tests, so we just verify that the override
    // is consulted (the resolver tries to fetch the override URL and returns a
    // network error, not a classification error).
    //
    // Uses `unknown_category/no_such_shader` — a slug shape that cannot match
    // any embedded entry regardless of which `shaders-*` features are on, so
    // the resolver always falls through to fetching the URL form.
    #[test]
    fn slug_uses_registry_override() {
        with_registry("http://127.0.0.1:1/", || {
            let input = ShaderInput::from("unknown_category/no_such_shader");
            let err = resolve(input).expect_err("expected fetch failure");
            match err {
                #[cfg(not(wasm))]
                ShaderError::RequestError(_) | ShaderError::FileNotFound(_) => {}
                #[cfg(wasm)]
                ShaderError::Error(_) => {}
                other => panic!("unexpected: {other:?}"),
            }
        });
    }

    // Story: Without any `shaders-*` feature enabled, the embedded lookup must
    // return None for every slug — the resolver always falls back to URL fetch.
    #[test]
    fn embedded_lookup_misses_when_no_feature_enabled() {
        // Lookups that would match if features were on:
        for slug in ["postfx/vignette", "noise/simplex2", "sdf2d/circle"] {
            // We can't directly assert "no feature" at runtime, but if every
            // category feature is off in this build, the lookup must be None.
            // Tests with features on live in `embedded_lookup_*_feature_*`.
            let got = crate::shader::embedded::lookup(slug);
            #[cfg(not(any(
                feature = "shaders-postfx",
                feature = "shaders-noise",
                feature = "shaders-sdf2d",
            )))]
            assert!(got.is_none(), "expected None for {slug}, got Some");
            // When at least one of these features IS on, the assertion is
            // tautological for that slug; skip it. Keep the loop running so the
            // test compiles in either configuration.
            let _ = got;
        }
    }

    // Story: When `shaders-postfx` is enabled, the embedded lookup returns the
    // helper source verbatim and the resolver short-circuits the URL fetch —
    // verified by pointing the registry at an unreachable URL and confirming
    // a slug-only resolve still succeeds.
    #[cfg(feature = "shaders-postfx")]
    #[test]
    fn embedded_postfx_serves_without_network() {
        // The lookup itself returns Some.
        let body = crate::shader::embedded::lookup("postfx/vignette")
            .expect("expected embedded postfx/vignette");
        assert!(
            body.contains("fn vignette("),
            "embedded body looks wrong: {body}"
        );

        // Compose with a small main shader and resolve through the full pipeline.
        // Registry override points nowhere — if the resolver tried to fetch
        // anything, this would error with a network failure instead.
        let main = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
  var out: VOut; out.pos = vec4<f32>(p[i], 0., 1.); out.uv = uv[i]; return out;
}
@fragment fn fs_main(in: VOut) -> @location(0) vec4<f32> {
  let v = vignette(in.uv, 0.5, 0.3);
  return vec4<f32>(v, v, v, 1.0);
}
"#;
        with_registry("http://127.0.0.1:1/", || {
            let input = ShaderInput::from(["postfx/vignette", main].as_slice());
            let res = resolve(input);
            assert!(res.is_ok(), "expected embedded short-circuit, got {res:?}");
        });
    }

    // Story: A slug whose category feature is NOT enabled still falls through
    // to URL fetch, even when other category features are on.
    #[cfg(all(feature = "shaders-postfx", not(feature = "shaders-sdf")))]
    #[test]
    fn embedded_other_categories_still_miss() {
        let got = crate::shader::embedded::lookup("sdf/sphere");
        assert!(
            got.is_none(),
            "sdf shouldn't be embedded without its feature"
        );
    }
}
