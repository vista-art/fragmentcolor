use super::*;
use crate::scene::Model;
use crate::{Light, Material, Mesh, Pass, Renderer, Target, Vertex};

const SIZE: u32 = 8;

/// A light color that cancels the 1/π of the toon diffuse, so a fully lit white
/// surface draws white.
const PI_WHITE: [f32; 3] = [std::f32::consts::PI; 3];

/// A quad over the whole target that faces +z, with UV0 from `[0, 0]` at its top left
/// corner to `[1, 1]`.
fn quad() -> Mesh {
    let top_left = ([-1.0, 1.0, 0.0], [0.0, 0.0]);
    let top_right = ([1.0, 1.0, 0.0], [1.0, 0.0]);
    let bottom_left = ([-1.0, -1.0, 0.0], [0.0, 1.0]);
    let bottom_right = ([1.0, -1.0, 0.0], [1.0, 1.0]);
    let mesh = Mesh::new();
    for (position, uv) in [
        top_left,
        bottom_left,
        bottom_right,
        top_left,
        bottom_right,
        top_right,
    ] {
        mesh.add_vertex(Vertex::pbr(position).set(Vertex::UV0, uv));
    }
    mesh
}

/// The quad's back: the same corners wound the other way, with a normal along -z,
/// as the far side of a sheet whose front faces away from the camera.
fn back_quad() -> Mesh {
    let top_left = [-1.0, 1.0, 0.0];
    let top_right = [1.0, 1.0, 0.0];
    let bottom_left = [-1.0, -1.0, 0.0];
    let bottom_right = [1.0, -1.0, 0.0];
    let mesh = Mesh::new();
    for position in [
        top_left,
        bottom_right,
        bottom_left,
        top_left,
        top_right,
        bottom_right,
    ] {
        mesh.add_vertex(Vertex::pbr(position).set(Vertex::NORMAL, [0.0_f32, 0.0, -1.0]));
    }
    mesh
}

/// A toon material seen from `[0, 0, 1]`, in front of the quad, with no ambient light,
/// so each test reads its direct light alone.
fn build_material() -> ToonMaterial {
    let material = ToonMaterial::new();
    let shader = material.shader();
    shader
        .set("camera.position", [0.0_f32, 0.0, 1.0])
        .expect("eye");
    shader.set("lights.ambient", [0.0_f32; 3]).expect("ambient");
    material
}

/// A directional light whose rays reach the quad at the cosine `ndl` to its normal,
/// tilted toward +x.
fn light_at(ndl: f32) -> Light {
    let tilt = (1.0 - ndl * ndl).sqrt();
    Light::directional([-tilt, 0.0, -ndl], PI_WHITE).expect("light")
}

/// Draws the quad with `material` under `light` into a fresh 8-bit target and returns
/// the RGBA of its center pixel. A light lights the shaders it reached until its last
/// holder drops, so the light drops here, and the next draw takes only its own.
async fn draw_center(renderer: &Renderer, material: &ToonMaterial, light: Light) -> [u8; 4] {
    draw_mesh_center(renderer, quad(), material, light).await
}

/// Draws `mesh` with `material` under `light`, as `draw_center` draws the quad.
async fn draw_mesh_center(
    renderer: &Renderer,
    mesh: Mesh,
    material: &ToonMaterial,
    light: Light,
) -> [u8; 4] {
    draw_model_center(renderer, &Model::new(mesh, material), light).await
}

/// Draws `model` under `light`, as `draw_center` draws the quad.
async fn draw_model_center(renderer: &Renderer, model: &Model, light: Light) -> [u8; 4] {
    let pass = Pass::new("toon");
    pass.add(model).expect("add the model");
    pass.add(&light).expect("add the light");
    let target = renderer
        .create_texture_target([SIZE, SIZE])
        .await
        .expect("texture target");
    renderer.render(&pass, &target).expect("render");
    let image = target.get_image().await.expect("read back");
    let start = (((SIZE / 2) * SIZE + SIZE / 2) * 4) as usize;
    [
        image[start],
        image[start + 1],
        image[start + 2],
        image[start + 3],
    ]
}

/// The 8-bit sRGB code of the linear `value`, as the shader encodes it.
fn encode(value: f32) -> u8 {
    let srgb = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).round() as u8
}

#[test]
fn toon_seeds_default_uniforms() {
    let material = ToonMaterial::new();
    let shader = material.shader();

    let base: [f32; 4] = shader.get("material.base_color").expect("read");
    let highlight: [f32; 3] = shader.get("material.highlight").expect("read");
    let size: f32 = shader.get("material.highlight_size").expect("read");
    let rim: [f32; 3] = shader.get("material.rim").expect("read");
    let width: f32 = shader.get("material.rim_width").expect("read");
    let bands: u32 = shader.get("material.bands").expect("read");
    let cutoff: f32 = shader.get("material.alpha_cutoff").expect("read");
    let flag: u32 = shader.get("material.alpha_mode_flag").expect("read");
    let set: u32 = shader.get("material.uv[0].uv_set").expect("read");

    assert_eq!(base, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!((highlight, size), ([0.0; 3], 0.05));
    assert_eq!((rim, width), ([0.0; 3], 0.2));
    assert_eq!((bands, cutoff, flag, set), (3, 0.5, 0, 0));
    assert!(!*material.shader.object.double_sided.read());
}

#[test]
fn setters_update_the_shader_and_share_it_with_clones() {
    let material = ToonMaterial::new();
    let handle = material.clone();

    handle
        .base_color([0.2, 0.4, 0.6, 0.8])
        .expect("base color")
        .bands(5)
        .expect("bands")
        .highlight([1.0, 0.9, 0.8])
        .expect("highlight")
        .highlight_size(0.1)
        .expect("highlight size")
        .rim([0.1, 0.2, 0.3])
        .expect("rim")
        .rim_width(0.4)
        .expect("rim width")
        .alpha_cutoff(0.25)
        .expect("alpha cutoff")
        .alpha_mode(AlphaMode::Mask)
        .double_sided(true);

    let shader = material.shader();
    let base: [f32; 4] = shader.get("material.base_color").expect("read");
    let bands: u32 = shader.get("material.bands").expect("read");
    let highlight: [f32; 3] = shader.get("material.highlight").expect("read");
    let size: f32 = shader.get("material.highlight_size").expect("read");
    let rim: [f32; 3] = shader.get("material.rim").expect("read");
    let width: f32 = shader.get("material.rim_width").expect("read");
    let cutoff: f32 = shader.get("material.alpha_cutoff").expect("read");
    let flag: u32 = shader.get("material.alpha_mode_flag").expect("read");
    assert_eq!((base, bands), ([0.2, 0.4, 0.6, 0.8], 5));
    assert_eq!((highlight, size), ([1.0, 0.9, 0.8], 0.1));
    assert_eq!((rim, width), ([0.1, 0.2, 0.3], 0.4));
    assert_eq!((cutoff, flag), (0.25, AlphaMode::Mask.flag()));
    assert_eq!(*material.shader.object.alpha_mode.read(), AlphaMode::Mask);
    assert!(*material.shader.object.double_sided.read());
}

// Story: a number setter refuses NaN and an infinity with an error that names the
// call and the item, and the material keeps the value it had.
#[test]
fn number_setters_refuse_numbers_that_are_not_finite() {
    let material = ToonMaterial::new();

    let highlight = material.highlight([0.0, f32::NAN, 0.0]);
    let width = material.rim_width(f32::INFINITY);

    assert_eq!(
        highlight.expect_err("NaN").to_string(),
        "ToonMaterial::highlight: item 1 must be finite, got NaN"
    );
    assert_eq!(
        width.expect_err("infinity").to_string(),
        "ToonMaterial::rim_width: value must be finite, got inf"
    );
    let kept: [f32; 3] = material.shader().get("material.highlight").expect("read");
    assert_eq!(kept, [0.0; 3]);
}

// Story: zero bands would divide the light into nothing, so `bands(0)` is an error,
// and the material keeps its bands.
#[test]
fn bands_refuses_zero() {
    let material = ToonMaterial::new().bands(4).expect("bands");

    let error = material.bands(0).expect_err("zero bands");

    assert!(matches!(error, MaterialError::Bands { count: 0, .. }));
    assert_eq!(
        error.to_string(),
        "ToonMaterial::bands takes 1 band or more, got 0"
    );
    let bands: u32 = material.shader().get("material.bands").expect("read");
    assert_eq!(bands, 4);
}

// Story: the ramp is read by light level, so its map refuses a UV set and a transform,
// naming the call, while the base color map takes both.
#[test]
fn ramp_map_refuses_a_uv_set_and_a_transform() {
    let material = ToonMaterial::new();

    let uv = material.ramp_map().uv(1).expect_err("a UV set");
    let transform = material.ramp_map().transform([0.0, 0.0], [2.0, 2.0], 0.0);

    assert!(matches!(uv, MaterialError::NoUv { .. }), "{uv:?}");
    assert!(
        uv.to_string().starts_with("ToonMaterial::ramp_map().uv:"),
        "{uv}"
    );
    let transform = transform.expect_err("a transform").to_string();
    assert!(
        transform.starts_with("ToonMaterial::ramp_map().transform:"),
        "{transform}"
    );
    material
        .base_color_map()
        .uv(1)
        .expect("the base color map's UV set");
    let set: u32 = material
        .shader()
        .get("material.uv[0].uv_set")
        .expect("read");
    assert_eq!(set, 1);
}

#[test]
fn material_enum_holds_a_toon_material() {
    let material = ToonMaterial::new();

    let held = Material::from(&material);

    assert!(matches!(held, Material::Toon(_)));
    let model = Model::new(quad(), &material);
    assert!(matches!(model.material(), Material::Toon(_)));
}

// Story: the light on a face steps up in even bands. At a cosine of 0.6 to the light,
// three bands light the face at 2/3, two light it fully, and a face turned away from
// the light gets none of it.
#[test]
fn diffuse_light_steps_in_bands() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = build_material();

        let three = draw_center(&renderer, &material, light_at(0.6)).await;
        material.bands(2).expect("bands");
        let two = draw_center(&renderer, &material, light_at(0.6)).await;
        let away = draw_center(&renderer, &material, light_at(-1.0)).await;

        let level = encode(2.0 / 3.0);
        assert!(three[0].abs_diff(level) <= 1, "{three:?} vs {level}");
        assert_eq!(three[0], three[2], "{three:?}");
        assert_eq!(two, [255, 255, 255, 255]);
        assert_eq!(away, [0, 0, 0, 255]);
    });
}

// Story: a ramp wider than one texel replaces the bands. Its right half colors the
// faces toward the light and its left half the faces turned away, and a ramp set
// from 8-bit pixels reads them as sRGB.
#[test]
fn ramp_map_replaces_the_bands() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = build_material().bands(1).expect("bands");
        let ramp: Vec<u8> = [[0, 0, 64, 255], [0, 188, 255, 255]].concat();
        material
            .ramp_map()
            .texture((ramp, [2u32, 1u32]))
            .expect("ramp map");

        let lit = draw_center(&renderer, &material, light_at(0.6)).await;
        let away = draw_center(&renderer, &material, light_at(-1.0)).await;

        // A cosine of 0.6 reads the ramp at 0.8, past its right texel's center, and
        // -1 at 0, past its left one's, where the edge clamps.
        assert_eq!((lit[0], lit[2]), (0, 255), "{lit:?}");
        assert!(lit[1].abs_diff(188) <= 1, "{lit:?}");
        assert_eq!((away[0], away[1]), (0, 0), "{away:?}");
        assert!(away[2].abs_diff(64) <= 1, "{away:?}");
    });
}

// Story: the highlight adds its color as a hard spot where the half vector between the
// light and the eye meets the normal, and a size of 0 hides it.
#[test]
fn highlight_adds_a_hard_spot() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = build_material()
            .base_color([1.0, 0.0, 0.0, 1.0])
            .expect("base color")
            .highlight([0.0, 1.0, 0.0])
            .expect("highlight");

        let shown = draw_center(&renderer, &material, light_at(1.0)).await;
        material.highlight_size(0.0).expect("size");
        let hidden = draw_center(&renderer, &material, light_at(1.0)).await;

        assert_eq!(shown, [255, 255, 0, 255]);
        assert_eq!(hidden, [255, 0, 0, 255]);
    });
}

// Story: the rim colors the faces seen edge-on, whatever lights them. Seen head on, the
// quad shows no rim, and seen from far to its side, it shows the rim's color.
#[test]
fn rim_marks_faces_seen_edge_on() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = build_material()
            .base_color([0.0, 0.0, 0.0, 1.0])
            .expect("base color")
            .rim([0.0, 0.0, 1.0])
            .expect("rim");

        let head_on = draw_center(&renderer, &material, light_at(1.0)).await;
        material
            .shader()
            .set("camera.position", [10.0_f32, 0.0, 0.5])
            .expect("eye");
        let edge_on = draw_center(&renderer, &material, light_at(1.0)).await;

        assert_eq!(head_on, [0, 0, 0, 255]);
        assert_eq!(edge_on, [0, 0, 255, 255]);
    });
}

// Story: a double-sided toon sheet seen from behind lights, bands and takes the rim as
// the side the camera sees, since its normal flips on the back face. Unflipped, the
// back would sit in the darkest band and fill with the rim's color.
#[test]
fn a_back_face_lights_as_the_side_the_camera_sees() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let white = build_material().double_sided(true);
        let rimmed = build_material()
            .double_sided(true)
            .base_color([0.0, 0.0, 0.0, 1.0])
            .expect("base color")
            .rim([0.0, 0.0, 1.0])
            .expect("rim");

        let lit = draw_mesh_center(&renderer, back_quad(), &white, light_at(1.0)).await;
        let rim = draw_mesh_center(&renderer, back_quad(), &rimmed, light_at(1.0)).await;

        assert_eq!(
            lit,
            [255, 255, 255, 255],
            "the light reaches the side the camera sees"
        );
        assert_eq!(rim, [0, 0, 0, 255], "the back seen head on shows no rim");
    });
}

// Story: a mirrored double-sided toon model lights its outer faces, which mirroring
// winds clockwise on screen, and seen head on they show no rim. Taken for back faces,
// they would sit in the darkest band and fill with the rim's color.
#[test]
fn a_mirrored_model_lights_its_outer_face() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let white = build_material().double_sided(true);
        let rimmed = build_material()
            .double_sided(true)
            .base_color([0.0, 0.0, 0.0, 1.0])
            .expect("base color")
            .rim([0.0, 0.0, 1.0])
            .expect("rim");
        let mirrored = [Model::new(quad(), &white), Model::new(quad(), &rimmed)];
        for model in &mirrored {
            model.scale([-1.0, 1.0, 1.0]).expect("mirror");
        }

        let lit = draw_model_center(&renderer, &mirrored[0], light_at(1.0)).await;
        let rim = draw_model_center(&renderer, &mirrored[1], light_at(1.0)).await;

        assert_eq!(
            lit,
            [255, 255, 255, 255],
            "the light reaches the side the camera sees"
        );
        assert_eq!(rim, [0, 0, 0, 255], "the face seen head on shows no rim");
    });
}

// Story: on a pass with no Light, the placeholder light from above lights a toon model,
// as it lights a PbrMaterial, so a fresh material doesn't draw near black.
#[test]
fn the_placeholder_lights_a_model_with_no_light() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = build_material();
        // The quad's corners, facing up, toward the placeholder that shines straight down.
        let facing_up = Mesh::new();
        for position in [
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
        ] {
            facing_up.add_vertex(Vertex::pbr(position).set(Vertex::NORMAL, [0.0_f32, 1.0, 0.0]));
        }
        let pass = Pass::new("toon");
        pass.add(&Model::new(facing_up, &material))
            .expect("add the model");
        let target = renderer
            .create_texture_target([SIZE, SIZE])
            .await
            .expect("texture target");

        renderer.render(&pass, &target).expect("render");

        let image = target.get_image().await.expect("read back");
        let start = (((SIZE / 2) * SIZE + SIZE / 2) * 4) as usize;
        assert!(
            image[start] > 100,
            "lit by the placeholder, red {}",
            image[start]
        );
    });
}
