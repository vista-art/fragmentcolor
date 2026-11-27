#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, RendererError, Shader};

const SIMULATION: &str = r#"
struct RangedSim {
    step: u32,
    counts: array<u32, 4>,
}

@group(0) @binding(0) var<storage, read_write> ranged_sim: RangedSim;

@compute @workgroup_size(1)
fn cs_main() {
    for (var i = 0u; i < 4u; i++) {
        ranged_sim.counts[i] += ranged_sim.step;
    }
}
"#;

/// The simulation with twice the counts, so its root takes a buffer of another size.
const WIDER: &str = r#"
struct RangedSim {
    step: u32,
    counts: array<u32, 8>,
}

@group(0) @binding(0) var<storage, read_write> ranged_sim: RangedSim;

@compute @workgroup_size(1)
fn cs_main() {
    for (var i = 0u; i < 8u; i++) {
        ranged_sim.counts[i] += ranged_sim.step;
    }
}
"#;

/// Three halves, six bytes, so the root's last word reaches into its buffer's padding.
/// The pass adds one to `b` on each run.
const HALVES: &str = r#"
enable f16;

struct Halves {
    a: f16,
    b: f16,
    c: f16,
}

@group(0) @binding(0) var<storage, read_write> halves: Halves;

@compute @workgroup_size(1)
fn cs_main() {
    halves.b += 1.0h;
}
"#;

/// Five halves, ten bytes, so the root's third word holds its last half and two bytes of padding.
/// The pass adds one to `c` and `d`, the halves in the middle word.
const FIFTHS: &str = r#"
enable f16;

struct Fifths {
    a: f16,
    b: f16,
    c: f16,
    d: f16,
    e: f16,
}

@group(0) @binding(0) var<storage, read_write> fifths: Fifths;

@compute @workgroup_size(1)
fn cs_main() {
    fifths.c += 1.0h;
    fifths.d += 1.0h;
}
"#;

/// The storage root `ranged_sim` as `Renderer::read_storage` reads it, as u32 words.
async fn read_sim(renderer: &Renderer, shader: &Shader) -> Vec<u32> {
    let bytes = renderer
        .read_storage(shader, "ranged_sim")
        .await
        .expect("read_storage failed");
    split_words(&bytes)
}

fn split_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect()
}

fn split_halves(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| u16::from_le_bytes(*chunk))
        .collect()
}

// Story: A compute pass keeps counts in a storage root, and the app changes one field
// of that root between frames. Only that field goes up, so the counts the pass wrote stay.
#[test]
fn setting_one_field_keeps_what_the_compute_pass_wrote() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step 1");
        renderer.render(&pass, &probe).expect("first render failed");

        // Act
        shader.set("ranged_sim.step", 10u32).expect("set step 10");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            split_words(&bytes),
            [10, 11, 11, 11, 11],
            "step, then counts of 1 + 10"
        );
    });
}

// Story: The app overwrites one item of an array the compute pass keeps.
// Its neighbors keep what the pass wrote.
#[test]
fn setting_one_item_keeps_its_neighbors() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step");
        renderer.render(&pass, &probe).expect("first render failed");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Act
        shader
            .set("ranged_sim.counts[2]", 100u32)
            .expect("set one count");
        renderer.render(&pass, &probe).expect("third render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            split_words(&bytes),
            [1, 3, 3, 101, 3],
            "step, then counts where only item 2 was set"
        );
    });
}

// Story: Raw bytes on the root still replace the whole root, what the pass wrote included.
#[test]
fn setting_the_root_replaces_every_byte() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step");
        renderer.render(&pass, &probe).expect("first render failed");

        // Act
        let root: Vec<u8> = [2u32, 50, 50, 50, 50]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        shader.set("ranged_sim", root.as_slice()).expect("set root");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            split_words(&bytes),
            [2, 52, 52, 52, 52],
            "the root's bytes, then one step of 2"
        );
    });
}

// Story: Two Renderers draw the same simulation. The app sets a field and one Renderer
// draws, then sets another and the other draws. Each Renderer takes both fields, the
// one the other Renderer uploaded included, and keeps the counts its own pass wrote.
#[test]
fn a_second_renderer_gets_the_changes_the_first_one_uploaded() {
    pollster::block_on(async move {
        // Arrange
        let left = Renderer::new();
        let right = Renderer::new();
        let left_probe = left
            .create_texture_target([8, 8])
            .await
            .expect("left target");
        let right_probe = right
            .create_texture_target([8, 8])
            .await
            .expect("right target");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step 1");
        left.render(&pass, &left_probe)
            .expect("left first render failed");
        right
            .render(&pass, &right_probe)
            .expect("right first render failed");

        // Act
        shader.set("ranged_sim.step", 5u32).expect("set step 5");
        left.render(&pass, &left_probe)
            .expect("left second render failed");
        shader
            .set("ranged_sim.counts[3]", 50u32)
            .expect("set one count");
        right
            .render(&pass, &right_probe)
            .expect("right second render failed");
        left.render(&pass, &left_probe)
            .expect("left third render failed");

        // Assert
        assert_eq!(
            read_sim(&right, &shader).await,
            [5, 6, 6, 6, 55],
            "right takes step 5 and count 3 over its counts of 1, then one step"
        );
        assert_eq!(
            read_sim(&left, &shader).await,
            [5, 11, 11, 11, 55],
            "left takes count 3 over its counts of 6, then one step"
        );
    });
}

// Story: Two Renderers draw one simulation, and the app sets its step each frame.
// Each Renderer uploads only the step, so the counts its own pass wrote add up.
#[test]
fn two_renderers_keep_what_their_passes_wrote() {
    pollster::block_on(async move {
        // Arrange
        let left = Renderer::new();
        let right = Renderer::new();
        let left_probe = left
            .create_texture_target([8, 8])
            .await
            .expect("left target");
        let right_probe = right
            .create_texture_target([8, 8])
            .await
            .expect("right target");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);

        // Act
        for step in 1..=3u32 {
            shader.set("ranged_sim.step", step).expect("set step");
            left.render(&pass, &left_probe).expect("left render failed");
            right
                .render(&pass, &right_probe)
                .expect("right render failed");
        }

        // Assert
        assert_eq!(
            read_sim(&left, &shader).await,
            [3, 6, 6, 6, 6],
            "left: step 3, then counts of 1 + 2 + 3"
        );
        assert_eq!(
            read_sim(&right, &shader).await,
            [3, 6, 6, 6, 6],
            "right: step 3, then counts of 1 + 2 + 3"
        );
    });
}

// Story: The app sets two items that don't touch. Both go up in one staging buffer,
// and the items between and after them keep what the pass wrote.
#[test]
fn setting_items_apart_keeps_the_items_between() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step");
        renderer.render(&pass, &probe).expect("first render failed");

        // Act
        shader
            .set("ranged_sim.counts[0]", 100u32)
            .expect("set count 0");
        shader
            .set("ranged_sim.counts[2]", 200u32)
            .expect("set count 2");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Assert
        assert_eq!(
            read_sim(&renderer, &shader).await,
            [1, 101, 2, 201, 2],
            "step, then counts where only items 0 and 2 were set"
        );
    });
}

// Story: Two shaders declare the same storage root, so they share one buffer. A field
// one of them sets goes up, and the fields the other one's pass wrote stay.
#[test]
fn shaders_that_share_a_root_upload_only_what_each_set() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let writer = Shader::new(SIMULATION).expect("writer shader failed");
        let other = Shader::new(SIMULATION).expect("other shader failed");
        let writes = Pass::from_shader("writer", &writer);
        let reads = Pass::from_shader("other", &other);
        writer.set("ranged_sim.step", 1u32).expect("set step");
        renderer
            .render(&writes, &probe)
            .expect("first render failed");
        renderer
            .render(&writes, &probe)
            .expect("second render failed");

        // Act
        other
            .set("ranged_sim.counts[1]", 100u32)
            .expect("set one count");
        renderer
            .render(&reads, &probe)
            .expect("other render failed");

        // Assert
        assert_eq!(
            read_sim(&renderer, &writer).await,
            [1, 3, 101, 3, 3],
            "the writer's step stays, and the other shader's count goes up"
        );
    });
}

// Story: Two shaders share a storage root, and two Renderers draw both. A count the
// second shader sets reaches each Renderer's buffer, and the step the first one set stays.
#[test]
fn a_shared_root_takes_each_set_on_every_renderer() {
    pollster::block_on(async move {
        // Arrange
        let left = Renderer::new();
        let right = Renderer::new();
        let left_probe = left
            .create_texture_target([8, 8])
            .await
            .expect("left target");
        let right_probe = right
            .create_texture_target([8, 8])
            .await
            .expect("right target");
        let writer = Shader::new(SIMULATION).expect("writer shader failed");
        let other = Shader::new(SIMULATION).expect("other shader failed");
        let writes = Pass::from_shader("writer", &writer);
        let reads = Pass::from_shader("other", &other);
        writer.set("ranged_sim.step", 1u32).expect("set step");
        left.render(&writes, &left_probe)
            .expect("left writer render failed");
        right
            .render(&writes, &right_probe)
            .expect("right writer render failed");

        // Act
        other
            .set("ranged_sim.counts[1]", 100u32)
            .expect("set one count");
        left.render(&reads, &left_probe)
            .expect("left other render failed");
        right
            .render(&reads, &right_probe)
            .expect("right other render failed");

        // Assert
        assert_eq!(
            read_sim(&left, &writer).await,
            [1, 2, 101, 2, 2],
            "left: the writer's step, and the other shader's count, then two steps"
        );
        assert_eq!(
            read_sim(&right, &writer).await,
            [1, 2, 101, 2, 2],
            "right: the writer's step, and the other shader's count, then two steps"
        );
    });
}

// Story: Two shaders declare the root at different sizes, so each render recreates its
// buffer. A recreated buffer takes the whole CPU copy, over what the pass wrote.
#[test]
fn a_recreated_buffer_takes_the_whole_root() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let narrow = Shader::new(SIMULATION).expect("narrow shader failed");
        let wide = Shader::new(WIDER).expect("wide shader failed");
        narrow.set("ranged_sim.step", 1u32).expect("set step");
        let narrow_pass = Pass::from_shader("narrow", &narrow);
        let wide_pass = Pass::from_shader("wide", &wide);
        renderer
            .render(&narrow_pass, &probe)
            .expect("first render failed");
        renderer
            .render(&narrow_pass, &probe)
            .expect("second render failed");

        // Act
        renderer
            .render(&wide_pass, &probe)
            .expect("wide render failed");
        renderer
            .render(&narrow_pass, &probe)
            .expect("third render failed");

        // Assert
        assert_eq!(
            read_sim(&renderer, &narrow).await,
            [1, 1, 1, 1, 1],
            "the narrow shader's CPU copy, then one step"
        );
    });
}

// Story: Two shaders share a storage root, and a third declares it at another size.
// The third one's render recreates the buffer, then the writer's recreates it again
// with the writer's copy. The buffer is new to the other shader, so its next render
// brings back the count it set, over what the writer's copy held there.
#[test]
fn a_recreated_shared_buffer_takes_each_set_again() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let writer = Shader::new(SIMULATION).expect("writer shader failed");
        let other = Shader::new(SIMULATION).expect("other shader failed");
        let wide = Shader::new(WIDER).expect("wide shader failed");
        let writes = Pass::from_shader("writer", &writer);
        let reads = Pass::from_shader("other", &other);
        let widens = Pass::from_shader("wide", &wide);
        writer.set("ranged_sim.step", 1u32).expect("set step");
        other
            .set("ranged_sim.counts[1]", 100u32)
            .expect("set one count");
        renderer
            .render(&writes, &probe)
            .expect("writer render failed");
        renderer
            .render(&reads, &probe)
            .expect("other render failed");

        // Act
        renderer
            .render(&widens, &probe)
            .expect("wide render failed");
        renderer
            .render(&writes, &probe)
            .expect("second writer render failed");
        renderer
            .render(&reads, &probe)
            .expect("second other render failed");

        // Assert
        assert_eq!(
            read_sim(&renderer, &writer).await,
            [1, 2, 101, 2, 2],
            "the writer's step, and the other shader's count again, then two steps"
        );
    });
}

// Story: An f16 set goes up as its whole 4-byte word, so the half that shares the
// word is rewritten from the CPU copy. A set of the last half reaches into the
// buffer's padding, past the root's six bytes. Runs where the device has SHADER_F16.
#[test]
fn an_f16_set_uploads_its_whole_word() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(HALVES).expect("halves shader failed");
        let pass = Pass::from_shader("halves", &shader);
        match renderer.render(&pass, &probe) {
            Err(RendererError::FeatureError { .. }) => {
                eprintln!("the device lacks SHADER_F16, so the test has nothing to check");
                return;
            }
            result => result.expect("first render failed"),
        }
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Act
        shader.set("halves.a", 2.0f32).expect("set a");
        renderer.render(&pass, &probe).expect("third render failed");
        let after_a = renderer
            .read_storage(&shader, "halves")
            .await
            .expect("read after a");
        shader.set("halves.c", 3.0f32).expect("set c");
        renderer
            .render(&pass, &probe)
            .expect("fourth render failed");
        let after_c = renderer
            .read_storage(&shader, "halves")
            .await
            .expect("read after c");

        // Assert
        let (one, two, three) = (0x3C00, 0x4000, 0x4200);
        assert_eq!(
            split_halves(&after_a)[..3],
            [two, one, 0],
            "a is 2, and b went back to its CPU 0 with a, then took one step"
        );
        assert_eq!(
            split_halves(&after_c)[..3],
            [two, two, three],
            "c is 3, and a and b keep what they held"
        );
    });
}

// Story: The app sets the first and the last of five halves. They go up apart, through
// one staging buffer, the last one as a whole word that reaches past the root's ten
// bytes into the buffer's padding. The halves between keep what the pass wrote.
// Runs where the device has SHADER_F16.
#[test]
fn halves_apart_reach_into_the_padding() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(FIFTHS).expect("fifths shader failed");
        let pass = Pass::from_shader("fifths", &shader);
        match renderer.render(&pass, &probe) {
            Err(RendererError::FeatureError { .. }) => {
                eprintln!("the device lacks SHADER_F16, so the test has nothing to check");
                return;
            }
            result => result.expect("first render failed"),
        }
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Act
        shader.set("fifths.a", 2.0f32).expect("set a");
        shader.set("fifths.e", 3.0f32).expect("set e");
        renderer.render(&pass, &probe).expect("third render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "fifths")
            .await
            .expect("read_storage failed");
        let (two, three) = (0x4000, 0x4200);
        assert_eq!(
            split_halves(&bytes),
            [two, 0, three, three, three],
            "a is 2 and e is 3, and c and d took three steps"
        );
    });
}
