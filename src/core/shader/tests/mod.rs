//! The shader tests: parsing, uniforms, storage and push constants,
//! mesh attachment, the queued setters and the creation checks.

use super::*;

#[test]
fn shader_tryfrom_str_variants() {
    let good = DEFAULT_SHADER;
    let s1 = Shader::try_from(good).unwrap();
    let s2 = Shader::try_from(good.to_string()).unwrap();
    let s3 = Shader::try_from(&good.to_string()).unwrap();

    // Touch API so they don't get optimized away
    let _ = s1.list_uniforms();
    let _ = s2.list_keys();
    let _ = s3.list_uniforms();

    let bad = "not wgsl";
    assert!(Shader::try_from(bad).is_err());
}

#[test]
fn shader_new_accepts_array_of_sources() {
    let helper = "fn util() -> f32 { return 1.0; }";
    let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(util()); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;

    let from_array = Shader::new([helper, main]).expect("from [&str; 2]");
    let from_vec: Vec<String> = vec![helper.to_string(), main.to_string()];
    let from_vec_call = Shader::new(from_vec).expect("from Vec<String>");
    let from_slice_str: Shader = Shader::new([helper, main].as_slice()).expect("from &[&str]");

    let _ = from_array.list_uniforms();
    let _ = from_vec_call.list_keys();
    let _ = from_slice_str.list_uniforms();
}

// Story: Rust sets and reads matrices as flat column-major lists, as every
// binding does, next to the nested arrays.
#[test]
fn set_and_get_flat_matrices() {
    let wgsl = r#"
@group(0) @binding(0) var<uniform> m4: mat4x4<f32>;
@group(0) @binding(1) var<uniform> m3: mat3x3<f32>;
@group(0) @binding(2) var<uniform> m2: mat2x2<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> {
    return m4[0] + vec4<f32>(m3[0], 0.0) + vec4<f32>(m2[0], 0.0, 0.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");

    shader.set("m4", [0.0; 16]).expect("flat mat4");
    let identity: Vec<f32> = (0..16)
        .map(|i| if i % 5 == 0 { 1.0 } else { 0.0 })
        .collect();
    shader.set("m4", identity.clone()).expect("mat4 from a Vec");
    assert_eq!(shader.get::<[f32; 16]>("m4").expect("get"), identity[..]);
    assert_eq!(shader.get::<Vec<f32>>("m4").expect("get"), identity);

    let three: [f32; 9] = std::array::from_fn(|i| i as f32);
    shader.set("m3", three).expect("flat mat3");
    assert_eq!(shader.get::<[f32; 9]>("m3").expect("get"), three);

    shader
        .set("m2", [1.0, 2.0, 3.0, 4.0])
        .expect("mat2 from four numbers");
    assert_eq!(
        shader.get::<[f32; 4]>("m2").expect("get"),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(shader.get::<Vec<u8>>("m2").expect("get").len(), 16);

    assert!(shader.set("m4", [0.0; 9]).is_err());
}

#[test]
fn shader_new_accepts_owned_string() {
    let owned: String = DEFAULT_SHADER.to_string();
    let _ = Shader::new(owned).expect("from String");
}

#[test]
fn shader_new_accepts_string_ref() {
    let owned: String = DEFAULT_SHADER.to_string();
    let _ = Shader::new(&owned).expect("from &String");
}

#[test]
fn validate_mesh_ok_and_errors() {
    // Shader expects pos: vec3 at location 0 and offset: vec2 at location 1
    let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");

    // Compatible mesh: pos=vec3, instance offset=vec2
    let m_ok = crate::Mesh::new();
    use crate::Vertex;
    m_ok.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);
    m_ok.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0f32, 0.0f32]));
    shader.validate_mesh(&m_ok).expect("compatible");

    // Missing attribute: no instance offset provided
    let m_missing = crate::Mesh::new();
    m_missing.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);
    let e = shader
        .validate_mesh(&m_missing)
        .expect_err("missing should err");
    match e {
        ShaderError::InvalidKey(_) => {}
        _ => panic!("unexpected error kind: {:?}", e),
    }

    // Type mismatch: instance offset present but wrong type (vec3 instead of vec2)
    let m_mismatch = crate::Mesh::new();
    m_mismatch.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);
    m_mismatch.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0f32, 0.0f32, 0.0f32]));
    let e2 = shader
        .validate_mesh(&m_mismatch)
        .expect_err("mismatch should err");
    assert_eq!(
        e2.to_string(),
        "Mesh attribute offset: expected vec2<f32>, got vec3<f32> at instance 0"
    );
}

const SHADER: &str = r#"
        struct VertexOutput {
            @builtin(position) coords: vec4<f32>,
        }

        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
            let x = f32(i32(in_vertex_index) - 1);
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
            return VertexOutput(vec4<f32>(x, y, 0.0, 1.0));
        }

        struct Circle {
            position: vec2<f32>,
            radius: f32,
            color: vec4<f32>,
        }

        @group(0) @binding(0)
        var<uniform> circle: Circle;

        @group(0) @binding(1) var<uniform> resolution: vec2<f32>;

        @fragment
        fn main(pixel: VertexOutput) -> @location(0) vec4<f32> {
            let uv = pixel.coords.xy / resolution;
            let circle_pos = circle.position.xy / resolution;
            let dist = distance(uv, circle_pos);
            let r = circle.radius / max(resolution.x, resolution.y);
            let circle_sdf = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
            return circle.color * circle_sdf;
        }
    "#;

#[test]
fn test_shader_should_parse_and_list_uniforms() {
    let shader = Shader::new(SHADER).unwrap();
    let mut uniforms = shader.list_uniforms();
    uniforms.sort();
    assert_eq!(uniforms, vec!["circle", "resolution"]);
}

#[test]
fn test_shader_should_parse_uniforms_and_list_keys() {
    let shader = Shader::new(SHADER).unwrap();
    let mut uniforms = shader.list_keys();
    uniforms.sort();
    assert_eq!(
        uniforms,
        vec![
            "circle",
            "circle.color",
            "circle.position",
            "circle.radius",
            "resolution"
        ]
    );
}

#[test]
fn test_shader_should_set_and_get_uniform() {
    let shader = Shader::new(SHADER).unwrap();
    shader.set("circle.position", [0.5, 0.5]).unwrap();
    shader.set("circle.radius", 0.25).unwrap();
    shader.set("circle.color", [1.0, 0.0, 0.0, 1.0]).unwrap();
    shader.set("resolution", [800.0, 600.0]).unwrap();

    let position: [f32; 2] = shader.get("circle.position").unwrap();
    let radius: f32 = shader.get("circle.radius").unwrap();
    let color: [f32; 4] = shader.get("circle.color").unwrap();
    let resolution: [f32; 2] = shader.get("resolution").unwrap();

    assert_eq!(position, [0.5, 0.5]);
    assert_eq!(radius, 0.25);
    assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(resolution, [800.0, 600.0]);
}

#[test]
fn test_shader_should_get_uniform_raw_bytes() {
    let shader = Shader::new(SHADER).unwrap();
    shader.set("circle.position", [0.5, 0.5]).unwrap();
    shader.set("circle.radius", 0.25).unwrap();
    shader.set("circle.color", [1.0, 0.0, 0.0, 1.0]).unwrap();
    shader.set("resolution", [800.0, 600.0]).unwrap();

    let storage = shader.object.storage.read();
    let position_bytes = storage.get_bytes("circle.position").unwrap();
    let radius_bytes = storage.get_bytes("circle.radius").unwrap();
    let color_bytes = storage.get_bytes("circle.color").unwrap();
    let resolution_bytes = storage.get_bytes("resolution").unwrap();

    assert_eq!(
        position_bytes,
        [
            0x00, 0x00, 0x00, 0x3f, //
            0x00, 0x00, 0x00, 0x3f,
        ]
    );
    assert_eq!(
        radius_bytes,
        [
            0x00, 0x00, 0x80, 0x3e //
        ]
    );
    assert_eq!(
        color_bytes,
        [
            0x00, 0x00, 0x80, 0x3f, //
            0x00, 0x00, 0x00, 0x00, //
            0x00, 0x00, 0x00, 0x00, //
            0x00, 0x00, 0x80, 0x3f
        ]
    );
    assert_eq!(
        resolution_bytes,
        [
            0x00, 0x00, 0x48, 0x44, //
            0x00, 0x00, 0x16, 0x44
        ]
    );
}

#[test]
fn test_invalid_shader_should_return_error() {
    let result = Shader::new("invalid shader");
    assert!(result.is_err());
}

#[test]
fn parses_texture_and_sampler_uniforms_meta() {
    let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let keys = shader.list_uniforms();
    assert!(keys.contains(&"tex".to_string()));
    assert!(keys.contains(&"samp".to_string()));

    // Verify meta
    let s = shader.object.storage.read();
    let (_, _, u_tex) = s.uniforms.get("tex").expect("tex uniform");
    match &u_tex.data {
        UniformData::Texture(meta) => {
            assert_eq!(meta.dim, crate::texture::TextureDim::D2);
            match meta.class {
                crate::texture::TextureClass::Sampled { .. } => {}
                _ => panic!("expected sampled image class"),
            }
        }
        _ => panic!("tex is not a texture uniform"),
    }
    let (_, _, u_samp) = s.uniforms.get("samp").expect("samp uniform");
    match &u_samp.data {
        UniformData::Sampler(info) => {
            // comparison defaults based on naga; any bool is acceptable
            let _ = info.comparison;
        }
        _ => panic!("samp is not a sampler uniform"),
    }
}

// Story: A simple read-only storage buffer with a single vec4 field should be parsed,
// listed among uniforms, and preserve its naga access flags.
#[test]
fn storage_read_only_parses_and_lists_fields() {
    let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> ssbo: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(0.5,0.5,0.5,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let mut names = shader.list_uniforms();
    names.sort();
    assert!(names.contains(&"ssbo".to_string()));

    // Verify meta
    let s = shader.object.storage.read();
    let (_, size, u) = s.uniforms.get("ssbo").expect("ssbo uniform");
    match u.data.clone() {
        UniformData::Storage(data) => {
            let entry = data.first().expect("storage data");
            let crate::StorageEntry {
                inner,
                span,
                access,
            } = entry;
            assert_eq!(*span, *size);
            assert!(access == &StorageAccess::Read);
            match inner.first().expect("inner shape") {
                UniformData::Struct(s) => {
                    assert_eq!(s.size, 16); // vec4<f32>
                    // Should have one field named 'a'
                    let mut seen_a = false;
                    for f in s.fields.iter() {
                        if f.name == "a" {
                            seen_a = true;
                            assert!(matches!(
                                f.ty.first().expect("field shape"),
                                UniformData::Vec4(_)
                            ));
                        }
                    }
                    assert!(seen_a);
                }
                _ => panic!("inner shape is not a struct"),
            }
        }
        _ => panic!("ssbo is not a storage buffer uniform"),
    }
}

// Story: A read-write storage buffer should carry write permission in its access flags.
#[test]
fn storage_read_write_meta_sets_writable() {
    let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read_write> sbuf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(0.2,0.4,0.6,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let s = shader.object.storage.read();
    let (_, _, u) = s.uniforms.get("sbuf").expect("sbuf uniform");
    match u.data.clone() {
        UniformData::Storage(data) => {
            let crate::StorageEntry { access, .. } = data.first().expect("storage data");
            assert!(access == &StorageAccess::ReadWrite);
        }
        _ => panic!("sbuf is not a storage buffer uniform"),
    }
}

// Story: Nested arrays and structs in a storage buffer should compute the right span.
#[test]
fn storage_nested_shapes_compute_span() {
    let wgsl = r#"
struct Inner { c: vec4<f32> };
struct Outer { a: vec4<f32>, arr: array<vec4<f32>, 2>, inner: Inner };
@group(0) @binding(0) var<storage, read> sto: Outer;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,0.,0.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let s = shader.object.storage.read();
    let (_, _, u) = s.uniforms.get("sto").expect("sto uniform");
    match &u.data {
        UniformData::Storage(data) => {
            let crate::StorageEntry { inner, span, .. } = data.first().expect("storage data");
            // a:16 + arr:2*16 + inner.c:16 = 64
            assert_eq!(span.clone(), 64);
            match inner.first().expect("inner shape") {
                UniformData::Struct(s) => assert_eq!(s.size, 64),
                _ => panic!("inner is not struct"),
            }
        }
        _ => panic!("sto is not a storage buffer"),
    }
}

// Story: Atomic fields in a storage buffer reflect with the layout of their scalar.
#[test]
fn storage_atomic_fields_reflect_as_scalars() {
    let wgsl = r#"
struct A { x: atomic<i32>, y: atomic<u32> };
@group(0) @binding(0) var<storage, read_write> a: A;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>, 4>;
@compute @workgroup_size(1)
fn cs_main() {
  atomicAdd(&a.x, 1);
  atomicAdd(&a.y, 1u);
  atomicAdd(&counts[0], 1u);
}
        "#;
    let shader = Shader::new(wgsl).expect("atomic storage parses");
    let s = shader.object.storage.read();

    let (_, _, a) = s.uniforms.get("a").expect("a uniform");
    let UniformData::Storage(data) = &a.data else {
        panic!("a is not a storage buffer");
    };
    let entry = data.first().expect("storage data");
    assert_eq!(entry.span, 8);
    assert_eq!(entry.access, StorageAccess::ReadWrite);
    let Some(UniformData::Struct(shape)) = entry.inner.first() else {
        panic!("a is not a struct");
    };
    let kinds: Vec<_> = shape.fields.iter().map(|field| &field.ty[0]).collect();
    assert!(matches!(
        kinds[..],
        [UniformData::Int(_), UniformData::UInt(_)]
    ));

    let (_, _, counts) = s.uniforms.get("counts").expect("counts uniform");
    let UniformData::Storage(data) = &counts.data else {
        panic!("counts is not a storage buffer");
    };
    assert_eq!(data.first().expect("storage data").span, 16);
}

// Story: Workgroup variables (unbound) should be ignored and not prevent parsing.
#[test]
fn workgroup_unbound_is_ignored() {
    let wgsl = r#"
var<workgroup> tile: array<vec4<f32>, 64>;
@compute @workgroup_size(1)
fn cs_main() { }
        "#;
    let _ = Shader::new(wgsl).expect("workgroup var ignored");
}

// Story: Push constants are parsed, listed, and settable just like other variables.
#[test]
fn push_constant_parsing_and_set() {
    let wgsl = r#"
struct PC { v: f32 };
var<immediate> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let mut names = shader.list_uniforms();
    names.sort();
    assert!(names.contains(&"pc".to_string()));
    shader.set("pc.v", 0.5f32).expect("set pc.v");
    let s = shader.object.storage.read();
    let bytes = s.get_bytes("pc").expect("pc bytes");
    assert_eq!(bytes.len(), 4);
    let v: f32 = bytemuck::cast_slice(&bytes[0..4])[0];
    assert_eq!(v, 0.5f32);
}

// Story: Multiple push-constant roots parse and expose both roots.
#[test]
fn push_constant_multiple_roots_listed() {
    let wgsl = r#"
struct A { v: f32 };
struct B { c: vec4<f32> };
var<immediate> a: A;
var<immediate> b: B;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let mut names = shader.list_uniforms();
    names.sort();
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
    // Set nested fields and verify CPU blobs updated
    shader.set("a.v", 1.0f32).expect("set a.v");
    shader.set("b.c", [0.0, 1.0, 0.0, 1.0]).expect("set b.c");
    let s = shader.object.storage.read();
    assert_eq!(s.get_bytes("a").unwrap().len(), 4);
    assert_eq!(s.get_bytes("b").unwrap().len(), 16);
}

// Story: Storage buffers can coexist with texture/sampler bindings.
#[test]
fn storage_and_texture_bindings_coexist() {
    let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> data: Buf;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    let mut names = shader.list_uniforms();
    names.sort();
    assert!(names.contains(&"data".to_string()));
    assert!(names.contains(&"tex".to_string()));
    assert!(names.contains(&"samp".to_string()));
}

// Story: Setting a storage-buffer field should update the CPU blob and be visible via get_bytes.
#[test]
fn storage_set_updates_cpu_blob_bytes() {
    let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> data: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    // Set a = (1,2,3,4)
    shader.set("data.a", [1.0, 2.0, 3.0, 4.0]).expect("set");
    let storage = shader.object.storage.read();
    let bytes = storage.get_bytes("data").expect("blob");
    assert_eq!(bytes.len(), 16);
    let expected: [u8; 16] = bytemuck::cast([1.0f32, 2.0, 3.0, 4.0]);
    assert_eq!(&bytes[0..16], &expected);
}

// Story: Arrays in storage buffers support element indexing using naga stride.
// - array<vec4<f32>, 4>: stride = 16 (std430); set element 2 and verify offset 2*16.
#[test]
fn storage_array_element_indexing() {
    let wgsl = r#"
struct Buf { arr: array<vec4<f32>, 4> };
@group(0) @binding(0) var<storage, read> data: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    // Set arr[2] = (9, 8, 7, 6)
    shader
        .set("data.arr[2]", [9.0, 8.0, 7.0, 6.0])
        .expect("set arr[2]");
    let s = shader.object.storage.read();
    let blob = s.get_bytes("data").expect("blob");
    assert_eq!(blob.len(), 4 * 16);
    // offset for index 2
    let start = 2 * 16;
    let expected: [u8; 16] = bytemuck::cast([9.0f32, 8.0, 7.0, 6.0]);
    assert_eq!(&blob[start..start + 16], &expected);
}

// Story: Arrays of structs in storage buffers can index into element then field.
#[test]
fn storage_array_of_structs_index_field() {
    let wgsl = r#"
struct Item { v: vec4<f32> };
struct Buf { items: array<Item, 3> };
@group(0) @binding(0) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    // Set items[1].v
    shader
        .set("buf.items[1].v", [0.1, 0.2, 0.3, 0.4])
        .expect("set items[1].v");
    let s = shader.object.storage.read();
    let blob = s.get_bytes("buf").expect("blob");
    // Expect stride >= 16; we only validate the written region matches expected bytes at index 1*stride
    // Find stride by reading through Uniform metadata
    let (_, _, u) = s.uniforms.get("buf").expect("buf uniform");
    let stride = match &u.data {
        UniformData::Storage(data) => {
            let crate::StorageEntry { inner, .. } = data.first().unwrap();
            match inner.first().expect("inner shape") {
                UniformData::Struct(s) => {
                    // fields[0] should be items: Array
                    match s.fields[0].ty.first().expect("field shape") {
                        UniformData::Array(items) => items.first().unwrap().stride,
                        _ => 0,
                    }
                }
                _ => 0,
            }
        }
        _ => 0,
    } as usize;
    assert!(stride >= 16);
    let start = stride; // index 1
    let expected: [u8; 16] = bytemuck::cast([0.1f32, 0.2, 0.3, 0.4]);
    assert_eq!(&blob[start..start + 16], &expected);
}

// Story: Top-level uniform struct with array supports element indexing in set() and get_bytes().
#[test]
fn uniform_array_element_indexing() {
    let wgsl = r#"
struct U { arr: array<vec4<f32>, 3> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    shader
        .set("u.arr[1]", [3.0, 2.0, 1.0, 0.0])
        .expect("set u.arr[1]");
    let s = shader.object.storage.read();
    // get_bytes with element should work via slow-path computation
    let bytes = s.get_bytes("u.arr[1]").expect("bytes");
    assert_eq!(bytes.len(), 16);
    let expected: [u8; 16] = bytemuck::cast([3.0f32, 2.0, 1.0, 0.0]);
    assert_eq!(bytes, &expected);
}

// Story: get_bytes on storage element slice returns only that element's bytes
#[test]
fn storage_get_bytes_element_slice() {
    let wgsl = r#"
struct Buf { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    shader
        .set("buf.arr[0]", [5.0, 6.0, 7.0, 8.0])
        .expect("set arr[0]");
    shader
        .set("buf.arr[1]", [1.0, 2.0, 3.0, 4.0])
        .expect("set arr[1]");
    let s = shader.object.storage.read();
    let e1 = s.get_bytes("buf.arr[1]").expect("bytes arr[1]");
    let expected: [u8; 16] = bytemuck::cast([1.0f32, 2.0, 3.0, 4.0]);
    assert_eq!(e1, &expected);
}

// Story: setting a field with a wrong type returns a TypeMismatch error.
#[test]
fn set_with_type_mismatch_errors() {
    let wgsl = SHADER;
    let shader = Shader::new(wgsl).expect("shader");
    // circle.radius expects a float
    let err = shader
        .set("circle.radius", [1.0f32, 2.0])
        .expect_err("type mismatch");
    match err {
        ShaderError::TypeMismatch(_) => {}
        _ => panic!("unexpected error: {:?}", err),
    }
}

// Story: getting a non-existent key returns UniformNotFound.
#[test]
fn get_uniform_not_found_errors() {
    let shader = Shader::new(SHADER).expect("shader");
    let err = shader.get::<f32>("does.not.exist").expect_err("not found");
    match err {
        ShaderError::UniformNotFound(_) => {}
        _ => panic!("unexpected error: {:?}", err),
    }
}

// Story: array index out of bounds triggers IndexOutOfBounds for uniform arrays.
#[test]
fn uniform_array_index_out_of_bounds_errors() {
    let wgsl = r#"
struct U { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
    let shader = Shader::new(wgsl).expect("shader");
    // index 2 is out of bounds for len 2 [0,1]
    let err = shader
        .set("u.arr[2]", [1.0, 2.0, 3.0, 4.0])
        .expect_err("oob");
    match err {
        ShaderError::IndexOutOfBounds { .. } => {}
        _ => panic!("unexpected error: {:?}", err),
    }
}

// Story: AST-driven mapping errors when a needed attribute is missing.
#[test]
fn ast_mapping_missing_attribute_errors() {
    let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,0.,1.,1.); }
            "#;

    let shader = crate::Shader::new(wgsl).expect("shader");

    let mesh = crate::Mesh::new();
    use crate::Vertex;
    mesh.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);

    let res = shader.add_mesh(&mesh);
    assert!(res.is_err());
    let s = format!("{}", res.unwrap_err());
    assert!(s.contains("Mesh attribute not found for shader input 'offset'"));
}

// Story: AST-driven mapping errors on type mismatch between shader input and mesh property format.
#[test]
fn ast_mapping_type_mismatch_errors() {
    let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,0.,1.); }
            "#;

    let shader = crate::Shader::new(wgsl).expect("shader");

    let mesh = crate::Mesh::new();
    use crate::{Vertex, VertexData};
    mesh.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);
    // Add instance with wrong-typed "offset" (vec3 instead of vec2)
    mesh.add_instance(Vertex::new([0.0, 0.0]).set("offset", VertexData::F32x3([0.0, 0.0, 0.0])));

    let res = shader.add_mesh(&mesh);
    assert!(res.is_err());
    let s = format!("{}", res.unwrap_err());
    assert_eq!(
        s,
        "Mesh attribute offset: expected vec2<f32>, got vec3<f32> at instance 0"
    );

    // Three plain numbers can't fill a vec2 either.
    let mesh = crate::Mesh::new();
    mesh.add_vertices([
        Vertex::new([-0.5, -0.5, 0.0]),
        Vertex::new([0.5, -0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
    ]);
    mesh.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0, 0.0, 0.0]));
    let s = shader.add_mesh(&mesh).expect_err("count").to_string();
    assert_eq!(
        s,
        "Mesh attribute offset: expected vec2<f32>, got 3 number(s) at instance 0"
    );
}

const TYPED_ID: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) @interpolate(flat) id: u32 };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.id = id;
  return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(f32(v.id)); }
"#;

const FLOAT_ID: &str = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: f32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos, id, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

fn mesh_with_ids(ids: &[f64]) -> crate::Mesh {
    let mesh = crate::Mesh::new();
    for (index, id) in ids.iter().enumerate() {
        let x = index as f32;
        mesh.add_vertex(crate::Vertex::new([x, 0.0]).set("id", *id));
    }
    mesh
}

// Story: a remove that comes while the same mesh attaches again waits for the
// attach, so the shader keeps the reads it records for the copy it still holds.
#[test]
fn a_remove_during_an_attach_keeps_the_reads() {
    let shader = Shader::new(FLOAT_ID).expect("shader");
    let mesh = mesh_with_ids(&[1.0, 2.0, 3.0]);
    shader.add_mesh(&mesh).expect("first attach");
    let remover = std::rc::Rc::new(std::cell::RefCell::new(None));
    let handle = remover.clone();
    let (removing, removed) = (shader.clone(), mesh.clone());
    crate::testing::race_hooks::once("declared", move || {
        let (sender, receiver) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            removing.remove_mesh(&removed);
            let _ = sender.send(());
        });
        let _ = receiver.recv_timeout(std::time::Duration::from_millis(200));
        *handle.borrow_mut() = Some(thread);
    });

    shader.add_mesh(&mesh).expect("second attach");
    if let Some(thread) = remover.borrow_mut().take() {
        thread.join().expect("the remover thread");
    }

    let held = shader.object.meshes.read().len();
    let reads = mesh.object.reads_of(shader.object.id);
    assert_eq!(held, 1, "one copy stays");
    assert!(
        reads.is_some_and(|reads| reads.vertex.contains_key("id")),
        "the copy it holds keeps its reads"
    );
}

// Story: validate_mesh is a dry run. It checks the mesh, and a later
// shader may still give its plain numbers another type.
#[test]
fn validate_mesh_commits_nothing() {
    let unsigned = Shader::new(TYPED_ID).expect("shader");
    let mesh = mesh_with_ids(&[1.0, 2.0, 3.0]);
    unsigned.validate_mesh(&mesh).expect("fits");
    assert!(mesh.object.types.read().vertex.is_empty());

    let float = Shader::new(FLOAT_ID).expect("shader");
    float.add_mesh(&mesh).expect("still untyped");
    assert_eq!(
        mesh.object.types.read().vertex.get("id"),
        Some(&wgpu::VertexFormat::Float32)
    );
}

// Story: a later vertex that lacks an attribute the shader reads fails the
// attach and names both, where it would otherwise draw with zeros.
#[test]
fn a_later_vertex_without_an_attribute_fails() {
    let shader = Shader::new(FLOAT_ID).expect("shader");
    let mesh = mesh_with_ids(&[1.0, 2.0]);
    mesh.add_vertex(crate::Vertex::new([2.0, 0.0]));
    let error = shader.add_mesh(&mesh).expect_err("vertex 2 has no id");
    assert_eq!(
        error.to_string(),
        "Mesh attribute id: expected f32, got no value at vertex 2"
    );
    assert!(shader.object.meshes.read().is_empty());

    // An attribute the shader doesn't read may come and go.
    let mesh = mesh_with_ids(&[1.0, 2.0]);
    mesh.add_vertex(
        crate::Vertex::new([2.0, 0.0])
            .set("id", 3.0)
            .set("extra", 1.0),
    );
    shader.add_mesh(&mesh).expect("extra is not read");
}

// Story: the same holds for instances.
#[test]
fn a_later_instance_without_an_attribute_fails() {
    let wgsl = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos + offset, 0.0, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    use crate::{Instance, Vertex};
    let shader = Shader::new(wgsl).expect("shader");
    let mesh = crate::Mesh::new();
    mesh.add_vertices([
        Vertex::new([0.0, 0.0]),
        Vertex::new([1.0, 0.0]),
        Vertex::new([0.0, 1.0]),
    ]);
    mesh.add_instance(Instance::new().set("offset", [0.0, 0.0]));
    mesh.add_instance(Instance::new().set("scale", 2.0));
    let error = shader
        .add_mesh(&mesh)
        .expect_err("instance 1 has no offset");
    assert_eq!(
        error.to_string(),
        "Mesh attribute offset: expected vec2<f32>, got no value at instance 1"
    );
}

// Story: an attach that fails leaves no type behind, so the next shader gets a clean try.
#[test]
fn failed_attach_leaves_no_types() {
    let unsigned = Shader::new(TYPED_ID).expect("shader");
    let mesh = mesh_with_ids(&[1.0, 2.0, -1.0]);
    let error = unsigned.add_mesh(&mesh).expect_err("negative u32");
    assert_eq!(
        error.to_string(),
        "Mesh attribute id: expected u32, got -1 at vertex 2"
    );
    assert!(mesh.object.types.read().vertex.is_empty());
    assert!(unsigned.object.meshes.read().is_empty());

    let float = Shader::new(FLOAT_ID).expect("shader");
    float.add_mesh(&mesh).expect("floats take -1");
    assert_eq!(float.object.meshes.read().len(), 1);
}

// Story: the @location members of a struct argument are vertex inputs like plain
// @location arguments, each under its own name, so a mesh property feeds each one.
#[test]
fn struct_members_take_a_mesh() {
    // Arrange
    let shader = Shader::new(
        r#"
struct Extra { @builtin(vertex_index) index: u32, @location(1) offset: vec2<f32> };
@vertex fn vs_main(@location(0) pos: vec2<f32>, extra: Extra) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos + extra.offset, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#,
    )
    .expect("shader");
    let mesh = crate::Mesh::new();
    mesh.add_vertices(
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
            .map(|corner| crate::Vertex::new(corner).set("offset", [0.0, 0.0])),
    );

    // Act
    let inputs = shader.object.reflect_vertex_inputs().expect("inputs");
    let attached = shader.add_mesh(&mesh);

    // Assert
    let names: Vec<(&str, u32)> = inputs
        .iter()
        .map(|input| (input.name.as_str(), input.location))
        .collect();
    assert_eq!(names, [("pos", 0), ("offset", 1)]);
    assert!(attached.is_ok(), "{attached:?}");
    assert_eq!(shader.object.meshes.read().len(), 1);
}

// Story: the shader Shader::from_vertex builds reads color at @location(1), whatever
// the color's place among the vertex's properties, so it fits a 16-attribute device.
#[test]
fn from_vertex_reads_color_at_location_one() {
    let vertex = (0..15).fold(crate::Vertex::new([0.0, 0.0]), |vertex, index| {
        vertex.set(&format!("p{index}"), 0.0)
    });
    let vertex = vertex.set("color", [1.0, 0.0, 0.0, 1.0]);

    let shader = Shader::from_vertex(&vertex).expect("shader");

    let inputs = shader.object.reflect_vertex_inputs().expect("inputs");
    let color = inputs.iter().find(|input| input.name == "color");
    assert_eq!(color.map(|input| input.location), Some(1));
}

// Story: the shader Shader::from_vertex builds reads a color of four plain numbers.
#[test]
fn from_vertex_reads_an_untyped_color() {
    let vertex = crate::Vertex::new([0.0, 0.0]).set("color", [1.0, 0.0, 0.0, 1.0]);
    let shader = Shader::from_vertex(&vertex).expect("shader");
    let inputs = shader.object.reflect_vertex_inputs().expect("inputs");
    assert!(
        inputs
            .iter()
            .any(|input| input.name == "color" && input.format == wgpu::VertexFormat::Float32x4)
    );

    let mesh = crate::Mesh::new();
    mesh.add_vertices(
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
            .map(|corner| crate::Vertex::new(corner).set("color", [1.0, 0.0, 0.0, 1.0])),
    );
    shader.add_mesh(&mesh).expect("color attaches");
}

// Story: non-blocking set() enqueues while storage is write-locked; flush_pending applies updates
#[test]
fn set_nonblocking_when_locked_then_flush_applies() {
    // Use the existing SHADER with uniforms: circle, resolution
    let shader = Shader::new(SHADER).unwrap();

    // Lock storage to simulate contention
    let _guard = shader.object.storage.write();

    // set() should not error and should enqueue (since write lock is held)
    shader
        .set("resolution", [1024.0f32, 768.0f32])
        .expect("enqueue ok");
    shader.set("circle.radius", 0.42f32).expect("enqueue ok");

    // Drop lock and flush pending
    drop(_guard);
    shader.object.flush_pending();

    // Verify values have been applied
    let res: [f32; 2] = shader.get("resolution").expect("resolution get");
    let r: f32 = shader.get("circle.radius").expect("radius get");
    assert_eq!(res, [1024.0, 768.0]);
    assert!((r - 0.42).abs() < 1e-6);
}

// Story: last-wins semantics — multiple enqueued writes to the same key result in the last value
#[test]
fn set_last_wins_queue_semantics() {
    let shader = Shader::new(SHADER).unwrap();
    let _guard = shader.object.storage.write();

    // Enqueue several updates while locked
    for i in 0..10 {
        shader.set("circle.radius", i as f32).expect("enqueue ok");
    }
    drop(_guard);
    shader.object.flush_pending();

    // Expect the last value (9.0)
    let r: f32 = shader.get("circle.radius").expect("radius");
    assert!((r - 9.0).abs() < 1e-6);
}

const TYPED: &str = r#"
struct Item { v: vec4<f32> };
struct Buf { items: array<Item, 3> };
struct Arr { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> count: u32;
@group(0) @binding(1) var<uniform> cell: vec2<i32>;
@group(0) @binding(2) var<uniform> rot: mat2x2<f32>;
@group(0) @binding(3) var<uniform> u: Arr;
@group(0) @binding(4) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> {
  let x = f32(count) + f32(cell.x) + rot[0].x + u.arr[1].x + buf.items[0].v.x;
  return vec4f(x, 0., 0., 1.);
}
    "#;

// Story: plain numbers reach integer uniforms, the way JS and Python send them.
#[test]
fn set_writes_whole_numbers_into_integer_uniforms() {
    let shader = Shader::new(TYPED).expect("shader");
    shader
        .object
        .set_input("count", UniformInput::Number(3.0))
        .expect("u32 from a number");
    shader
        .object
        .set_input("cell", UniformInput::Numbers(vec![2.0, -1.0]))
        .expect("vec2<i32> from numbers");
    assert_eq!(shader.get::<u32>("count").expect("count"), 3);
    assert_eq!(shader.get::<[i32; 2]>("cell").expect("cell"), [2, -1]);

    // Rust takes the same path: an i32 literal fits the u32 slot.
    shader.set("count", 7).expect("u32 from an i32 literal");
    assert_eq!(shader.get::<u32>("count").expect("count"), 7);

    let err = shader.set("count", -1).expect_err("negative u32");
    assert!(err.to_string().ends_with("count: expected u32, got -1"));
}

// Story: a mat2x2 uniform takes four numbers from any language.
#[test]
fn set_builds_a_mat2_from_four_numbers() {
    let shader = Shader::new(TYPED).expect("shader");
    let values = vec![1.0, 2.0, 3.0, 4.0];
    shader
        .object
        .set_input("rot", UniformInput::Numbers(values))
        .expect("mat2x2 from four numbers");
    let rot: [[f32; 2]; 2] = shader.get("rot").expect("rot");
    assert_eq!(rot, [[1.0, 2.0], [3.0, 4.0]]);
    shader
        .set("rot", [1.0, 0.0, 0.0, 1.0])
        .expect("mat2x2 from a vec4");
}

// Story: a value with the wrong length is rejected and the uniform keeps its value.
#[test]
fn set_rejects_wrong_lengths_and_keeps_the_value() {
    let shader = Shader::new(SHADER).expect("shader");
    shader.set("resolution", [800.0, 600.0]).expect("vec2");
    let three = UniformInput::Numbers(vec![1.0, 2.0, 3.0]);
    let err = shader
        .object
        .set_input("resolution", three)
        .expect_err("three numbers for a vec2");
    assert!(matches!(err, ShaderError::TypeMismatch(_)));
    assert!(
        err.to_string()
            .ends_with("resolution: expected vec2<f32>, got 3 number(s)")
    );
    let resolution: [f32; 2] = shader.get("resolution").expect("resolution");
    assert_eq!(resolution, [800.0, 600.0]);
}

// Story: indexed keys check the declared type, not only the byte count.
#[test]
fn indexed_keys_check_the_declared_type() {
    let shader = Shader::new(TYPED).expect("shader");
    // A vec4<u32> has the 16 bytes of a vec4<f32> but the wrong type.
    let uints = UniformData::UVec4(vec![1, 2, 3, 4]);
    let err = shader
        .object
        .set("buf.items[1].v", uints.clone())
        .expect_err("vec4<u32> into vec4<f32>");
    assert!(matches!(err, ShaderError::TypeMismatch(_)));
    assert!(shader.object.set("u.arr[1]", uints).is_err());

    // Loose numbers resolve to the declared vec4<f32>.
    shader.set("buf.items[1].v", [1, 2, 3, 4]).expect("numbers");
    let v: [f32; 4] = shader.get("buf.items[1].v").expect("indexed get");
    assert_eq!(v, [1.0, 2.0, 3.0, 4.0]);
    shader
        .set("u.arr[1]", [5.0, 6.0, 7.0, 8.0])
        .expect("array element");
    let e: [f32; 4] = shader.get("u.arr[1]").expect("indexed get");
    assert_eq!(e, [5.0, 6.0, 7.0, 8.0]);

    let storage = shader.object.storage.read();
    let shape = storage.shape("buf.items[2].v").expect("shape");
    assert!(matches!(shape, UniformData::Vec4(_)));
}

// Story: raw bytes fill a storage root, and are rejected anywhere else.
#[test]
fn set_takes_bytes_only_for_buffer_roots() {
    let shader = Shader::new(TYPED).expect("shader");
    let blob: Vec<u8> = (0..48).collect();
    shader
        .object
        .set_input("buf", UniformInput::Bytes(blob.clone()))
        .expect("bytes into a storage root");
    let storage = shader.object.storage.read();
    assert_eq!(storage.get_bytes("buf").expect("blob"), &blob[..]);
    drop(storage);

    let bytes = UniformInput::Bytes(vec![0; 8]);
    assert!(shader.object.set_input("cell", bytes.clone()).is_err());
    assert!(shader.object.set_input("buf.items[0].v", bytes).is_err());
}

// Story: an array of structs costs one key per field, not one per element, and its elements still resolve.
#[test]
fn array_elements_resolve_without_keys() {
    let shader = Shader::new(
        r#"
struct Item { v: vec4<f32>, w: f32 };
struct U { items: array<Item, 4096> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return u.items[0].v; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(u.items[1].w); }
"#,
    )
    .expect("shader");
    let mut keys = shader.list_keys();
    keys.sort();
    assert_eq!(keys, ["u", "u.items"]);
    shader.set("u.items[4095].w", 2.5).expect("last element");
    assert_eq!(shader.get::<f32>("u.items[4095].w").expect("get"), 2.5);
    let err = shader.set("u.items[4096].w", 1.0).unwrap_err();
    assert!(matches!(err, ShaderError::IndexOutOfBounds { .. }), "{err}");
    // The element sits at its stride: 4095 elements of 32 bytes, then `w` after `v`.
    let bytes: Vec<u8> = shader.get::<Vec<u8>>("u").expect("bytes");
    assert_eq!(bytes.len(), 4096 * 32);
    assert_eq!(
        &bytes[4095 * 32 + 16..4095 * 32 + 20],
        &2.5f32.to_le_bytes()
    );
}

// Story: a shader that needs a feature no device opens with fails at Shader::new.
#[test]
fn features_no_device_opens_fail_at_new() {
    let frag = |body: &str| {
        format!(
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(0.0); }}\n{body}"
        )
    };
    let sources = [
        frag("enable f16;\n@fragment fn fs_main() -> @location(0) vec4<f32> { let h: f16 = 1.0h; return vec4<f32>(f32(h)); }"),
        frag("@fragment fn fs_main(@builtin(primitive_index) index: u32) -> @location(0) vec4<f32> { return vec4<f32>(f32(index)); }"),
        frag("@group(0) @binding(0) var video: texture_external;\n@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(video, vec2<u32>(0u)); }"),
        "enable subgroups;\n@compute @workgroup_size(64) fn cs_main(@builtin(subgroup_size) size: u32) { }".to_string(),
    ];
    for source in sources {
        assert!(Shader::new(source.as_str()).is_err(), "{source}");
    }
    // Immediate data is one of the features a device may open with.
    let immediate = frag(
        "var<immediate> pc: vec4<f32>;\n@fragment fn fs_main() -> @location(0) vec4<f32> { return pc; }",
    );
    assert!(Shader::new(immediate.as_str()).is_ok());
}

// Story: an immediate root a uniform can't hold fails at Shader::new on every device,
// since a device without native immediates binds it as a uniform.
#[test]
fn immediates_follow_uniform_rules() {
    let shader = |declaration: &str| {
        format!(
            "{declaration}
@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(0.0); }}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(read()); }}"
        )
    };
    let array = shader("var<immediate> pc: array<f32, 4>;\nfn read() -> f32 { return pc[1]; }");
    let field = shader(
        "struct Weights { weights: array<f32, 4> };\nvar<immediate> pc: Weights;\nfn read() -> f32 { return pc.weights[1]; }",
    );
    for source in [array, field] {
        let err = Shader::new(source.as_str()).unwrap_err().to_string();
        assert!(
            err.contains("immediate roots pc must also follow the uniform layout rules"),
            "{err}"
        );
    }
    let vector = shader("var<immediate> pc: vec4<f32>;\nfn read() -> f32 { return pc.y; }");
    assert!(Shader::new(vector.as_str()).is_ok());
}

// Story: two vertex inputs with one name fail at Shader::new, since both would read
// one mesh attribute: members of two structs, or a member and a plain argument.
#[test]
fn vertex_inputs_with_one_name_fail() {
    let structs = r#"
struct VertexIn { @location(0) position: vec2<f32> };
struct InstanceIn { @location(1) position: vec2<f32> };
@vertex fn vs_main(v: VertexIn, i: InstanceIn) -> @builtin(position) vec4<f32> {
  return vec4<f32>(v.position + i.position, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    let mixed = r#"
struct VertexIn { @location(0) position: vec2<f32>, @location(1) uv: vec2<f32> };
@vertex fn vs_main(v: VertexIn, @location(3) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(v.position + v.uv + uv, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    for (source, first, second, name) in [(structs, 0, 1, "position"), (mixed, 1, 3, "uv")] {
        let err = Shader::new(source).unwrap_err().to_string();
        let expected = format!(
            "the vertex inputs at @location({first}) and @location({second}) are both named `{name}`"
        );
        assert!(err.contains(&expected), "{err}");
    }
}

// Story: a filtering sampler that samples a depth texture fails at Shader::new on every
// platform, since the web binds depth only with a comparison sampler or textureLoad.
#[test]
fn filtering_samplers_on_depth_fail() {
    let filtered = r#"
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var linear: sampler;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(textureSample(depth, linear, vec2<f32>(0.5))); }
"#;
    let err = Shader::new(filtered).unwrap_err().to_string();
    assert!(
        err.contains("`linear` filters, and it samples the depth texture `depth`"),
        "{err}"
    );
    let compared = r#"
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var shadow: sampler_comparison;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(textureSampleCompare(depth, shadow, vec2<f32>(0.5), 0.5)); }
"#;
    assert!(Shader::new(compared).is_ok());
}

/// A shader with one texture uniform, `image`.
fn build_shader() -> Shader {
    Shader::new(
        r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(image, vec2<i32>(0), 0); }
"#,
    )
    .expect("shader")
}

/// The texture id `image` holds.
fn read_image(object: &ShaderObject) -> u64 {
    match object.get_uniform("image").map(|uniform| uniform.data) {
        Ok(UniformData::Texture(meta)) => meta.id.id,
        other => panic!("image holds no texture: {other:?}"),
    }
}

// Story: every texture write and every map a setter makes waits for `texture_writes`,
// which a flush holds while it applies the queue, so no setter call lands in between.
#[test]
fn texture_writes_wait_for_the_lock() {
    use std::sync::atomic::AtomicUsize;
    let shader = build_shader();
    let object = &shader.object;
    let done = AtomicUsize::new(0);
    let count = |result: Result<(), ShaderError>| {
        done.fetch_add(1, Ordering::SeqCst);
        result
    };
    std::thread::scope(|scope| {
        let held = object.texture_writes.lock();
        let meta = crate::texture::TextureMeta::from(TextureId { id: 7 });
        let writers = [
            scope.spawn(|| count(object.set("image", UniformData::Texture(meta)))),
            scope.spawn(|| {
                let id = UniformInput::Texture(TextureId { id: 8 });
                count(object.set_input("image", id))
            }),
            scope.spawn(|| {
                let lazy = crate::TextureInput::from("tests/assets/missing.png");
                let source = crate::texture::TextureSource::try_from(lazy).expect("a source");
                count(object.set_map("image", source))
            }),
        ];
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
        while std::time::Instant::now() < deadline {
            let finished = done.load(Ordering::SeqCst);
            assert_eq!(finished, 0, "a texture write didn't wait for the lock");
            std::thread::yield_now();
        }
        drop(held);
        for writer in writers {
            writer.join().expect("a writer").expect("a write");
        }
    });
    assert_eq!(done.load(Ordering::SeqCst), 3);
}

// Story: a flush holds `texture_writes` while it applies the queue, so a texture
// setter can't slip a newer value into the queue between its read and its removal.
#[test]
fn a_flush_waits_for_texture_writes() {
    use std::sync::atomic::AtomicBool;
    let shader = build_shader();
    let object = &shader.object;
    {
        let _busy = object.storage.read();
        let id = UniformInput::Texture(TextureId { id: 5 });
        object.set_input("image", id).expect("queues");
    }
    assert!(!object.pending.is_empty(), "the set was applied at once");
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let held = object.texture_writes.lock();
        let flush = scope.spawn(|| {
            object.flush_pending();
            done.store(true, Ordering::SeqCst);
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
        while std::time::Instant::now() < deadline {
            let flushed = done.load(Ordering::SeqCst);
            assert!(!flushed, "the flush didn't wait for texture_writes");
            std::thread::yield_now();
        }
        drop(held);
        flush.join().expect("the flush");
    });
    assert!(
        object.pending.is_empty(),
        "the flush left the update queued"
    );
    assert_eq!(read_image(object), 5);
}

// Story: a set made after the flush took its key stays queued,
// and the next flush applies it, so the newer value wins over the one the flush took.
#[test]
fn a_set_during_the_flush_stays_queued() {
    let shader = Shader::new(SHADER).expect("a shader");
    let object = &shader.object;
    {
        let _busy = object.storage.read();
        object.set("circle.radius", 1.0f32).expect("queues");
    }
    object.apply_queue(|key| {
        if key == "circle.radius" {
            object.set("circle.radius", 2.0f32).expect("queues again");
        }
    });
    assert!(
        !object.pending.is_empty(),
        "the flush dropped the newer set"
    );
    object.flush_pending();
    assert!(object.pending.is_empty(), "the second flush left it queued");
    let radius: f32 = shader.get("circle.radius").expect("a radius");
    assert_eq!(radius, 2.0);
}

// Story: two globals on one slot fail at Shader::new, used by different entry points or not used at all.
#[test]
fn two_globals_on_one_slot_fail() {
    let split = r#"
@group(0) @binding(0) var<uniform> a: vec4<f32>;
@group(0) @binding(0) var<uniform> b: vec4<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return b; }
"#;
    let unused = r#"
@group(1) @binding(2) var<uniform> a: vec4<f32>;
@group(1) @binding(2) var tex: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    for (source, slot) in [
        (split, "@group(0) @binding(0)"),
        (unused, "@group(1) @binding(2)"),
    ] {
        let err = Shader::new(source).unwrap_err().to_string();
        assert!(err.contains(&format!("both use {slot}")), "{err}");
    }
    let parts = Shader::new([
        "@group(0) @binding(0) var<uniform> a: vec4<f32>;",
        "@group(0) @binding(0) var<uniform> b: vec4<f32>;\n@vertex fn vs_main() -> @builtin(position) vec4<f32> { return b; }\n@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
    ]);
    assert!(parts.is_err());
    // An immediate root never takes a binding slot.
    let immediate = r#"
var<immediate> pc: vec4<f32>;
@group(0) @binding(0) var<uniform> a: vec4<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a + pc; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    assert!(Shader::new(immediate).is_ok());
}

// Story: raw bytes of any other length than the root's leave the root as it was.
#[test]
fn set_refuses_bytes_that_miss_the_root_size() {
    let shader = Shader::new(TYPED).expect("shader");
    let blob: Vec<u8> = (0..48).collect();
    shader.set("buf", blob.as_slice()).expect("exact bytes");
    for length in [3, 47, 49, 96] {
        let err = shader.set("buf", vec![9u8; length]).unwrap_err();
        assert!(
            err.to_string()
                .ends_with(&format!("buf: expected 48 bytes, got {length}")),
            "{err}"
        );
    }
    let err = shader
        .object
        .set("buf", UniformData::Bytes(vec![9; 3]))
        .unwrap_err();
    assert!(
        err.to_string().contains("expected 48 bytes, got 3"),
        "{err}"
    );
    let storage = shader.object.storage.read();
    assert_eq!(storage.get_bytes("buf").expect("blob"), &blob[..]);
}

// Story: get on a struct root reads the bytes as they are now, not the reflected defaults.
#[test]
fn get_reads_struct_roots_from_current_bytes() {
    let shader = Shader::new(SHADER).expect("shader");
    shader.set("circle.radius", 0.25).expect("radius");
    let circle = shader.object.get_uniform_data("circle").expect("circle");
    let UniformData::Bytes(bytes) = circle else {
        panic!("struct roots read as bytes, got {circle:?}");
    };
    // `radius` sits at offset 8, after `position: vec2<f32>`.
    assert_eq!(&bytes[8..12], &0.25f32.to_le_bytes());
}

// Story: a loose value queued while the storage is locked resolves when it flushes.
#[test]
fn set_queues_loose_values_while_locked() {
    let shader = Shader::new(TYPED).expect("shader");
    let guard = shader.object.storage.write();
    let count = UniformInput::Number(5.0);
    shader.object.set_input("count", count).expect("queued");
    let cell = UniformInput::Numbers(vec![1.0, 2.0, 3.0]);
    shader
        .object
        .set_input("cell", cell)
        .expect("queued, checked at flush");
    drop(guard);

    shader.object.flush_pending();
    assert!(shader.object.pending.is_empty());
    assert_eq!(shader.get::<u32>("count").expect("count"), 5);
    assert_eq!(shader.get::<[i32; 2]>("cell").expect("cell"), [0, 0]);
}

// Story: a shader rendered on its own is freed with its pass when dropped.
#[test]
fn dropped_shader_frees_after_render() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([4u32, 4u32])
            .await
            .expect("texture target");
        let shader = Shader::default();
        let object = Arc::downgrade(&shader.object);
        let pass = Arc::downgrade(&shader.pass);

        renderer.render(&shader, &target).expect("render");
        drop(shader);

        assert!(object.upgrade().is_none(), "the shader leaked");
        assert!(pass.upgrade().is_none(), "the shader's pass leaked");
    });
}
