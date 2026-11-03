#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Instance, Mesh, Pass, Renderer, Shader, Target, Vertex};

/// Green when `id` arrives as the u32 16777217 and `cell` as the vec2<i32> (-1, 2), red otherwise.
/// An f32 transport would round 2^24 + 1 and draw red. The test runs in the vertex stage,
/// so every pixel is exactly 0 or 255 and the target's color space can't blur it.
const TYPED: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) ok: u32,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) id: u32,
    @location(2) cell: vec2<i32>,
) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(position, 0.0, 1.0);
    out.ok = select(0u, 1u, id == 16777217u && all(cell == vec2<i32>(-1, 2)));
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    if (v.ok == 1u) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

/// Green when `id` arrives as the f32 2.5, red otherwise.
const FLOAT: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) ok: u32,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) id: f32) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(position, 0.0, 1.0);
    out.ok = select(0u, 1u, id == 2.5);
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    if (v.ok == 1u) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

/// Green when `weight` arrives as the u32 3, red otherwise.
const WEIGHT: &str = r#"
struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) ok: u32,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) weight: u32) -> VOut {
    var out: VOut;
    out.pos = vec4<f32>(position, 0.0, 1.0);
    out.ok = select(0u, 1u, weight == 3u);
    return out;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    if (v.ok == 1u) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

const FULLSCREEN: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
const GREEN: [u8; 4] = [0, 255, 0, 255];

/// Renders `shader` into a fresh 8x8 target and returns its pixels.
async fn draw(renderer: &Renderer, shader: &Shader) -> Vec<u8> {
    let target = renderer
        .create_texture_target([8, 8])
        .await
        .expect("create_texture_target failed");
    renderer
        .render(&Pass::from_shader("vertex types", shader), &target)
        .expect("render failed");
    target.get_image().await.expect("read back")
}

fn assert_green(image: &[u8]) {
    assert_eq!(image.len(), 8 * 8 * 4, "image size");
    for (index, pixel) in image.chunks(4).enumerate() {
        assert_eq!(pixel, GREEN, "pixel {index}");
    }
}

// Story: plain numbers on a vertex take the integer types the shader declares, exactly.
#[test]
fn plain_numbers_take_the_integer_types_the_shader_declares() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let shader = Shader::new(TYPED).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| {
            Vertex::new(position)
                .set("id", 16_777_217)
                .set("cell", [-1, 2])
        }));

        shader
            .add_mesh(&mesh)
            .expect("plain numbers fit u32 and vec2<i32>");

        assert_green(&draw(&renderer, &shader).await);
    });
}

// Story: an instance's plain numbers take the shader's types the same way.
#[test]
fn instance_numbers_take_the_shader_type() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let shader = Shader::new(TYPED).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("cell", [-1, 2])));
        mesh.add_instance(Instance::new().set("id", 16_777_217));

        shader.add_mesh(&mesh).expect("plain numbers fit u32");

        assert_green(&draw(&renderer, &shader).await);
    });
}

// Story: vertices added after the attach pack in the types the attach recorded.
#[test]
fn vertices_added_after_the_attach_convert_too() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let shader = Shader::new(TYPED).expect("shader failed");
        let mesh = Mesh::new();
        let point = Vertex::new([0.0, 0.0])
            .set("id", 16_777_217)
            .set("cell", [-1, 2]);
        mesh.add_vertices([point.clone(), point.clone(), point]);
        shader.add_mesh(&mesh).expect("attach");

        mesh.add_vertices(FULLSCREEN.map(|position| {
            Vertex::new(position)
                .set("id", 16_777_217)
                .set("cell", [-1, 2])
        }));

        assert_green(&draw(&renderer, &shader).await);
    });
}

// Story: a vertex or an instance added after the attach holds every attribute the
// shader reads, or render fails naming the attribute, where the GPU read zeros.
#[test]
fn values_added_after_the_attach_hold_what_the_shader_reads() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(TYPED).expect("shader failed");
        let vertices = Mesh::new();
        vertices.add_vertices(FULLSCREEN.map(|position| {
            Vertex::new(position)
                .set("id", 16_777_217)
                .set("cell", [-1, 2])
        }));
        shader.add_mesh(&vertices).expect("attach");
        vertices.add_vertices(
            [[0.5, 0.5], [0.6, 0.5], [0.5, 0.6]]
                .map(|position| Vertex::new(position).set("id", 16_777_217)),
        );
        let instances = Mesh::new();
        instances
            .add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("cell", [-1, 2])));
        instances.add_instance(Instance::new().set("id", 16_777_217));
        let other = Shader::new(TYPED).expect("shader failed");
        other.add_mesh(&instances).expect("attach");
        instances.add_instance(Instance::new());

        let vertex = renderer.render(&shader, &target);
        let instance = renderer.render(&other, &target);

        let message = |result: Result<(), fragmentcolor::RendererError>| match result {
            Err(fragmentcolor::RendererError::MeshError(error)) => error.to_string(),
            other => panic!("expected a mesh error, got {other:?}"),
        };
        assert_eq!(
            message(vertex),
            "Invalid vertex: cell: expected vec2<i32>, got no value at vertex 3"
        );
        assert_eq!(
            message(instance),
            "Invalid instance: id: expected u32, got no value at instance 1"
        );
    });
}

/// Reads `uv` from each vertex beside the position.
const UV: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position + uv * 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
"#;

/// Reads the position alone.
const PLAIN: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
"#;

// Story: a mesh holds what the shaders attached to it read, and a shader that lets
// the mesh go takes its attributes along, so a vertex added later may leave them out.
#[test]
fn a_removed_shader_takes_its_attributes_along() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let reader = Shader::new(UV).expect("shader failed");
        let plain = Shader::new(PLAIN).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("uv", [0.0, 0.0])));
        reader.add_mesh(&mesh).expect("attach the uv reader");
        plain.add_mesh(&mesh).expect("attach the plain shader");
        mesh.add_vertices([[0.5, 0.5], [0.6, 0.5], [0.5, 0.6]].map(Vertex::new));

        let held = renderer.render(&plain, &target);
        reader.remove_mesh(&mesh);
        let released = renderer.render(&plain, &target);

        match held {
            Err(fragmentcolor::RendererError::MeshError(error)) => assert_eq!(
                error.to_string(),
                "Invalid vertex: uv: expected vec2<f32>, got no value at vertex 3"
            ),
            other => {
                panic!("expected a mesh error while the uv reader holds the mesh, got {other:?}")
            }
        }
        assert!(released.is_ok(), "got {released:?}");
    });
}

/// Moves the triangle by an instance's `offset`, read beside a vertex `uv` at the same location.
const OFFSET_UV: &str = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) offset: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position + offset, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

// Story: instances added after clear_instances hold what the shader reads from them,
// or render names the attribute, where a vertex attribute at its location stood in.
#[test]
fn cleared_instances_hold_what_the_shader_reads() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(OFFSET_UV).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("uv", [0.0, 0.0])));
        mesh.add_instance(Instance::new().set("offset", [0.0, 0.0]));
        shader.add_mesh(&mesh).expect("attach");

        mesh.clear_instances();
        mesh.add_instance(Instance::new());
        let empty = renderer.render(&shader, &target);
        mesh.clear_instances();
        let none = renderer.render(&shader, &target);

        let message = |result: Result<(), fragmentcolor::RendererError>| match result {
            Err(fragmentcolor::RendererError::MeshError(error)) => error.to_string(),
            other => panic!("expected a mesh error, got {other:?}"),
        };
        assert_eq!(
            message(empty),
            "Invalid instance: offset: expected vec2<f32>, got no value at instance 0"
        );
        assert_eq!(
            message(none),
            "Invalid instance: offset: expected vec2<f32>, got no instances"
        );
    });
}

// Story: an f32 or u32 from Rust keeps its type, so it must match the shader exactly.
#[test]
fn typed_rust_values_must_match() {
    let typed = Shader::new(TYPED).expect("shader failed");
    let float = Shader::new(FLOAT).expect("shader failed");
    let id: u32 = 7;

    let mesh = Mesh::new();
    mesh.add_vertices(
        FULLSCREEN.map(|position| Vertex::new(position).set("id", id).set("cell", [-1, 2])),
    );
    typed.add_mesh(&mesh).expect("a u32 fits a u32");

    let mesh = Mesh::new();
    mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("id", id)));
    let error = float.add_mesh(&mesh).expect_err("a u32 is not an f32");
    assert_eq!(
        error.to_string(),
        "Mesh attribute id: expected f32, got u32 at vertex 0"
    );

    let mesh = Mesh::new();
    mesh.add_vertices(FULLSCREEN.map(|position| {
        Vertex::new(position)
            .set("id", 1)
            .set("cell", [0.5f32, 0.5])
    }));
    let error = typed.add_mesh(&mesh).expect_err("f32 values are not i32");
    assert_eq!(
        error.to_string(),
        "Mesh attribute cell: expected vec2<i32>, got vec2<f32> at vertex 0"
    );
}

// Story: a value that doesn't fit fails the attach, naming the attribute, the type and the value.
#[test]
fn a_value_that_doesnt_fit_names_attribute_type_and_value() {
    let shader = Shader::new(TYPED).expect("shader failed");
    let failures = [
        (
            [1.0, -1.0, 1.0],
            [[-1.0, 2.0]; 3],
            "Mesh attribute id: expected u32, got -1 at vertex 1",
        ),
        (
            [1.0; 3],
            [[-1.0, 2.5], [-1.0, 2.0], [-1.0, 2.0]],
            "Mesh attribute cell: expected vec2<i32>, got 2.5 at vertex 0",
        ),
    ];
    for (ids, cells, text) in failures {
        let mesh = Mesh::new();
        for ((position, id), cell) in FULLSCREEN.into_iter().zip(ids).zip(cells) {
            mesh.add_vertex(Vertex::new(position).set("id", id).set("cell", cell));
        }
        let error = shader.validate_mesh(&mesh).expect_err("validate");
        assert_eq!(error.to_string(), text);
        let error = shader.add_mesh(&mesh).expect_err("attach");
        assert_eq!(error.to_string(), text);
    }

    let mesh = Mesh::new();
    mesh.add_vertices(
        FULLSCREEN.map(|position| Vertex::new(position).set("id", 1).set("cell", [1, 2, 3])),
    );
    let error = shader.add_mesh(&mesh).expect_err("three numbers");
    assert_eq!(
        error.to_string(),
        "Mesh attribute cell: expected vec2<i32>, got 3 number(s) at vertex 0"
    );
}

// Story: once a shader types an attribute, a shader that declares another type can't attach,
// and the first shader keeps drawing.
#[test]
fn two_shaders_cannot_type_one_attribute_two_ways() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let typed = Shader::new(TYPED).expect("shader failed");
        let float = Shader::new(FLOAT).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| {
            Vertex::new(position)
                .set("id", 16_777_217)
                .set("cell", [-1, 2])
        }));

        typed.add_mesh(&mesh).expect("first shader types id");
        let error = float.add_mesh(&mesh).expect_err("second shader disagrees");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id is u32 on this mesh, but the shader declares f32"
        );

        assert_green(&draw(&renderer, &typed).await);
    });
}

// Story: an attribute no shader reads packs as floats, and the first shader that reads it types it,
// while the shader that ignores it keeps drawing.
#[test]
fn an_unread_number_is_a_float_until_a_shader_types_it() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let typed = Shader::new(TYPED).expect("shader failed");
        let weight = Shader::new(WEIGHT).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| {
            Vertex::new(position)
                .set("id", 16_777_217)
                .set("cell", [-1, 2])
                .set("weight", 3)
        }));

        typed.add_mesh(&mesh).expect("attach");
        assert_green(&draw(&renderer, &typed).await);

        weight.add_mesh(&mesh).expect("weight takes u32");
        assert_green(&draw(&renderer, &weight).await);
        assert_green(&draw(&renderer, &typed).await);
    });
}

// Story: Pass::add_mesh tries the next shader when one rejects the mesh, and when every shader
// rejects it, the error says why the pass's last shader did.
#[test]
fn pass_add_mesh_tries_the_next_shader() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let float = Shader::new(FLOAT).expect("shader failed");
        let typed = Shader::new(TYPED).expect("shader failed");
        let pass = Pass::from_shader("float", &float);
        pass.add_shader(&typed).expect("add_shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(
            FULLSCREEN.map(|position| Vertex::new(position).set("id", 2.5).set("cell", [-1, 2])),
        );

        pass.add_mesh(&mesh).expect("the float shader takes 2.5");
        let error = typed.add_mesh(&mesh).expect_err("id is an f32 now");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id is f32 on this mesh, but the shader declares u32"
        );
        assert_green(&draw(&renderer, &float).await);

        let pass = Pass::from_shader("typed", &typed);
        let mesh = Mesh::new();
        mesh.add_vertices(
            FULLSCREEN.map(|position| Vertex::new(position).set("id", 2.5).set("cell", [-1, 2])),
        );
        let error = pass.add_mesh(&mesh).expect_err("2.5 is not a u32");
        assert_eq!(
            error.to_string(),
            "No compatible shader exists for this mesh: Mesh attribute id: expected u32, got 2.5 at vertex 0"
        );
    });
}

// Story: after clear_instances the next instance sets the instance layout again, while the type
// a shader recorded for an attribute still holds for the instances that follow.
#[test]
fn cleared_instances_keep_a_recorded_type() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let typed = Shader::new(TYPED).expect("shader failed");
        let float = Shader::new(FLOAT).expect("shader failed");
        let weight = Shader::new(WEIGHT).expect("shader failed");
        let mesh = Mesh::new();
        mesh.add_vertices(FULLSCREEN.map(|position| Vertex::new(position).set("cell", [-1, 2])));
        mesh.add_instance(Instance::new().set("id", 16_777_217));
        typed.add_mesh(&mesh).expect("id takes u32");
        assert_green(&draw(&renderer, &typed).await);

        mesh.clear_instances();
        mesh.add_instance(Instance::new().set("id", 2.5f32));
        let error = float.add_mesh(&mesh).expect_err("id is a u32 on this mesh");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id is u32 on this mesh, but the shader declares f32"
        );
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let error = renderer
            .render(&Pass::from_shader("typed", &typed), &target)
            .expect_err("an f32 can't pack as the recorded u32");
        assert!(
            error
                .to_string()
                .contains("id: expected u32, got f32 at instance 0"),
            "{error}"
        );

        mesh.clear_instances();
        mesh.add_instance(Instance::new().set("id", 16_777_217).set("weight", 3));
        assert_green(&draw(&renderer, &typed).await);
        weight.add_mesh(&mesh).expect("weight takes u32");
        assert_green(&draw(&renderer, &weight).await);
        assert_green(&draw(&renderer, &typed).await);
    });
}
