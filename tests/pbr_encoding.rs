#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{
    Material, Mesh, Model, Pass, Renderer, Target, TextureData, TextureFormat, TextureInput,
    TextureOptions, TextureUsage, Vertex,
};

// A black surface that only glows, so every pixel holds the emissive light.
const GLOW: f32 = 0.2;

/// One triangle that covers clip space, facing the identity camera.
fn cover() -> Model {
    let mesh = Mesh::new();
    mesh.add_vertices(
        [[-1.0, -1.0, 0.0], [3.0, -1.0, 0.0], [-1.0, 3.0, 0.0]]
            .map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    let material = Material::pbr()
        .base_color([0.0, 0.0, 0.0, 1.0])
        .emissive([GLOW, GLOW, GLOW]);
    Model::new(mesh, material)
}

/// `GLOW` encoded as sRGB, in 8-bit steps.
fn encoded() -> f32 {
    (1.055 * GLOW.powf(1.0 / 2.4) - 0.055) * 255.0
}

fn half_to_float(bits: u16) -> f32 {
    let exponent = i32::from((bits >> 10) & 0x1f);
    let mantissa = f32::from(bits & 0x3ff);
    let magnitude = match exponent {
        0 => mantissa * 2f32.powi(-24),
        _ => (1.0 + mantissa / 1024.0) * 2f32.powi(exponent - 15),
    };
    if bits & 0x8000 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

async fn render_into(renderer: &Renderer, format: TextureFormat) -> Vec<u8> {
    let usage = TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC;
    let options = TextureOptions {
        size: Some([4, 4].into()),
        format,
        ..Default::default()
    }
    .with_usage(usage);
    let input = TextureInput {
        data: TextureData::Empty,
        options,
    };
    let texture = renderer
        .create_storage_texture(input)
        .await
        .expect("create_storage_texture failed");
    let pass = Pass::new("glow");
    pass.add(&cover()).expect("add failed");
    pass.set_target(&texture).expect("set_target failed");
    let frame = renderer
        .create_texture_target([4, 4])
        .await
        .expect("create_texture_target failed");
    renderer.render(&pass, &frame).expect("render failed");
    renderer
        .read_texture(texture.id())
        .await
        .expect("read_texture failed")
}

// Story: PBR light reaches every kind of target once, encoded for display
// where the target stores display values and linear where it keeps light.
#[test]
fn pbr_encodes_for_the_target_format() {
    pollster::block_on(async move {
        let renderer = Renderer::new();

        // A texture target stores display values: the shader encodes.
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("create_texture_target failed");
        let pass = Pass::new("glow");
        pass.add(&cover()).expect("add failed");
        renderer.render(&pass, &target).expect("render failed");
        let image = target.get_image().await.expect("read back");
        assert!(
            (f32::from(image[0]) - encoded()).abs() <= 2.0,
            "Rgba8Unorm holds {}, expected about {}",
            image[0],
            encoded()
        );

        // An sRGB format encodes on write: the shader writes linear light, once.
        let srgb = render_into(&renderer, TextureFormat::Rgba8UnormSrgb).await;
        assert!(
            (f32::from(srgb[0]) - encoded()).abs() <= 2.0,
            "Rgba8UnormSrgb holds {}, expected about {}",
            srgb[0],
            encoded()
        );

        // A float format keeps linear light for the passes after it.
        let float = render_into(&renderer, TextureFormat::Rgba16Float).await;
        let red = half_to_float(u16::from_le_bytes([float[0], float[1]]));
        assert!(
            (red - GLOW).abs() <= 0.005,
            "Rgba16Float holds {red}, expected {GLOW}"
        );
    });
}
