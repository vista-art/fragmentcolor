//! Render a single PBR-shaded triangle through Model + Material + Pass.
//!
//! Demonstrates the v0.11.2 higher-level path: build a `Mesh`, wrap it in a
//! `Model` paired with `Material::pbr()`, position the Model with a world-space
//! translation, then hand the Model to a `Pass` via `add_model`. The Material
//! ships the default Cook-Torrance + GGX physically-based shader; the camera
//! and light are baseline uniforms you can override directly on the underlying
//! Shader.

use fragmentcolor::{Material, Mesh, Model, Pass, Renderer, Target, Vertex};

fn main() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([256u32, 256u32])
            .await
            .expect("texture target");

        // A triangle in the world-space XY plane, facing +Z. The PBR shader
        // expects position (vec3), normal (vec3), uv0 (vec2) in this order.
        let mesh = Mesh::new();
        for (pos, uv) in [
            ([0.0_f32, 0.5, 0.0], [0.5, 1.0]),
            ([-0.5, -0.5, 0.0], [0.0, 0.0]),
            ([0.5, -0.5, 0.0], [1.0, 0.0]),
        ] {
            mesh.add_vertex(
                Vertex::new(pos)
                    .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
                    .set(Vertex::UV0, uv),
            );
        }

        let material = Material::pbr()
            .base_color([0.85, 0.4, 0.2, 1.0])
            .metallic(0.0)
            .roughness(0.35);

        // Camera + light overrides (Material::pbr's defaults render to NDC under
        // an identity view-projection; that's fine for this clip-space triangle).
        material
            .shader()
            .set("camera.position", [0.0_f32, 0.0, 2.0])
            .ok();
        material
            .shader()
            .set("light.direction", [0.3_f32, -1.0, -0.4])
            .ok();

        let model = Model::new(mesh, material);
        // Nudge it slightly to the right to show the per-Model transform path.
        model.translate([0.1, 0.0, 0.0]);

        let pass = Pass::new("triangle");
        pass.add_model(&model).expect("add_model");
        renderer.render(&pass, &target).expect("render");

        let image = target.get_image().await;
        assert_eq!(image.len(), 256 * 256 * 4);
        println!(
            "rendered {} bytes — first pixel: ({}, {}, {}, {})",
            image.len(),
            image[0],
            image[1],
            image[2],
            image[3],
        );
    });
}
