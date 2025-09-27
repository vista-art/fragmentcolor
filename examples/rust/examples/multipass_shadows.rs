use fragmentcolor::{Frame, Pass, Renderer, Shader, Target};

const CIRCLE_SOURCE: &str = include_str!("circle.wgsl");

// Two-pass cast-shadow approximation rendered offscreen:
// 1) Shadow pass: draw a dark, offset circle; clear to transparent
// 2) Main pass: load previous and draw the colored circle on top
fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async move {
        // Renderer + headless target
        let renderer = Renderer::new();
        let size = [256u32, 256u32];
        let target = renderer.create_texture_target(size).await?;

        // Shared uniforms
        let res = [size[0] as f32, size[1] as f32];
        // Circle coordinates are pixel-like but centered at (0,0) in UV space after division by resolution
        // So the center is [0,0], and small offsets are in pixels
        let center = [0.0f32, 0.0f32];
        let radius = 60.0f32;
        let border = 2.0f32;

        // Shadow shader (dark, slightly offset)
        let shadow = Shader::new(CIRCLE_SOURCE)?;
        shadow.set("resolution", res)?;
        shadow.set("circle.position", [center[0] + 12.0, center[1] - 12.0])?; // slight offset for the shadow
        shadow.set("circle.radius", radius + 2.0)?; // slightly larger
        shadow.set("circle.border", border)?;
        shadow.set("circle.color", [0.0, 0.0, 0.0, 0.45])?; // semi-transparent black

        // Main shader (colored circle at center)
        let main = Shader::new(CIRCLE_SOURCE)?;
        main.set("resolution", res)?;
        main.set("circle.position", center)?;
        main.set("circle.radius", radius)?;
        main.set("circle.border", border)?;
        main.set("circle.color", [0.2, 0.7, 1.0, 1.0])?; // cyan-ish

        // Passes
        let pass_shadow = {
            let p = Pass::from_shader("shadow", &shadow);
            p.set_clear_color([0.0, 0.0, 0.0, 0.0]); // transparent background
            p
        };
        let pass_main = {
            let p = Pass::from_shader("main", &main);
            p.load_previous(); // load shadow pass result
            p
        };

        // Compose and render
        let mut frame = Frame::new();
        frame.add_pass(&pass_shadow);
        frame.add_pass(&pass_main);

        renderer.render(&frame, &target)?;

        // Quick check
        let image = target.get_image();
        assert_eq!(image.len(), (size[0] * size[1] * 4) as usize);
        Ok(())
    })
}
