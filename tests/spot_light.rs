#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Target, Vertex};

const SIZE: u32 = 16;

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

/// Render the wall under a spot light two meters away with the given cone.
async fn render(inner: f32, outer: f32) -> Vec<u8> {
    let renderer = Renderer::new();
    let target = renderer
        .create_texture_target([SIZE, SIZE])
        .await
        .expect("texture target");
    let camera = Camera::perspective(1.0, 1.0, 0.1, 100.0)
        .expect("camera")
        .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .expect("camera");
    let spot = Light::spot([0.0, 0.0, 2.0], [0.0, 0.0, -1.0], [1.0, 1.0, 1.0])
        .set_intensity(20.0)
        .set_cone_angles(inner, outer)
        .expect("cone angles");
    let pass = Pass::new("spot");
    pass.add(&wall()).expect("wall");
    pass.add(&camera).expect("camera");
    pass.add(&spot).expect("spot");
    renderer.render(&pass, &target).expect("render");
    target.get_image().await.expect("read back")
}

/// The red channel of the pixel at `(x, y)`.
fn red(image: &[u8], x: u32, y: u32) -> u8 {
    image[((y * SIZE + x) * 4) as usize]
}

#[test]
fn equal_cone_angles_draw_a_hard_edged_spot() {
    pollster::block_on(async {
        // A 0.5 radian cone reaches about one meter across the wall, a few
        // pixels around the center. The corners sit far outside it.
        let image = render(0.5, 0.5).await;
        let center = red(&image, SIZE / 2, SIZE / 2);
        let corner = red(&image, 0, 0);
        assert!(
            center > 200,
            "the cone's center should be lit, got {center}"
        );
        assert!(
            center > corner.saturating_add(100),
            "outside the cone should stay dark, got center {center} and corner {corner}"
        );
    });
}
