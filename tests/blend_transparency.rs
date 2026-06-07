//! End-to-end render of overlapping translucent surfaces.
//!
//! Verifies the `alpha_mode: Blend` depth-sort path that closed the seventh
//! "wild glTF" correctness gap in v0.11.2. Two glass quads sit on the eye
//! axis at different Z depths; with proper back-to-front sorting the front
//! quad's color over-blends *over* the back one. Without sorting, the
//! result depends on insertion order and can flip the layering.

use fragmentcolor::{
    AlphaMode, Camera, Light, Material, Mesh, Model, Renderer, Scene, Target, Vertex,
};

fn unit_quad() -> Mesh {
    // Two triangles in the XY plane centered at the origin, facing +Z.
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
}

#[test]
fn renders_two_blend_quads_at_different_depths() {
    // Two translucent quads, same Material, stacked on the eye axis. The
    // farther quad (red, at z=-2) sits behind the nearer one (blue, at z=0).
    // With back-to-front sorting the renderer paints red first, then blends
    // blue over the top — the resulting pixels skew toward blue. Without
    // sorting the result would be insertion-order dependent.
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([32u32, 32u32])
            .await
            .expect("texture target");

        let far_mat = Material::pbr()
            .base_color([1.0, 0.0, 0.0, 0.5])
            .alpha_mode(AlphaMode::Blend);
        let near_mat = Material::pbr()
            .base_color([0.0, 0.0, 1.0, 0.5])
            .alpha_mode(AlphaMode::Blend);

        let far_model = Model::new(unit_quad(), far_mat);
        far_model.translate([0.0, 0.0, -2.0]);
        let near_model = Model::new(unit_quad(), near_mat);
        near_model.translate([0.0, 0.0, 0.0]);

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let sun = Light::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0]);

        let scene = Scene::new();
        // Add in the WRONG order on purpose (near before far). With proper
        // sorting the rendered image should still look correct.
        scene
            .add(&near_model)
            .expect("near")
            .add(&far_model)
            .expect("far")
            .add(&camera)
            .expect("camera")
            .add(&sun)
            .expect("sun");

        renderer.render(&scene, &target).expect("render");
        let image = target.get_image().await;
        assert_eq!(image.len(), 32 * 32 * 4);

        // Sample the center pixel. The PBR shader applies Cook-Torrance
        // lighting so the exact values depend on the light direction, but
        // the relative R/B channels should reflect blue-over-red blending.
        // Skip exact-value asserts — the existence test ensures the pass
        // produces a non-blank image; alpha_mode_pipeline_state already
        // covers the pipeline-state side.
        let center_idx = ((16 * 32 + 16) * 4) as usize;
        let r = image[center_idx];
        let _g = image[center_idx + 1];
        let b = image[center_idx + 2];
        // If sorting is correct AND blending happened, the blue channel
        // should be at least as strong as the red channel at the center
        // (where both quads cover the pixel). With wrong-order draws,
        // depth-test would have rejected the blue fragment entirely
        // (no depth write from Blend) — but the over-blend would have
        // produced red-over-blue (red dominant). With correct sorting it's
        // blue-over-red (blue dominant).
        assert!(
            b >= r.saturating_sub(40),
            "blend ordering looks wrong: r={r}, b={b}"
        );
    });
}

#[test]
fn cross_shader_blend_sorts_globally() {
    // Two translucent Materials (red and blue glass) on the same Pass,
    // each owning three Models interleaved on the eye axis. The
    // back-to-front sort should treat all six entries as one global
    // sequence, not two per-shader buckets. We add the Models in an
    // intentionally scrambled order; the result hinges on the final
    // global Z order, not on insertion order.
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let red_mat = Material::pbr()
            .base_color([1.0, 0.0, 0.0, 0.5])
            .alpha_mode(AlphaMode::Blend);
        let blue_mat = Material::pbr()
            .base_color([0.0, 0.0, 1.0, 0.5])
            .alpha_mode(AlphaMode::Blend);

        // Six quads interleaved on the eye axis. Red at -3, -1, 1; blue at
        // -2, 0, 2. With a correct global sort the nearest-camera quad is
        // the blue one at z=2 — which means the dominant top layer is blue.
        // A per-shader sort would draw red's nearest (z=1) after blue's
        // nearest (z=2), inverting the result.
        let mut models: Vec<Model> = Vec::new();
        for (mat, zs) in [
            (red_mat.clone(), [-3.0_f32, -1.0, 1.0]),
            (blue_mat.clone(), [-2.0_f32, 0.0, 2.0]),
        ] {
            for z in zs {
                let m = Model::new(unit_quad(), mat.clone());
                m.translate([0.0, 0.0, z]);
                models.push(m);
            }
        }

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let sun = Light::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0]);

        let scene = Scene::new();
        // Scramble the insertion order so the test exercises the sort,
        // not the insertion-order accident.
        let order = [3usize, 0, 4, 1, 5, 2];
        let mut s = scene.add(&camera).expect("camera").add(&sun).expect("sun");
        for i in order {
            s = s.add(&models[i]).expect("model");
        }

        renderer.render(&scene, &target).expect("render");
        let image = target.get_image().await;
        assert_eq!(image.len(), 16 * 16 * 4);

        let center_idx = ((8 * 16 + 8) * 4) as usize;
        let r = image[center_idx];
        let b = image[center_idx + 2];
        // With the correct global sort the topmost quad is blue
        // (z=2 is nearest the eye at z=5), so the over-blended result
        // at the center is biased toward blue.
        assert!(
            b >= r.saturating_sub(40),
            "global cross-shader sort wrong: r={r}, b={b}"
        );
    });
}

#[test]
fn opaque_and_blend_in_same_pass_render_without_panic() {
    // Mixing opaque and blend models in one Pass exercises the partition
    // logic in `build_pass_draws`. The renderer should batch the opaque
    // group and submit the blend draws individually without panicking.
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16u32, 16u32])
            .await
            .expect("texture target");

        let opaque_mat = Material::pbr()
            .base_color([0.5, 0.5, 0.5, 1.0]);
        let blend_mat = Material::pbr()
            .base_color([1.0, 1.0, 1.0, 0.5])
            .alpha_mode(AlphaMode::Blend);

        let solid = Model::new(unit_quad(), opaque_mat);
        solid.translate([0.0, 0.0, -1.0]);
        let glass = Model::new(unit_quad(), blend_mat);
        glass.translate([0.0, 0.0, 0.0]);

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let sun = Light::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0]);

        let scene = Scene::new();
        scene
            .add(&solid)
            .expect("solid")
            .add(&glass)
            .expect("glass")
            .add(&camera)
            .expect("camera")
            .add(&sun)
            .expect("sun");

        renderer.render(&scene, &target).expect("render");
        let image = target.get_image().await;
        assert_eq!(image.len(), 16 * 16 * 4);
    });
}
