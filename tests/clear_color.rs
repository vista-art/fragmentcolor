#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{
    Pass, Renderer, Shader, TextureData, TextureFormat, TextureInput, TextureOptions, TextureUsage,
};

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

// Story: a float target clears to the exact color the pass sets, values
// above 1 and below 0 included, where every channel was clamped to 0..1.
#[test]
fn float_target_keeps_the_clear_color() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let options = TextureOptions {
            size: Some([4, 4].into()),
            format: TextureFormat::Rgba16Float,
            ..Default::default()
        }
        .with_usage(TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC);
        let input = TextureInput {
            data: TextureData::Empty,
            options,
        };
        let texture = renderer
            .create_storage_texture(input)
            .await
            .expect("create_storage_texture failed");

        // The shader draws only pixel (0, 0), so pixel (1, 0) holds the clear color.
        let pass = Pass::from_shader("clear", &Shader::default());
        pass.set_viewport([0, 0, 1, 1])
            .expect("set_viewport failed");
        pass.set_clear_color([4.0, -1.0, 0.25, 1.0])
            .expect("set_clear_color failed");
        pass.set_target(&texture).expect("set_target failed");
        let frame = renderer
            .create_texture_target([4, 4])
            .await
            .expect("create_texture_target failed");
        renderer.render(&pass, &frame).expect("render failed");

        let bytes = renderer
            .read_texture(texture.id())
            .await
            .expect("read_texture failed");
        let (halves, _) = bytes[8..16].as_chunks::<2>();
        let pixel: Vec<f32> = halves
            .iter()
            .map(|half| half_to_float(u16::from_le_bytes(*half)))
            .collect();
        assert_eq!(pixel, [4.0, -1.0, 0.25, 1.0]);
    });
}
