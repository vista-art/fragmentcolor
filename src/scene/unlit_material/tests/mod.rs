use super::*;
use crate::scene::Model;
use crate::{Light, Material, Mesh, Pass, Renderer, Target, Vertex};

const SIZE: u32 = 8;

/// A quad over the whole target, with UV0 from `[0, 0]` at its top left corner to `[1, 1]`.
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

/// Draws `pass` into a fresh 8-bit target and returns the RGBA of its center pixel.
async fn draw_center(renderer: &Renderer, pass: &Pass) -> [u8; 4] {
    let target = renderer
        .create_texture_target([SIZE, SIZE])
        .await
        .expect("texture target");
    renderer.render(pass, &target).expect("render");
    let image = target.get_image().await.expect("read back");
    let start = (((SIZE / 2) * SIZE + SIZE / 2) * 4) as usize;
    [
        image[start],
        image[start + 1],
        image[start + 2],
        image[start + 3],
    ]
}

/// A pass that draws the full-screen quad with `material`.
fn pass_with(material: &UnlitMaterial) -> Pass {
    let pass = Pass::new("unlit");
    pass.add(&Model::new(quad(), material))
        .expect("add the model");
    pass
}

#[test]
fn unlit_seeds_default_uniforms() {
    let material = UnlitMaterial::new();

    let base: [f32; 4] = material.shader().get("material.base_color").expect("read");
    let cutoff: f32 = material
        .shader()
        .get("material.alpha_cutoff")
        .expect("read");
    let flag: u32 = material
        .shader()
        .get("material.alpha_mode_flag")
        .expect("read");
    let set: u32 = material
        .shader()
        .get("material.uv[0].uv_set")
        .expect("read");
    let scale: [f32; 2] = material.shader().get("material.uv[0].scale").expect("read");

    assert_eq!(base, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!((cutoff, flag, set, scale), (0.5, 0, 0, [1.0, 1.0]));
    assert!(!*material.shader.object.double_sided.read());
}

#[test]
fn setters_update_the_shader_and_share_it_with_clones() {
    let material = UnlitMaterial::new();
    let handle = material.clone();

    handle
        .base_color([0.2, 0.4, 0.6, 0.8])
        .expect("base color")
        .alpha_cutoff(0.25)
        .expect("alpha cutoff")
        .alpha_mode(AlphaMode::Mask)
        .double_sided(true);

    let base: [f32; 4] = material.shader().get("material.base_color").expect("read");
    let cutoff: f32 = material
        .shader()
        .get("material.alpha_cutoff")
        .expect("read");
    let flag: u32 = material
        .shader()
        .get("material.alpha_mode_flag")
        .expect("read");
    assert_eq!(base, [0.2, 0.4, 0.6, 0.8]);
    assert_eq!((cutoff, flag), (0.25, AlphaMode::Mask.flag()));
    assert_eq!(*material.shader.object.alpha_mode.read(), AlphaMode::Mask);
    assert!(*material.shader.object.double_sided.read());
}

// Story: a number setter refuses NaN and an infinity with an error that names the
// call and the item, and the material keeps the value it had.
#[test]
fn number_setters_refuse_numbers_that_are_not_finite() {
    let material = UnlitMaterial::new()
        .base_color([0.5, 0.5, 0.5, 1.0])
        .expect("base color");

    let color = material.base_color([0.0, f32::NAN, 0.0, 1.0]);
    let cutoff = material.alpha_cutoff(f32::INFINITY);

    assert_eq!(
        color.expect_err("NaN").to_string(),
        "UnlitMaterial::base_color: item 1 must be finite, got NaN"
    );
    assert_eq!(
        cutoff.expect_err("infinity").to_string(),
        "UnlitMaterial::alpha_cutoff: value must be finite, got inf"
    );
    let base: [f32; 4] = material.shader().get("material.base_color").expect("read");
    assert_eq!(base, [0.5, 0.5, 0.5, 1.0]);
}

// Story: the map an unlit material returns names the unlit material in its errors.
#[test]
fn map_errors_name_the_unlit_material() {
    let material = UnlitMaterial::new();

    let error = material.base_color_map().uv(2).expect_err("a third UV set");

    assert!(
        error
            .to_string()
            .contains("UnlitMaterial::base_color_map().uv"),
        "{error}"
    );
    let set: u32 = material
        .shader()
        .get("material.uv[0].uv_set")
        .expect("read");
    assert_eq!(set, 0);
}

#[test]
fn material_enum_holds_an_unlit_material() {
    let material = UnlitMaterial::new();

    let held = Material::from(&material);

    assert!(matches!(held, Material::Unlit(_)));
    let model = Model::new(quad(), &material);
    assert!(matches!(model.material(), Material::Unlit(_)));
}

// Story: an unlit material draws its base color as it is. A light that would light a PBR
// material changes nothing, and the 8-bit target gets the color encoded as sRGB.
#[test]
fn draws_the_base_color_with_no_lighting() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = UnlitMaterial::new()
            .base_color([1.0, 0.5, 0.0, 1.0])
            .expect("base color");
        let pass = pass_with(&material);
        let dark = Light::directional([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]).expect("light");
        pass.add(&dark).expect("add a black light");

        let pixel = draw_center(&renderer, &pass).await;

        // linear 0.5 encodes to sRGB 0.7354, which rounds to 188.
        let [r, g, b, a] = pixel;
        assert_eq!((r, b, a), (255, 0, 255), "{pixel:?}");
        assert!(g.abs_diff(188) <= 1, "{pixel:?}");
    });
}

// Story: the base color map multiplies the base color, and a map set from 8-bit pixels
// with no format reads them as sRGB, as glTF stores base color.
#[test]
fn base_color_map_multiplies_the_base_color() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let pixels: Vec<u8> = [[0, 188, 255, 255]; 4].concat();
        let material = UnlitMaterial::new()
            .base_color([1.0, 1.0, 0.5, 1.0])
            .expect("base color");
        material
            .base_color_map()
            .texture((pixels, [2u32, 2u32]))
            .expect("base color map");

        let pixel = draw_center(&renderer, &pass_with(&material)).await;

        // sRGB 188 reads as linear 0.5 and encodes back to 188, and 255 times 0.5 is
        // linear 0.5 too.
        let [r, g, b, _] = pixel;
        assert_eq!(r, 0, "{pixel:?}");
        assert!(g.abs_diff(188) <= 1, "{pixel:?}");
        assert!(b.abs_diff(188) <= 1, "{pixel:?}");
    });
}

// Story: in Mask mode, a fragment whose alpha falls under the cut-off is discarded, so
// the clear color shows through, and one at or over it draws.
#[test]
fn mask_mode_discards_below_the_cutoff() {
    pollster::block_on(async {
        let renderer = Renderer::new();
        let material = UnlitMaterial::new()
            .base_color([0.0, 1.0, 0.0, 0.4])
            .expect("base color")
            .alpha_mode(AlphaMode::Mask);

        let hidden = draw_center(&renderer, &pass_with(&material)).await;
        material.alpha_cutoff(0.3).expect("cutoff");
        let shown = draw_center(&renderer, &pass_with(&material)).await;

        assert_eq!(hidden[1], 0, "{hidden:?}");
        assert_eq!(shown[1], 255, "{shown:?}");
    });
}
