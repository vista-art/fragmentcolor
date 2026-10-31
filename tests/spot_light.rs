#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Target, Vertex};

/// A four-meter white square at the origin, facing the camera on +Z.
fn wall() -> Model {
    let mesh = Mesh::new();
    let corners = [
        [-2.0, -2.0, 0.0],
        [2.0, -2.0, 0.0],
        [2.0, 2.0, 0.0],
        [-2.0, -2.0, 0.0],
        [2.0, 2.0, 0.0],
        [-2.0, 2.0, 0.0],
    ];
    mesh.add_vertices(
        corners.map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, [0.0, 0.0, 1.0])),
    );
    let material = Material::pbr()
        .base_color([1.0, 1.0, 1.0, 1.0])
        .roughness(1.0)
        .metallic(0.0);
    Model::new(mesh, material)
}

/// A spot light two meters in front of the wall, aimed at its center.
struct Spot {
    intensity: f32,
    inner: f32,
    outer: f32,
}

/// Render the wall at `size` × `size` pixels under `spot`, seen from five
/// meters away, and return the red channel of every pixel.
async fn render(size: u32, spot: Spot) -> Vec<u8> {
    let renderer = Renderer::new();
    let target = renderer
        .create_texture_target([size, size])
        .await
        .expect("texture target");
    let camera = Camera::perspective(1.0, 1.0, 0.1, 100.0)
        .expect("camera")
        .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .expect("camera");
    let light = Light::spot([0.0, 0.0, 2.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0])
        .set_intensity(spot.intensity)
        .set_cone_angles(spot.inner, spot.outer)
        .expect("cone angles");
    let pass = Pass::new("spot");
    pass.add(&wall()).expect("wall");
    pass.add(&camera).expect("camera");
    pass.add(&light).expect("spot");
    renderer.render(&pass, &target).expect("render");
    let image = target.get_image().await.expect("read back");
    image
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[0])
        .collect()
}

#[test]
fn equal_cone_angles_draw_a_hard_edged_spot() {
    pollster::block_on(async {
        // The wall spans pixels 2 to 13 of 16, and a 0.5 radian cone lights
        // about 1.1 meters of it around the center. Pixel (3, 8) sits on the
        // wall about 1.5 meters from the axis, outside the cone, where only
        // the ambient term lights it (about 48 once encoded as sRGB).
        let size = 16;
        let red = render(
            size,
            Spot {
                intensity: 20.0,
                inner: 0.5,
                outer: 0.5,
            },
        )
        .await;
        let at = |x: u32, y: u32| red[(y * size + x) as usize];
        let center = at(size / 2, size / 2);
        let outside = at(3, 8);
        assert!(
            center > 200,
            "the cone's center should be lit, got {center}"
        );
        assert!(
            outside < 100,
            "the wall outside the cone should stay dark, got {outside} beside a center of {center}"
        );
    });
}

#[test]
fn a_narrow_cone_lights_its_axis_fully() {
    pollster::block_on(async {
        // An odd size puts the middle pixel's center on the spot's axis, and an
        // intensity of 4 keeps that pixel below white, so a dimmer axis shows.
        let size = 15;
        let middle = (size / 2 * size + size / 2) as usize;
        let axis = |red: Vec<u8>| red[middle];
        let wide = axis(
            render(
                size,
                Spot {
                    intensity: 4.0,
                    inner: 0.2,
                    outer: 0.5,
                },
            )
            .await,
        );
        for (inner, outer) in [(0.0005, 0.001), (0.001, 0.001)] {
            let narrow = axis(
                render(
                    size,
                    Spot {
                        intensity: 4.0,
                        inner,
                        outer,
                    },
                )
                .await,
            );
            assert!(
                narrow.abs_diff(wide) <= 2,
                "cone ({inner}, {outer}) should light its axis like a wide one: {narrow} against {wide}"
            );
        }
    });
}
