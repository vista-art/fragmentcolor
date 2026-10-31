//! Mesh tests: schemas, packing, uploads and races.

use super::schema::{Field, MODEL_COLUMNS};
use super::*;
use crate::core::RenderContext;
use crate::testing::device::request_small_device;

impl MeshObject {
    /// What the shader `shader` reads, while it holds the mesh.
    pub(crate) fn reads_of(&self, shader: u64) -> Option<AttributeTypes> {
        let required = self.required.read();
        required.get(&shader).map(|attached| attached.reads.clone())
    }
}

/// The streams the next upload packs again.
fn stale(mesh: &Mesh) -> Repacked {
    let packed = mesh.object.packed.read();
    Repacked {
        vertices: packed.vertices_at != Some(mesh.object.vertex_changes.load(Ordering::Acquire)),
        instances: packed.instances_at
            != Some(mesh.object.instance_changes.load(Ordering::Acquire)),
    }
}

// Story: the Model columns are told apart by name and format alone, so the
// per-frame check that skips a Model's mesh builds no layout to compare with.
#[test]
fn the_model_columns_match_by_name_and_format() {
    let floats = |names: &[&str]| {
        let fields = names
            .iter()
            .map(|name| Field::new(name, wgpu::VertexFormat::Float32x4));
        VertexSchema::new(fields.collect())
    };
    let integers = VertexSchema::new(
        MODEL_COLUMNS
            .map(|name| Field::new(name, wgpu::VertexFormat::Uint32x4))
            .to_vec(),
    );

    assert!(build_columns().is_model());
    assert!(floats(&MODEL_COLUMNS).is_model());
    assert!(!floats(&MODEL_COLUMNS[..3]).is_model(), "three columns");
    assert!(!floats(&["model_0", "model_1", "model_2", "tint"]).is_model());
    assert!(!integers.is_model(), "another format");
}

#[test]
fn derives_schema_and_stride_for_vertex() {
    let v = Vertex::new([0.1f32, 0.2f32])
        .set("uv", [0.3f32, 0.4])
        .set("color", [1.0f32, 1.0, 1.0, 1.0]);
    let schema = derive_vertex_schema(&v, &HashMap::new());
    // position (vec2), color (vec4), uv (vec2) sorted by key name
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields[0].name, "position");
    assert_eq!(schema.fields[0].fmt, wgpu::VertexFormat::Float32x2);
    assert_eq!(schema.fields[1].name, "color");
    assert_eq!(schema.fields[1].fmt, wgpu::VertexFormat::Float32x4);
    assert_eq!(schema.fields[2].name, "uv");
    assert_eq!(schema.fields[2].fmt, wgpu::VertexFormat::Float32x2);
    assert_eq!(schema.stride, 8 + 16 + 8);
}

// Story: one key with two shapes would misalign every later vertex, so packing
// rejects it instead, and the same goes for a position of the wrong dimension.
#[test]
fn ensure_packed_rejects_values_that_differ_from_the_layout() {
    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([0.0f32, 0.0, 0.0]).set("uv", [0.0f32, 0.0, 0.0]),
        Vertex::new([1.0f32, 0.0, 0.0]).set("uv", [1.0f32, 0.0]),
        Vertex::new([0.0f32, 1.0, 0.0]).set("uv", [0.0f32, 1.0, 0.0]),
    ]);
    match mesh.object.ensure_packed() {
        Err(MeshError::InvalidVertex(text)) => {
            assert_eq!(text, "uv: expected vec3<f32>, got vec2<f32> at vertex 1");
        }
        other => panic!("expected an invalid vertex, got {other:?}"),
    }

    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([0.0f32, 0.0, 0.0]),
        Vertex::new([1.0f32, 0.0]),
        Vertex::new([0.0f32, 1.0, 0.0]),
    ]);
    assert!(matches!(
        mesh.object.ensure_packed(),
        Err(MeshError::LayoutMismatch(_))
    ));

    let mesh = Mesh::new();
    mesh.add_vertices([[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]]);
    mesh.add_instances([
        Instance::new().set("offset", [0.0f32, 0.0]),
        Instance::new().set("offset", 1.0f32),
    ]);
    match mesh.object.ensure_packed() {
        Err(MeshError::InvalidInstance(text)) => {
            assert_eq!(text, "offset: expected vec2<f32>, got f32 at instance 1");
        }
        other => panic!("expected an invalid instance, got {other:?}"),
    }
}

// Story: a vertex that sets a property named position fails the pack, since its
// position comes from Vertex::new, and the mesh layout would list it twice.
#[test]
fn ensure_packed_refuses_a_position_property() {
    let mesh = Mesh::new();
    mesh.add_vertices(
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
            .map(|corner| Vertex::new(corner).set("position", [5.0, 5.0])),
    );

    match mesh.object.ensure_packed() {
        Err(MeshError::InvalidVertex(text)) => assert_eq!(
            text,
            "vertex 0 sets a property named 'position', and a vertex takes its position from Vertex::new"
        ),
        other => panic!("expected an invalid vertex, got {other:?}"),
    }
}

// Story: vertex, index and instance data past the device's buffer limit fail
// the upload with an error that names the buffer, before any buffer is made,
// and fail again on the next upload while the data stays too large.
#[test]
fn uploads_stay_within_the_buffer_limit() {
    pollster::block_on(async move {
        let (device, queue) = request_small_device(65536).await.expect("device");
        let context = RenderContext::new(device, queue);
        let corners = (0..8193).map(|index| [index as f32, 0.0]);
        let vertices = Mesh::new();
        vertices.add_vertices(corners);
        let indices = Mesh::new();
        indices.add_vertices([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        indices.set_indices((0..16386).map(|index| index % 3));
        let instances = Mesh::new();
        instances.add_vertices([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        instances.add_instances((0..4097).map(|_| Instance::new().set("tint", [1.0; 4])));

        let message = |mesh: &Mesh| match mesh.object.upload(&context) {
            Err(error) => error.to_string(),
            Ok(_) => panic!("expected an error"),
        };

        assert_eq!(
            message(&vertices),
            "the mesh's vertex buffer needs max_buffer_size of at least 65544, and the device has 65536"
        );
        assert_eq!(
            message(&vertices),
            "the mesh's vertex buffer needs max_buffer_size of at least 65544, and the device has 65536",
            "the next upload"
        );
        assert_eq!(
            message(&indices),
            "the mesh's index buffer needs max_buffer_size of at least 65544, and the device has 65536"
        );
        assert_eq!(
            message(&instances),
            "the mesh's instance buffer needs max_buffer_size of at least 65552, and the device has 65536"
        );
    });
}

// Story: the mesh builds a shader's layouts once for each pack and keeps them
// until it packs again, and a shader that lets the mesh go leaves none behind.
#[test]
fn layouts_stay_until_the_mesh_packs_again() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let shader = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        let mesh = Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        shader.add_mesh(&mesh).expect("attach");
        renderer.render(&shader, &target).expect("render");
        let id = shader.object.id;
        let refuse = || Err(RendererError::Error("rebuilt".into()));
        let latest = |mesh: &Mesh| {
            let cache = mesh.object.cache.read();
            cache.as_ref().map(|cached| cached.upload.schemas.clone())
        };
        let fetch = |schemas: Option<Arc<Schemas>>| {
            let schemas = schemas.expect("an upload");
            mesh.object.fetch_layouts(id, &schemas, refuse)
        };

        let first = latest(&mesh);
        let kept = fetch(first.clone());
        mesh.add_vertices([[0.0, 0.0], [0.1, 0.0], [0.0, 0.1]]);
        renderer.render(&shader, &target).expect("render again");
        let older = fetch(first);
        let newer = fetch(latest(&mesh));
        shader.remove_mesh(&mesh);
        let entries = mesh.object.required.read().len();
        let released = fetch(latest(&mesh));

        assert!(kept.is_ok(), "a still mesh keeps its layouts: {kept:?}");
        assert!(older.is_err(), "an older pack builds them again");
        assert!(newer.is_ok(), "the render kept the new pack's: {newer:?}");
        assert_eq!(entries, 0, "a shader that let go keeps no entry");
        assert!(released.is_err(), "and no layouts");
    });
}

// Story: a shader draws a mesh with the layouts of the pack its buffers came
// from, so another shader that types an attribute meanwhile doesn't fail the frame.
#[test]
fn layouts_follow_the_pack_the_buffers_came_from() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let reads_color = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) color: vec4<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, color.a); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        let reads_weight = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) weight: u32) -> @builtin(position) vec4<f32> { return vec4<f32>(p, f32(weight), 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        let mesh = Mesh::new();
        mesh.add_vertices(
            [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
                .map(|corner| Vertex::new(corner).set("color", [1.0; 4]).set("weight", 2)),
        );
        reads_color.add_mesh(&mesh).expect("attach");
        let late = mesh.clone();
        let attached = std::rc::Rc::new(std::cell::RefCell::new(None));
        let seen = attached.clone();
        // The weight shader types the plain numbers as u32, which resets the
        // mesh layout after the color shader's upload and before its layouts.
        crate::testing::race_hooks::once("uploaded", move || {
            *seen.borrow_mut() = Some(reads_weight.add_mesh(&late).map(drop));
        });

        let rendered = renderer.render(&reads_color, &target);

        let attached = attached.borrow_mut().take().expect("an attach in between");
        assert!(attached.is_ok(), "{attached:?}");
        assert!(rendered.is_ok(), "{rendered:?}");
    });
}

// Story: the quick view checks the mesh before each draw instead of recording
// what it reads, so a shader of the user's own reads only what it declares.
#[test]
fn the_quick_view_leaves_other_shaders_their_rules() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
        renderer.render(&mesh, &target).expect("quick view");
        let own = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");
        own.add_mesh(&mesh).expect("attach");

        mesh.add_vertices(corners.map(Vertex::new));
        let drawn = renderer.render(&own, &target);
        let previewed = renderer.render(&mesh, &target);

        assert!(drawn.is_ok(), "the own shader reads no color: {drawn:?}");
        let error = previewed
            .expect_err("the quick view reads color")
            .to_string();
        assert!(error.contains("color"), "{error}");
        assert!(error.contains("vertex 3"), "{error}");
    });
}

// Story: the quick view fixes the types it gives plain numbers each time it
// checks the mesh, so a color it later reads from an instance stays four floats.
#[test]
fn the_quick_view_keeps_the_types_a_new_check_finds() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(
            corners.map(|corner| Vertex::new(corner).set("color", [1.0, 0.0, 0.0, 1.0])),
        );
        renderer.render(&mesh, &target).expect("first render");
        // An instance color is read before the vertex one.
        mesh.add_instance(Instance::new().set("color", [0.0, 1.0, 0.0, 1.0]));
        renderer
            .render(&mesh, &target)
            .expect("render with the instance");
        let integers = crate::Shader::new(
            "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) color: vec4<u32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, f32(color.x), 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        )
        .expect("shader");

        let attached = integers.add_mesh(&mesh);
        let again = renderer.render(&mesh, &target);

        match attached {
            Err(ShaderError::AttributeConflict { name, .. }) => assert_eq!(name, "color"),
            other => panic!("expected the color conflict, got {other:?}"),
        }
        assert!(again.is_ok(), "{again:?}");
    });
}

// Story: every render of a mesh the quick view refuses says why,
// concurrent ones included, rather than the error for a shader with no mesh.
#[test]
fn every_render_of_a_refused_quick_view_says_why() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        mesh.add_vertices(
            [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
                .map(|corner| Vertex::new(corner).set("position", [5.0, 5.0])),
        );
        mesh.passes();
        let pass = crate::Pass {
            object: mesh.pass.clone(),
        };

        for render in ["first", "second"] {
            let error = renderer.render(&pass, &target).expect_err(render);
            let text = error.to_string();
            assert!(
                text.contains("property named 'position'"),
                "{render}: {text}"
            );
        }
    });
}

// Story: a pass that requires a mesh draws it through the quick view, which
// checks the mesh on each render of that pass, so a vertex added later without
// the color the quick view reads fails the render instead of drawing zeros.
#[test]
fn a_required_quick_view_checks_each_render() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
        let pass = crate::Pass::new("main");
        pass.require(&mesh).expect("require");
        renderer.render(&pass, &target).expect("first render");

        mesh.add_vertices(corners.map(Vertex::new));
        let error = renderer
            .render(&pass, &target)
            .expect_err("vertex 3 has no color")
            .to_string();

        assert!(error.contains("color"), "{error}");
        assert!(error.contains("vertex 3"), "{error}");
    });
}

// Story: a pass that requires a mesh while it is empty draws the mesh
// once it has vertices, and reports a vertex the quick view refuses.
#[test]
fn a_mesh_required_while_empty_draws_once_it_has_vertices() {
    pollster::block_on(async move {
        use crate::Target;
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        let mesh = Mesh::new();
        let pass = crate::Pass::new("main");
        pass.load_previous();
        pass.require(&mesh).expect("require");
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));

        renderer.render(&pass, &target).expect("render");
        let image = target.get_image().await.expect("read back");

        let refused = Mesh::new();
        let other = crate::Pass::new("other");
        other.require(&refused).expect("require");
        refused.add_vertices(corners.map(|corner| Vertex::new(corner).set("position", [5.0, 5.0])));
        let error = renderer
            .render(&other, &target)
            .expect_err("a position property")
            .to_string();

        assert_eq!(mesh.pass.shaders.read().len(), 1, "the quick view's shader");
        assert_eq!(image[..4], [255, 255, 255, 255], "the white mesh");
        assert!(error.contains("property named 'position'"), "{error}");
    });
}

// Story: a mesh added to another mesh's quick view draws beside it, since
// only the quick view's own mesh waits for its generation to hold still.
#[test]
fn a_second_mesh_on_a_quick_view_draws() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        let colored = corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4]));
        let first = Mesh::new();
        first.add_vertices(colored.clone());
        renderer.render(&first, &target).expect("first render");
        let second = Mesh::new();
        second.add_vertices(colored.clone());
        second.add_vertices(colored);
        first.passes()[0].add_mesh(&second).expect("attach");
        let apart = first.object.generation() != second.object.generation();

        let rendered = renderer.render(&first, &target);

        assert!(apart, "the two meshes count their changes apart");
        assert!(rendered.is_ok(), "{rendered:?}");
    });
}

// Story: a mesh the quick view refused when a pass required it draws
// once it fits, instead of failing as a shader with no mesh.
#[test]
fn a_required_quick_view_draws_a_mesh_fixed_later() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
        // An instance color is read first, and two floats don't fit the vec4.
        mesh.add_instance(Instance::new().set("color", [1.0f32, 0.0]));
        let pass = crate::Pass::new("main");
        pass.require(&mesh).expect("require");

        let refused = renderer.render(&pass, &target);
        mesh.clear_instances();
        let fixed = renderer.render(&pass, &target);

        let error = refused.expect_err("the instance color").to_string();
        assert!(error.contains("color"), "{error}");
        assert!(fixed.is_ok(), "{fixed:?}");
    });
}

// Story: a check of the quick view on another thread can't let the mesh go between
// this render's check and its read of the mesh list, so a vertex without color
// added meanwhile fails the render with its own error, not a shader with no mesh.
#[test]
fn a_concurrent_check_leaves_the_render_its_error() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
        renderer.render(&mesh, &target).expect("first render");
        let shader = mesh.pass.shaders.read()[0].clone();
        let late = mesh.clone();
        let checker = Arc::new(parking_lot::Mutex::new(None));
        let slot = checker.clone();
        // Vertices without color arrive, and another thread checks the quick view
        // and gets a while to let the mesh go, before this render reads its meshes.
        crate::testing::race_hooks::once("preview checked", move || {
            late.add_vertices(corners.map(Vertex::new));
            let (sender, receiver) = std::sync::mpsc::channel();
            let thread = std::thread::spawn(move || {
                let _ = shader.check_preview();
                let _ = sender.send(());
            });
            let _ = receiver.recv_timeout(std::time::Duration::from_millis(200));
            *slot.lock() = Some(thread);
        });

        let rendered = renderer.render(&mesh, &target);
        if let Some(thread) = checker.lock().take() {
            thread.join().expect("the checker thread");
        }

        let error = rendered.expect_err("vertex 3 has no color").to_string();
        assert!(error.contains("vertex 3"), "{error}");
    });
}

// Story: the render checks the quick view's mesh again when it changed during
// the upload, so a vertex another thread adds while the pack runs fails that render.
#[test]
fn a_vertex_added_during_the_quick_view_pack_is_checked() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("texture target");
        let mesh = Mesh::new();
        let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
        mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
        mesh.set_indices([0, 1, 2]);
        let late = mesh.clone();
        let (visible, seen) = std::sync::mpsc::channel();
        let (finish, finished) = std::sync::mpsc::channel::<()>();
        let adder = Arc::new(parking_lot::Mutex::new(None));
        let slot = adder.clone();
        // Another thread adds a vertex without color and holds still once other
        // threads can see it, while this one packs.
        crate::testing::race_hooks::once("pack", move || {
            let thread = std::thread::spawn(move || {
                crate::testing::race_hooks::once("added", move || {
                    let _ = visible.send(());
                    let _ = finished.recv();
                });
                late.add_vertex([0.0, 0.0]);
            });
            let _ = seen.recv_timeout(std::time::Duration::from_secs(10));
            *slot.lock() = Some(thread);
        });

        let rendered = renderer.render(&mesh, &target);
        let _ = finish.send(());
        if let Some(thread) = adder.lock().take() {
            thread.join().expect("the adder thread");
        }

        let error = rendered.expect_err("vertex 3 has no color").to_string();
        assert!(error.contains("vertex 3"), "{error}");
    });
}

/// Watches the hook point `name` and reports whether `locked` held there, with the
/// generation still where it was when the watch began, so the change had no reader.
fn watch_change(
    object: &Arc<MeshObject>,
    name: &'static str,
    locked: fn(&MeshObject) -> bool,
) -> std::rc::Rc<std::cell::Cell<Option<bool>>> {
    let seen = std::rc::Rc::new(std::cell::Cell::new(None));
    let (object, kept, before) = (object.clone(), seen.clone(), object.generation());
    crate::testing::race_hooks::once(name, move || {
        kept.set(Some(locked(&object) && object.generation() == before));
    });
    seen
}

// Story: a change stays out of every reader's reach until the mesh counts it,
// so a pack that sees a vertex or an instance change finds the generation moved.
#[test]
fn a_change_is_counted_before_any_reader_sees_it() {
    let mesh = Mesh::new();
    let verts = |object: &MeshObject| object.verts.try_read().is_none();
    let insts = |object: &MeshObject| object.insts.try_read().is_none();

    let vertex = watch_change(&mesh.object, "pushed", verts);
    mesh.add_vertex([0.0, 0.0]);
    let instance = watch_change(&mesh.object, "instance pushed", insts);
    mesh.add_instance(Instance::new());
    let cleared = watch_change(&mesh.object, "instances cleared", insts);
    mesh.clear_instances();

    assert_eq!(vertex.get(), Some(true), "add_vertex");
    assert_eq!(instance.get(), Some(true), "add_instance");
    assert_eq!(cleared.get(), Some(true), "clear_instances");
}

// Story: a batch of vertices or instances lands as one change, so a render on
// another thread sees all of the batch or none of it, never part of a triangle.
#[test]
fn a_batch_lands_as_one_change() {
    let mesh = Mesh::new();
    let (object, seen) = (
        mesh.object.clone(),
        std::rc::Rc::new(std::cell::Cell::new(0)),
    );
    let kept = seen.clone();
    crate::testing::race_hooks::once("added", move || kept.set(object.verts.read().len()));
    let hidden = watch_change(&mesh.object, "pushed", |object| {
        object.verts.try_read().is_none()
    });

    let before = mesh.object.generation();
    mesh.add_vertices([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
    let vertices = mesh.object.generation() - before;
    let before = mesh.object.generation();
    mesh.add_instances([Instance::new(), Instance::new()]);
    let instances = mesh.object.generation() - before;
    let before = mesh.object.generation();
    mesh.add_vertices(Vec::<Vertex>::new());
    let nothing = mesh.object.generation() - before;

    assert_eq!(seen.get(), 3, "the first reader sees the whole batch");
    assert_eq!(hidden.get(), Some(true), "and none of it before the count");
    assert_eq!(vertices, 1, "one change for three vertices");
    assert_eq!(instances, 1, "one change for two instances");
    assert_eq!(nothing, 0, "an empty batch changes nothing");
}

// Story: the quick view reads the first vertex in a statement of its own,
// so it holds no vertex guard while the shader checks the vertices again.
// Under a fair lock, a writer waiting between the two reads would hang both.
#[test]
fn the_quick_view_holds_no_vertex_guard_while_it_attaches() {
    let mesh = Mesh::new();
    mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
    let object = mesh.object.clone();
    let free = std::rc::Rc::new(std::cell::Cell::new(false));
    let seen = free.clone();
    crate::testing::race_hooks::once("preview", move || {
        seen.set(object.verts.try_write().is_some());
    });

    mesh.passes();

    assert!(
        free.get(),
        "the vertices were locked while the mesh attached"
    );
}

// Story: a vertex or an index that arrives while an upload runs is checked by
// the upload that packs it, so a partial triangle never reaches the GPU.
#[test]
fn a_change_during_the_upload_is_checked() {
    pollster::block_on(async move {
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = crate::core::renderer::device::request_device(&adapter)
            .await
            .expect("device");
        let context = RenderContext::new(device, queue);

        let derived = Mesh::new();
        derived.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let late = derived.clone();
        crate::testing::race_hooks::once("pack", move || late.add_vertex([0.5, 0.5]));
        let error = derived.object.upload(&context).expect_err("4 vertices");
        assert_eq!(
            error.to_string(),
            RendererError::from(MeshError::VertexCount(4)).to_string(),
            "derived indices"
        );

        let given = Mesh::new();
        given.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        given.set_indices([0, 1, 2]);
        let late = given.clone();
        crate::testing::race_hooks::once("pack", move || late.set_indices([0, 1, 2, 0]));
        let error = given.object.upload(&context).expect_err("4 indices");
        assert_eq!(
            error.to_string(),
            RendererError::from(MeshError::IndexCount(4)).to_string(),
            "given indices"
        );
    });
}

// Story: an instance added after the pack and before the write stays unpacked,
// and the upload draws the rows its own pack made.
#[test]
fn an_instance_added_during_the_upload_packs_next_time() {
    pollster::block_on(async move {
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = crate::core::renderer::device::request_device(&adapter)
            .await
            .expect("device");
        let context = RenderContext::new(device, queue);
        let mesh = Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let tint = || Instance::new().set("tint", [1.0; 4]);
        mesh.add_instances([tint(), tint(), tint()]);
        mesh.object.upload(&context).expect("first upload");
        mesh.clear_instances();
        mesh.add_instances([tint(), tint()]);

        let late = mesh.clone();
        crate::testing::race_hooks::once("packed", move || late.add_instance(tint()));
        let during = mesh.object.upload(&context).expect("second upload").counts;
        let after = mesh.object.upload(&context).expect("third upload").counts;

        assert_eq!(during.instance_rows, Some(2), "the rows the pack made");
        assert_eq!(
            after.instance_rows,
            Some(3),
            "the late instance packs next time"
        );
        assert_eq!(mesh.object.packed.read().rows, 3);
    });
}

// Story: a Model attach sets the Model columns before it records the columns
// as reads, so a render of the mesh's other shaders that packs in between passes.
#[test]
fn a_pack_during_a_model_attach_finds_the_columns() {
    pollster::block_on(async move {
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = crate::core::renderer::device::request_device(&adapter)
            .await
            .expect("device");
        let context = RenderContext::new(device, queue);
        let mesh = Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let mut read = AttributeTypes::default();
        for field in build_columns().fields {
            read.instance.insert(field.name, field.fmt);
        }
        let binding = Binding {
            read,
            columns: true,
            ..Default::default()
        };

        let object = mesh.object.clone();
        let packed = std::rc::Rc::new(std::cell::RefCell::new(None));
        let seen = packed.clone();
        crate::testing::race_hooks::once("columns", move || {
            let upload = object.upload(&context).map(drop);
            *seen.borrow_mut() = Some(upload);
        });
        mesh.object.declare(1, binding).expect("declare");

        let upload = packed.borrow_mut().take().expect("a pack in between");
        assert!(upload.is_ok(), "{upload:?}");
    });
}

// Story: the cached upload names the context it belongs to, so after two
// contexts take turns, the cache and the buffers belong to the last one.
#[test]
fn the_cache_belongs_to_the_context_that_filled_it() {
    pollster::block_on(async move {
        let mut contexts = Vec::new();
        for _ in 0..2 {
            let instance = crate::core::renderer::device::create_instance().await;
            let adapter = crate::core::renderer::device::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::core::renderer::device::request_device(&adapter)
                .await
                .expect("device");
            contexts.push(RenderContext::new(device, queue));
        }
        let mesh = Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);

        for context in [&contexts[0], &contexts[1], &contexts[0], &contexts[1]] {
            mesh.object.upload(context).expect("buffers");
            let cache = mesh.object.cache.read();
            let gpu = mesh.object.gpu.read();
            assert_eq!(cache.as_ref().map(|cached| cached.owner), Some(context.id));
            assert_eq!(gpu.as_ref().map(|streams| streams.owner), Some(context.id));
        }
    });
}

// Story: a plain number packs as a float until a shader types it, then
// the mesh repacks it in that type and keeps it for the mesh's lifetime.
#[test]
fn untyped_attributes_pack_as_floats_until_declared() {
    let mesh = Mesh::new();
    mesh.add_vertices([
        Vertex::new([0.0f32, 0.0]).set("id", 3),
        Vertex::new([1.0f32, 0.0]).set("id", 4),
        Vertex::new([0.0f32, 1.0]).set("id", 5),
    ]);
    let field = |mesh: &Mesh| -> (wgpu::VertexFormat, Vec<u8>, Vec<u8>) {
        mesh.object.ensure_packed().expect("pack");
        let schema = mesh.object.vertex_schema.read().clone().expect("schema");
        let bytes = mesh.object.packed.read().vertices.clone();
        let stride = schema.stride as usize;
        (
            schema.fields[1].fmt,
            bytes[8..12].to_vec(),
            bytes[stride + 8..stride + 12].to_vec(),
        )
    };

    let (format, first, second) = field(&mesh);
    assert_eq!(format, wgpu::VertexFormat::Float32);
    assert_eq!(first, 3.0f32.to_ne_bytes());
    assert_eq!(second, 4.0f32.to_ne_bytes());

    let mut added = AttributeTypes::default();
    added
        .vertex
        .insert("id".to_string(), wgpu::VertexFormat::Uint32);
    let binding = Binding {
        types: added,
        ..Default::default()
    };
    mesh.object.declare(1, binding).expect("declare");
    assert!(mesh.object.vertex_schema.read().is_none());
    assert!(stale(&mesh).vertices);

    let (format, first, second) = field(&mesh);
    assert_eq!(format, wgpu::VertexFormat::Uint32);
    assert_eq!(first, 3u32.to_ne_bytes());
    assert_eq!(second, 4u32.to_ne_bytes());

    let mut conflict = AttributeTypes::default();
    conflict
        .vertex
        .insert("id".to_string(), wgpu::VertexFormat::Float32);
    let binding = Binding {
        types: conflict,
        ..Default::default()
    };
    let error = mesh.object.declare(1, binding).expect_err("conflict");
    assert_eq!(
        error.to_string(),
        "Mesh attribute id is u32 on this mesh, but the shader declares f32"
    );
    assert_eq!(
        mesh.object.types.read().vertex.get("id"),
        Some(&wgpu::VertexFormat::Uint32)
    );
}

// Story: attaching to shaders that agree on every type keeps the packed
// bytes and the GPU buffers, since nothing about them changes.
#[test]
fn declaring_nothing_keeps_the_gpu_cache() {
    pollster::block_on(async move {
        let wgsl = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos, f32(id), 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([-1.0f32, -1.0]).set("id", 1),
            Vertex::new([3.0f32, -1.0]).set("id", 1),
            Vertex::new([-1.0f32, 3.0]).set("id", 1),
        ]);
        let first = crate::Shader::new(wgsl).expect("shader");
        first.add_mesh(&mesh).expect("first attach");

        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = crate::core::renderer::device::request_device(&adapter)
            .await
            .expect("device");
        let context = RenderContext::new(device, queue);
        mesh.object.upload(&context).expect("buffers");
        let cached = |mesh: &Mesh| {
            let generation = mesh.object.generation.load(Ordering::Acquire);
            mesh.object.find_cached(context.id, generation).is_some()
        };
        assert!(cached(&mesh));

        let second = crate::Shader::new(wgsl).expect("shader");
        second.add_mesh(&mesh).expect("second attach");
        first.validate_mesh(&mesh).expect("validate again");
        assert!(cached(&mesh));
        assert!(!stale(&mesh).vertices);
    });
}

#[test]
fn ensure_packed_deduplicates_and_indices_track() {
    let mesh = Mesh::new();
    use crate::Vertex;
    let a = Vertex::new([0.0f32, 0.0]).set("uv", [0.0f32, 0.0]);
    let b = Vertex::new([1.0f32, 0.0]).set("uv", [1.0f32, 0.0]);
    let c = Vertex::new([0.0f32, 1.0]).set("uv", [0.0f32, 1.0]);
    mesh.add_vertices([a.clone(), b.clone(), c.clone(), a.clone(), c.clone(), b]);

    // Force packing and inspect internals
    mesh.object.ensure_packed().expect("pack");
    let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
    let pv = mesh.object.packed.read().vertices.clone();
    let idx = mesh.object.packed.read().indices.clone();
    // Unique verts should be a, b, c => 3 * stride bytes
    assert_eq!(pv.len() as u64, 3 * stride);
    // Indices mirror [a,b,c,a,c,b] => [0,1,2,0,2,1]
    assert_eq!(idx, vec![0, 1, 2, 0, 2, 1]);
}

#[test]
fn instance_packing_override_and_vertex_buffers() {
    pollster::block_on(async move {
        let mesh = Mesh::new();
        use crate::{Instance, Vertex};
        mesh.add_vertices([
            Vertex::new([-0.5f32, -0.5f32]),
            Vertex::new([0.5f32, -0.5f32]),
            Vertex::new([0.0f32, 0.5f32]),
        ]);
        let inst = Instance::new().set("id", 7u32);
        mesh.add_instances([inst.clone(), inst]);

        // Build GPU buffers
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, queue) = crate::core::renderer::device::request_device(&adapter)
            .await
            .expect("device");
        let context = RenderContext::new(device, queue);

        // Without override, instance_count == 2 (matches buffer)
        let counts = mesh.object.upload(&context).expect("vb").counts;
        assert_eq!(counts.instance_count, 2);

        // With override, instance_count == 5
        mesh.set_instance_count(5);
        let counts2 = mesh.object.upload(&context).expect("vb2").counts;
        assert_eq!(counts2.instance_count, 5);

        // Adding an instance auto-clears the override (otherwise the GPU would
        // draw past the actual buffer end and read garbage). count falls back
        // to the new buffer length (3).
        mesh.add_instance(Instance::new().set("id", 8u32));
        let counts3 = mesh.object.upload(&context).expect("vb3").counts;
        assert_eq!(counts3.instance_count, 3);

        // clear_instances resets everything: empty buffer + no override → 1.
        mesh.set_instance_count(99);
        mesh.clear_instances();
        let counts4 = mesh.object.upload(&context).expect("vb4").counts;
        assert_eq!(counts4.instance_count, 1);
    });
}

#[test]
fn renderable_passes_creates_shader_from_first_vertex() {
    let mesh = Mesh::new();
    use crate::Vertex;
    mesh.add_vertices([
        Vertex::new([-0.5f32, -0.5f32]),
        Vertex::new([0.5f32, -0.5f32]),
        Vertex::new([0.0f32, 0.5f32]),
    ]);
    let passes = mesh.passes();
    assert_eq!(passes.len(), 1);
}

#[test]
fn set_indices_preserves_non_deduped_vertices() {
    let mesh = Mesh::new();
    use crate::Vertex;
    // Three vertices with fully identical attributes — the auto path
    // would collapse these into a single unique vertex.
    let v = Vertex::new([0.0f32, 0.0]).set("uv", [0.5f32, 0.5]);
    mesh.add_vertex(v.clone());
    mesh.add_vertex(v.clone());
    mesh.add_vertex(v);
    mesh.set_indices([0u32, 1, 2]);

    mesh.object.ensure_packed().expect("pack");
    let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
    let pv = mesh.object.packed.read().vertices.clone();
    let idx = mesh.object.packed.read().indices.clone();
    // No dedup: 3 vertices worth of bytes, and indices preserved verbatim.
    assert_eq!(pv.len() as u64, 3 * stride);
    assert_eq!(idx, vec![0, 1, 2]);
}

#[test]
fn clear_indices_restores_auto_derivation() {
    let mesh = Mesh::new();
    use crate::Vertex;
    let a = Vertex::new([0.0f32, 0.0]).set("uv", [0.0f32, 0.0]);
    let b = Vertex::new([1.0f32, 0.0]).set("uv", [1.0f32, 0.0]);
    mesh.add_vertices([a.clone(), b.clone(), a.clone()]);

    // Switch to override mode with explicit indices.
    mesh.set_indices([0u32, 1, 2]);
    mesh.object.ensure_packed().expect("pack override");
    let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
    assert_eq!(mesh.object.packed.read().vertices.len() as u64, 3 * stride);
    assert_eq!(mesh.object.packed.read().indices, vec![0, 1, 2]);

    // Clear and re-pack: dedup should collapse `a` and produce derived indices.
    mesh.clear_indices();
    mesh.object.ensure_packed().expect("pack auto");
    assert_eq!(mesh.object.packed.read().vertices.len() as u64, 2 * stride);
    assert_eq!(mesh.object.packed.read().indices, vec![0, 1, 0]);
}

// Story: a triangle drawn from explicit indices renders without error.
#[test]
fn render_with_mesh_explicit_indices() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");
        let pass = crate::Pass::from_shader("mesh-explicit-indices", &shader);

        let mesh = Mesh::new();
        use crate::Vertex;
        // Four corners of a quad, sharing two positions across the
        // diagonal — dedup would collapse them; explicit indices keep
        // every corner so we can draw two triangles cleanly.
        mesh.add_vertices([
            Vertex::new([-0.5f32, -0.5]),
            Vertex::new([0.5f32, -0.5]),
            Vertex::new([0.5f32, 0.5]),
            Vertex::new([-0.5f32, 0.5]),
        ]);
        mesh.set_indices([0u32, 1, 2, 0, 2, 3]);

        pass.add_mesh(&mesh).expect("mesh is compatible");
        renderer.render(&pass, &target).expect("render ok");
    });
}
