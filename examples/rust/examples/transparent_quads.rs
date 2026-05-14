//! Three stacked translucent quads, rendered with `alpha_mode: Blend` and
//! the v0.11.2 depth-sort path. The Pass holds Models in deliberately-wrong
//! insertion order (near, middle, far); the renderer sorts them back-to-
//! front using the Camera's view matrix before issuing the draws, so the
//! result looks the same as it would with a hand-sorted scene.

use fragmentcolor::{
    AlphaMode, Camera, Light, Material, Mesh, Model, Renderer, Scene, Target, Vertex,
};

fn main() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([512u32, 512u32])
            .await
            .expect("texture target");

        // Six-vertex quad covering [-0.5, 0.5] in the XY plane, facing +Z.
        let quad = || {
            let mesh = Mesh::new();
            for (pos, uv) in [
                ([-0.5_f32, 0.5, 0.0], [0.0_f32, 1.0]),
                ([-0.5, -0.5, 0.0], [0.0, 0.0]),
                ([0.5, 0.5, 0.0], [1.0, 1.0]),
                ([0.5, 0.5, 0.0], [1.0, 1.0]),
                ([-0.5, -0.5, 0.0], [0.0, 0.0]),
                ([0.5, -0.5, 0.0], [1.0, 0.0]),
            ] {
                mesh.add_vertex(
                    Vertex::new(pos)
                        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                        .set(Vertex::UV0, uv)
                        .set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0])
                        .set(Vertex::UV1, [0.0, 0.0])
                        .set(Vertex::TANGENT, [1.0, 0.0, 0.0, 1.0]),
                );
            }
            mesh
        };

        // Three different-colored glass panels at distinct Z depths. Each
        // shares the PBR shader (single pipeline + single bind group); only
        // the base-color factor and the alpha differ.
        let red_glass = Material::pbr()
            .expect("pbr")
            .base_color([1.0, 0.2, 0.2, 0.45])
            .alpha_mode(AlphaMode::Blend);
        let green_glass = Material::pbr()
            .expect("pbr")
            .base_color([0.2, 1.0, 0.2, 0.45])
            .alpha_mode(AlphaMode::Blend);
        let blue_glass = Material::pbr()
            .expect("pbr")
            .base_color([0.2, 0.4, 1.0, 0.45])
            .alpha_mode(AlphaMode::Blend);

        let red = Model::new(quad(), red_glass);
        red.translate([0.0, 0.0, -1.0]);

        let green = Model::new(quad(), green_glass);
        green.translate([0.0, 0.0, 0.0]);

        let blue = Model::new(quad(), blue_glass);
        blue.translate([0.0, 0.0, 1.0]);

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let sun = Light::directional([0.2, -0.5, -1.0], [1.0, 1.0, 1.0]);

        let scene = Scene::new();
        // Add in wrong order on purpose — the renderer sorts them.
        scene
            .add(&blue)
            .expect("blue")
            .add(&red)
            .expect("red")
            .add(&green)
            .expect("green")
            .add(&camera)
            .expect("camera")
            .add(&sun)
            .expect("sun");

        renderer.render(&scene, &target).expect("render");

        let image = target.get_image().await;
        assert_eq!(image.len(), 512 * 512 * 4);

        // Count pixels where the color is "obviously translucent": any
        // channel above zero but below 250. A fully-opaque clear-color
        // target would be uniformly transparent (all-zero alpha pre-render
        // → blend output keeps the clear color in covered pixels and the
        // glass colors in covered pixels). Print the counts so smoke runs
        // produce a self-documenting line in the build log.
        let mut blended = 0usize;
        let mut nonempty = 0usize;
        for px in image.chunks_exact(4) {
            if px[0] > 0 || px[1] > 0 || px[2] > 0 {
                nonempty += 1;
                if px[0] < 240 && px[1] < 240 && px[2] < 240 {
                    blended += 1;
                }
            }
        }
        println!(
            "rendered {} bytes — {} non-empty pixels, {} blended pixels",
            image.len(),
            nonempty,
            blended,
        );
    });
}
