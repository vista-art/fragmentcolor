//! Render a single PBR-shaded triangle through Model + Material + Pass.
//!
//! Demonstrates the v0.11.2 higher-level path: build a `Mesh`, wrap it in a
//! `Model` paired with `Material::pbr()`, absorb a `Camera` and a `Light` via
//! `material.add(...)`, then hand the Model to a `Pass` via `add_model`. The
//! Material ships the default Cook-Torrance + GGX physically-based shader.

use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Target, Vertex};

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

        let material = Material::pbr(&renderer)
            .await
            .expect("PBR Material requires shaders-mesh + shaders-material features (default)")
            .base_color([0.85, 0.4, 0.2, 1.0])
            .metallic(0.0)
            .roughness(0.35);

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .look_at([0.0, 0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let sun = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        material.add(&camera).add(&sun);

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
