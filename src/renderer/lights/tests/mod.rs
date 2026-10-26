//! GPU tests for clustered lights. Each one renders the same scene on fresh
//! renderers: clustered, with the grid turned off as the reference, or on the
//! uniform path that a device without fragment storage takes. A mesh keeps
//! the buffers of the device that drew it first, so every renderer gets a
//! pass built for it.

use super::grid::GRID_MARGIN;
use super::*;
use crate::{Camera, Light, Material, Mesh, Model, Pass, Renderer, Shader, Target, Vertex};
use std::sync::atomic::Ordering;

/// How a test renderer hands lights to its shaders.
#[derive(Debug, Clone, Copy)]
enum Mode {
    Clustered,
    /// Clustered buffers with no grid, so every fragment loops over every light.
    Reference,
    /// The uniform path with a block of this many lights.
    Uniform(u32),
    /// The clustered path with this storage binding limit.
    Limit(u64),
    /// The reference at this storage binding limit.
    LimitReference(u64),
}

struct Drawn {
    image: Vec<u8>,
    context: Arc<RenderContext>,
}

async fn open(mode: Mode, size: u32) -> (Renderer, crate::TextureTarget, Arc<RenderContext>) {
    let renderer = Renderer::new();
    let target = renderer
        .create_texture_target([size, size])
        .await
        .expect("texture target");
    let context = renderer.context(None).await.expect("context");
    match mode {
        Mode::Clustered => {}
        Mode::Reference => context.light_probe.no_grid.store(true, Ordering::Relaxed),
        Mode::Uniform(count) => {
            *context.light_path.write() = LightPath::Uniform { count };
            crate::shader::lower_test_cap(count);
        }
        Mode::Limit(limit) => *context.light_path.write() = LightPath::Clustered { limit },
        Mode::LimitReference(limit) => {
            *context.light_path.write() = LightPath::Clustered { limit };
            context.light_probe.no_grid.store(true, Ordering::Relaxed);
        }
    }
    (renderer, target, context)
}

/// Renders the pass `build` makes on a fresh renderer in `mode`.
async fn draw(build: impl Fn() -> Pass, mode: Mode, size: u32) -> Drawn {
    let (renderer, target, context) = open(mode, size).await;
    renderer.render(&build(), &target).expect("render");
    let image = target.get_image().await.expect("read back");
    Drawn { image, context }
}

/// Every channel within 1, and a mean difference of at most 0.05.
fn assert_close(found: &[u8], expected: &[u8], label: &str) {
    assert_eq!(found.len(), expected.len(), "{label}: sizes differ");
    let mut largest = 0u8;
    let mut total = 0u64;
    for (a, b) in found.iter().zip(expected) {
        let difference = a.abs_diff(*b);
        largest = largest.max(difference);
        total += u64::from(difference);
    }
    let mean = total as f64 / found.len() as f64;
    assert!(
        largest <= 1 && mean <= 0.05,
        "{label}: largest channel difference {largest}, mean {mean:.4}"
    );
}

/// The image is neither black nor washed out, so a comparison means something.
fn assert_varied(image: &[u8], label: &str) {
    let reds: Vec<u8> = image
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[0])
        .collect();
    let mean = reds.iter().map(|red| f64::from(*red)).sum::<f64>() / reds.len() as f64;
    let lit = reds.iter().filter(|red| (40..250).contains(*red)).count();
    assert!(
        (20.0..230.0).contains(&mean) && lit > reds.len() / 8,
        "{label}: mean red {mean:.1}, {lit} mid-tone pixels"
    );
}

/// The number of binning dispatches a context recorded.
fn dispatches(context: &RenderContext) -> u64 {
    context.light_probe.dispatches.load(Ordering::Relaxed)
}

/// The number of light set prepares a context ran.
fn prepares(context: &RenderContext) -> u64 {
    context.light_probe.prepares.load(Ordering::Relaxed)
}

/// The bytes a context wrote to light buffers.
fn uploads(context: &RenderContext) -> u64 {
    context.light_probe.uploads.load(Ordering::Relaxed)
}

/// The number of light sets of the context that bin with a grid.
fn grids(context: &RenderContext) -> usize {
    context
        .light_sets
        .read()
        .values()
        .filter(|set| set.binned.is_some())
        .count()
}

/// A deterministic generator, so the scenes repeat run to run.
struct Seeds(u64);

impl Seeds {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }

    fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }

    fn color(&mut self) -> [f32; 3] {
        [
            self.between(0.2, 1.0),
            self.between(0.2, 1.0),
            self.between(0.2, 1.0),
        ]
    }
}

/// A quad over `corners` in counterclockwise order around `normal`.
fn quad(corners: [[f32; 3]; 4], normal: [f32; 3]) -> Mesh {
    let mesh = Mesh::new();
    let [a, b, c, d] = corners;
    mesh.add_vertices(
        [a, b, c, a, c, d].map(|corner| Vertex::pbr(corner).set(Vertex::NORMAL, normal)),
    );
    mesh
}

fn rough() -> Material {
    Material::pbr()
        .base_color([1.0, 1.0, 1.0, 1.0])
        .roughness(0.8)
        .metallic(0.0)
}

/// A floor at y = 0 and a wall at z = -6, both 12 meters wide.
fn floor_and_wall(material: &Material) -> [Model; 2] {
    let floor = quad(
        [
            [-6.0, 0.0, -6.0],
            [-6.0, 0.0, 6.0],
            [6.0, 0.0, 6.0],
            [6.0, 0.0, -6.0],
        ],
        [0.0, 1.0, 0.0],
    );
    let wall = quad(
        [
            [-6.0, 0.0, -6.0],
            [6.0, 0.0, -6.0],
            [6.0, 6.0, -6.0],
            [-6.0, 6.0, -6.0],
        ],
        [0.0, 0.0, 1.0],
    );
    [
        Model::new(floor, material.clone()),
        Model::new(wall, material.clone()),
    ]
}

fn camera() -> Camera {
    Camera::perspective(1.0, 1.0, 0.1, 100.0)
        .expect("camera")
        .look_at([0.0, 4.0, 9.0], [0.0, 1.0, -2.0], [0.0, 1.0, 0.0])
        .expect("camera")
}

/// A pass with the floor, the wall, `camera` and `lights`.
fn lit_pass(name: &str, camera: Option<&Camera>, lights: &[Light]) -> Pass {
    let pass = Pass::new(name);
    for model in floor_and_wall(&rough()) {
        pass.add(&model).expect("model");
    }
    if let Some(camera) = camera {
        pass.add(camera).expect("camera");
    }
    for light in lights {
        pass.add(light).expect("light");
    }
    pass
}

/// A point light at `position` with `range`.
fn bulb(position: [f32; 3], color: [f32; 3], intensity: f32, range: f32) -> Light {
    let light = Light::point(position, color).set_intensity(intensity);
    light.set_range(range).expect("range");
    light
}

/// 600 seeded lights: 512 points with ranges from 0.5 to 4, 64 spots, 16
/// points without a range and 8 directional lights.
fn seeded_lights(seeds: &mut Seeds) -> Vec<Light> {
    let place = |seeds: &mut Seeds| {
        [
            seeds.between(-6.0, 6.0),
            seeds.between(0.2, 4.0),
            seeds.between(-6.0, 6.0),
        ]
    };
    let mut lights = Vec::new();
    for _ in 0..512 {
        let (position, color) = (place(seeds), seeds.color());
        let intensity = seeds.between(0.05, 0.4);
        lights.push(bulb(position, color, intensity, seeds.between(0.5, 4.0)));
    }
    for _ in 0..64 {
        let direction = [seeds.between(-0.5, 0.5), -1.0, seeds.between(-0.5, 0.5)];
        let outer = seeds.between(0.1, 1.2);
        let light = Light::spot(place(seeds), direction, seeds.color())
            .set_intensity(seeds.between(0.2, 1.0))
            .set_cone_angles(outer * seeds.next(), outer)
            .expect("cones");
        light.set_range(seeds.between(1.0, 5.0)).expect("range");
        lights.push(light);
    }
    for _ in 0..16 {
        lights.push(Light::point(place(seeds), seeds.color()).set_intensity(0.01));
    }
    for _ in 0..8 {
        let direction = [seeds.between(-1.0, 1.0), -1.0, seeds.between(-1.0, 1.0)];
        lights.push(Light::directional(direction, seeds.color()).set_intensity(0.02));
    }
    lights
}

#[test]
fn six_hundred_lights_match_the_reference() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(1));
        let camera = camera();
        let build = || lit_pass("600 lights", Some(&camera), &lights);
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        let uniform = draw(build, Mode::Uniform(1021), 96).await;
        assert_varied(&reference.image, "600 lights");
        assert_eq!(grids(&clustered.context), 1, "the clustered pass bins");
        assert_eq!(grids(&reference.context), 0, "the reference has no grid");
        assert_close(&clustered.image, &reference.image, "clustered");
        assert_close(&uniform.image, &reference.image, "uniform path");
    });
}

/// The floor point `height` above y = 0 at depth `depth` and clip x `ndc`.
fn floor_point(frustum: &Frustum, depth: f32, ndc: f32, height: f32) -> Option<Vec3> {
    // Solve depth_plane · p = depth and (row0 - ndc · row3) · p = 0 for x and z.
    let plane = frustum.depth_plane;
    let side = frustum.view_proj.row(0) - frustum.view_proj.row(3) * ndc;
    let rest = |row: glam::Vec4, value: f32| value - row.y * height - row.w;
    let (a, b, e) = (plane.x, plane.z, rest(plane, depth));
    let (c, d, f) = (side.x, side.z, rest(side, 0.0));
    let determinant = a * d - b * c;
    if determinant.abs() < 1.0e-6 {
        return None;
    }
    let point = Vec3::new(
        (e * d - b * f) / determinant,
        height,
        (a * f - e * c) / determinant,
    );
    (point.x.abs() < 6.0 && point.z.abs() < 6.0).then_some(point)
}

#[test]
fn lights_on_cluster_boundaries_match_the_reference() {
    pollster::block_on(async {
        let camera = camera();
        let matrix = Mat4::from_cols_array_2d(&camera.view_proj());
        let frustum = Frustum::from_matrix(matrix).expect("frustum");
        // A far light sets the reach, so the grid below is the one the pass builds.
        let far = Vec3::new(0.0, 0.5, -5.0);
        let reach = frustum.reach(&[Sphere {
            center: far,
            radius: 9.0,
        }]);
        let grid = frustum.grid(reach);
        let mut lights = vec![bulb(far.to_array(), [0.2; 3], 0.2, 9.0)];
        let mut seeds = Seeds(3);
        let [width, _, slices] = frustum.size;
        for slice in 0..slices {
            for tile in 0..=width {
                for offset in [-GRID_MARGIN, 0.0, GRID_MARGIN] {
                    let depth = grid.depth_at(slice as f32 + offset);
                    let ndc = (tile as f32 + offset) / width as f32 * 2.0 - 1.0;
                    let Some(center) = floor_point(&frustum, depth, ndc, 0.3) else {
                        continue;
                    };
                    let range = (grid.depth_at(slice as f32 + 1.0) - depth).max(0.35);
                    lights.push(bulb(center.to_array(), seeds.color(), 0.3, range));
                }
            }
        }
        assert!(lights.len() > 100, "only {} boundary lights", lights.len());
        let build = || lit_pass("boundaries", Some(&camera), &lights);
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        assert_varied(&reference.image, "boundaries");
        let sets = clustered.context.light_sets.read();
        let binned = sets
            .values()
            .find_map(|set| set.binned.map(|(grid, _)| grid));
        assert_eq!(
            binned,
            Some(grid),
            "the pass bins in the grid the lights sit on"
        );
        drop(sets);
        assert_close(&clustered.image, &reference.image, "boundaries");
    });
}

#[test]
fn lights_behind_and_around_the_eye_match_the_reference() {
    pollster::block_on(async {
        let camera = camera();
        let eye = Vec3::new(0.0, 4.0, 9.0);
        let forward = (Vec3::new(0.0, 1.0, -2.0) - eye).normalize();
        let mut lights = Vec::new();
        let colors = [[1.0, 0.3, 0.3], [0.3, 1.0, 0.3], [0.3, 0.3, 1.0]];
        for (depth, color) in [-2.0, 0.0, 0.05].into_iter().zip(colors) {
            let position = eye + forward * depth + Vec3::new(depth, -1.0, 0.0);
            lights.push(bulb(position.to_array(), color, 4.0, 9.0));
        }
        // The camera sits inside this one.
        lights.push(bulb(eye.to_array(), [0.5, 0.5, 0.2], 3.0, 12.0));
        // Small lights keep a grid, which the large ones above would skip.
        let mut seeds = Seeds(31);
        for _ in 0..64 {
            let position = [seeds.between(-6.0, 6.0), 0.3, seeds.between(-6.0, 6.0)];
            lights.push(bulb(position, seeds.color(), 0.2, 1.0));
        }
        let build = || lit_pass("around the eye", Some(&camera), &lights);
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        assert_varied(&reference.image, "around the eye");
        let promoted = clustered
            .context
            .light_sets
            .read()
            .values()
            .find_map(|set| set.binned.map(|(_, promoted)| promoted));
        assert!(
            promoted.is_some_and(|promoted| promoted > 0),
            "a light around the eye covers every tile: {promoted:?}"
        );
        assert_close(&clustered.image, &reference.image, "around the eye");
    });
}

#[test]
fn orthographic_and_reverse_cameras_match_the_reference() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(5));
        let ortho = Camera::orthographic(-7.0, 7.0, -7.0, 7.0, -5.0, 20.0)
            .expect("camera")
            .look_at([0.0, 4.0, 9.0], [0.0, 1.0, -2.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let build = || lit_pass("orthographic", Some(&ortho), &lights);
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        assert_varied(&reference.image, "orthographic");
        assert_eq!(grids(&clustered.context), 1);
        assert_close(&clustered.image, &reference.image, "orthographic");

        // Reverse-Z on a single plane, which needs no depth order.
        let reverse = Camera::perspective(1.0, 1.0, 50.0, 0.1)
            .expect("camera")
            .look_at([0.0, 3.0, 4.0], [0.0, 2.0, -6.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let build = || {
            let [_, wall] = floor_and_wall(&rough());
            let single = Pass::new("reverse-Z");
            single.add(&wall).expect("wall");
            single.add(&reverse).expect("camera");
            for light in &lights {
                single.add(light).expect("light");
            }
            single
        };
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        assert_varied(&reference.image, "reverse-Z");
        assert_eq!(grids(&clustered.context), 1);
        assert_close(&clustered.image, &reference.image, "reverse-Z");
    });
}

#[test]
fn passes_without_a_camera_bin_in_the_shader_matrix() {
    pollster::block_on(async {
        // The identity camera of a fresh Material sees x and y from -1 to 1
        // and depth from 0 to 1.
        let mut seeds = Seeds(9);
        let lights: Vec<Light> = (0..200)
            .map(|_| {
                let position = [
                    seeds.between(-1.0, 1.0),
                    seeds.between(-1.0, 1.0),
                    seeds.between(0.2, 0.45),
                ];
                let color = seeds.color();
                bulb(position, color, 0.01, seeds.between(0.1, 0.5))
            })
            .collect();
        let build = || {
            let material = rough().double_sided(true);
            let square = quad(
                [
                    [-1.0, -1.0, 0.5],
                    [1.0, -1.0, 0.5],
                    [1.0, 1.0, 0.5],
                    [-1.0, 1.0, 0.5],
                ],
                [0.0, 0.0, -1.0],
            );
            let pass = Pass::new("identity");
            pass.add(&Model::new(square, material)).expect("model");
            for light in &lights {
                pass.add(light).expect("light");
            }
            pass
        };
        let reference = draw(build, Mode::Reference, 64).await;
        let clustered = draw(build, Mode::Clustered, 64).await;
        assert_varied(&reference.image, "identity");
        assert_eq!(grids(&clustered.context), 1);
        assert_close(&clustered.image, &reference.image, "identity");

        // A matrix set by hand, with no Camera on the pass.
        let lights = seeded_lights(&mut Seeds(13));
        let build = || {
            let hand = rough();
            hand.shader()
                .set("camera.view_proj", camera().view_proj())
                .expect("view_proj");
            hand.shader()
                .set("camera.position", [0.0, 4.0, 9.0])
                .expect("position");
            let pass = Pass::new("hand-set");
            for model in floor_and_wall(&hand) {
                pass.add(&model).expect("model");
            }
            for light in &lights {
                pass.add(light).expect("light");
            }
            pass
        };
        let reference = draw(build, Mode::Reference, 96).await;
        let clustered = draw(build, Mode::Clustered, 96).await;
        assert_varied(&reference.image, "hand-set");
        assert_eq!(grids(&clustered.context), 1);
        assert_close(&clustered.image, &reference.image, "hand-set");
    });
}

/// Draws two passes that share a Material under different cameras, the first
/// into an offscreen target, and returns both images with the context.
async fn draw_shared(lights: &[Light], mode: Mode) -> ((Vec<u8>, Vec<u8>), Arc<RenderContext>) {
    let (renderer, target, context) = open(mode, 96).await;
    let material = rough();
    let side = Camera::perspective(1.2, 1.0, 0.5, 40.0)
        .expect("camera")
        .look_at([7.0, 3.0, 2.0], [-2.0, 1.0, -3.0], [0.0, 1.0, 0.0])
        .expect("camera");
    let front = camera();
    let first = Pass::new("first camera");
    let second = Pass::new("second camera");
    for (pass, camera) in [(&first, &side), (&second, &front)] {
        for model in floor_and_wall(&material) {
            pass.add(&model).expect("model");
        }
        pass.add(camera).expect("camera");
    }
    for light in lights {
        first.add(light).expect("light");
    }
    let offscreen = renderer
        .create_texture_target([96, 96])
        .await
        .expect("offscreen");
    first.set_target(&offscreen).expect("target");
    let passes = [&first, &second];
    renderer
        .render(&passes.as_slice(), &target)
        .expect("render");
    let images = (
        offscreen.get_image().await.expect("first"),
        target.get_image().await.expect("second"),
    );
    (images, context)
}

#[test]
fn a_material_shared_by_two_cameras_matches_the_reference() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(17));
        let (reference, _) = draw_shared(&lights, Mode::Reference).await;
        let (clustered, context) = draw_shared(&lights, Mode::Clustered).await;
        assert_eq!(grids(&context), 2, "each pass bins in its own camera");
        assert_varied(&reference.1, "shared material");
        assert_close(&clustered.0, &reference.0, "first camera");
        assert_close(&clustered.1, &reference.1, "second camera");
    });
}

#[test]
fn sixteen_thousand_lights_grow_every_buffer() {
    pollster::block_on(async {
        let mut seeds = Seeds(19);
        let lights: Vec<Light> = (0..16384)
            .map(|_| {
                let position = [
                    seeds.between(-6.0, 6.0),
                    seeds.between(0.1, 2.0),
                    seeds.between(-6.0, 6.0),
                ];
                let color = seeds.color();
                bulb(position, color, 0.02, seeds.between(0.2, 1.0))
            })
            .collect();
        let camera = camera();
        let build = || lit_pass("16384 lights", Some(&camera), &lights);
        let reference = draw(build, Mode::Reference, 128).await;
        let clustered = draw(build, Mode::Clustered, 128).await;
        assert_varied(&reference.image, "16384 lights");
        let sets = clustered.context.light_sets.read();
        let set = sets.values().next().expect("a light set");
        assert!(set.capacity >= 16384, "lights hold {}", set.capacity);
        // One light lists at most half the clusters, or it is promoted.
        let one = grow(u64::from(2 * GRID_CELLS + GRID_CELLS / 2), MIN_WORDS) * 4;
        let grid = set.light_grid.as_ref().map_or(0, |grid| grid.size);
        assert!(
            grid > one,
            "the light grid holds {grid} bytes, one light takes {one}"
        );
        let bins = set.bins.as_ref().map_or(0, |bins| bins.size);
        assert!(
            bins > BINS_HEADER + BIN_SIZE * MIN_LIGHTS,
            "the bins hold {bins} bytes"
        );
        drop(sets);
        assert_close(&clustered.image, &reference.image, "16384 lights");
    });
}

#[test]
fn a_small_storage_limit_promotes_lights() {
    pollster::block_on(async {
        let limit = 256 * 1024;
        let mut seeds = Seeds(23);
        let mut lights: Vec<Light> = (0..2000)
            .map(|_| {
                let position = [
                    seeds.between(-6.0, 6.0),
                    seeds.between(0.2, 3.0),
                    seeds.between(-6.0, 6.0),
                ];
                let color = seeds.color();
                bulb(position, color, 0.004, seeds.between(3.0, 6.0))
            })
            .collect();
        let camera = camera();
        let build = |lights: &[Light]| lit_pass("promoted", Some(&camera), lights);
        let clustered = draw(|| build(&lights), Mode::Limit(limit), 96).await;
        let reference = draw(|| build(&lights), Mode::LimitReference(limit), 96).await;
        let promoted = clustered
            .context
            .light_sets
            .read()
            .values()
            .filter_map(|set| set.binned.map(|(_, promoted)| promoted))
            .max()
            .expect("a grid");
        assert!(promoted > 0, "the limit should promote lights");
        assert_varied(&reference.image, "promoted");
        assert_close(&clustered.image, &reference.image, "promoted");

        // One light past the cap the limit sets fails the render, before any binning.
        let cap = LightPath::Clustered { limit }.cap();
        while lights.len() <= cap as usize {
            lights.push(Light::point([0.0, 1.0, 0.0], [0.0; 3]));
        }
        let (renderer, target, context) = open(Mode::Limit(limit), 96).await;
        match renderer.render(&build(&lights), &target) {
            Err(RendererError::PassError(PassError::LightCapReached { cap: found })) => {
                assert_eq!(found, cap);
            }
            other => panic!("expected LightCapReached, got {other:?}"),
        }
        assert_eq!(dispatches(&context), 0);
    });
}

#[test]
fn idle_light_sets_go_once_no_lit_pass_renders() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(41));
        let camera = camera();
        let (renderer, target, context) = open(Mode::Clustered, 16).await;
        renderer
            .render(&lit_pass("lit", Some(&camera), &lights), &target)
            .expect("render");
        assert_eq!(context.light_sets.read().len(), 1);
        // Unlit renders age the set, and it goes after IDLE_RENDERS of them.
        let unlit = Pass::from_shader("unlit", &Shader::default());
        for _ in 0..=IDLE_RENDERS {
            renderer.render(&unlit, &target).expect("render");
        }
        assert!(context.light_sets.read().is_empty(), "the idle set stays");
    });
}

#[test]
fn still_renders_dispatch_once() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(29));
        let camera = camera();
        let pass = lit_pass("still", Some(&camera), &lights);
        let (renderer, target, context) = open(Mode::Clustered, 32).await;
        for _ in 0..3 {
            renderer.render(&pass, &target).expect("render");
        }
        assert_eq!(dispatches(&context), 1, "three still renders");
        let before = uploads(&context);
        lights[0].set_color([0.1, 0.2, 0.3]);
        renderer.render(&pass, &target).expect("render");
        assert_eq!(
            dispatches(&context),
            1,
            "a color change only rewrites bytes"
        );
        assert_eq!(
            uploads(&context) - before,
            u64::from(LIGHT_SIZE),
            "a color change uploads one light"
        );
        lights[0].set_position([1.0, 1.0, 1.0]).expect("position");
        renderer.render(&pass, &target).expect("render");
        assert_eq!(dispatches(&context), 2, "a moved light rebins");
        camera
            .look_at([1.0, 4.0, 9.0], [0.0, 1.0, -2.0], [0.0, 1.0, 0.0])
            .expect("camera");
        renderer.render(&pass, &target).expect("render");
        assert_eq!(dispatches(&context), 3, "a moved camera rebins");
        camera.set_aspect(1.5).expect("aspect");
        renderer.render(&pass, &target).expect("render");
        assert_eq!(dispatches(&context), 4, "a new aspect rebins");

        // A pass listed twice in one render bins once.
        let twice = lit_pass("twice", Some(&camera), &lights);
        let (renderer, target, context) = open(Mode::Clustered, 32).await;
        let listed = [&twice, &twice];
        renderer
            .render(&listed.as_slice(), &target)
            .expect("render");
        assert_eq!(dispatches(&context), 1, "a pass listed twice");
        assert_eq!(prepares(&context), 1, "a pass listed twice prepares once");

        // Lights without a range never bin.
        let suns: Vec<Light> = (0..4)
            .map(|index| Light::directional([index as f32, -1.0, 0.0], [0.3; 3]))
            .collect();
        let sunny = lit_pass("directional only", Some(&camera), &suns);
        let (renderer, target, context) = open(Mode::Clustered, 32).await;
        renderer.render(&sunny, &target).expect("render");
        renderer.render(&sunny, &target).expect("render");
        assert_eq!(dispatches(&context), 0, "directional lights only");
    });
}

/// A floor with a red ambient and a wall with the default one, lit by `lights`.
fn two_ambients(lights: &[Light], camera: Option<&Camera>) -> Pass {
    let red = rough();
    red.shader()
        .set("lights.ambient", [0.5, 0.0, 0.0])
        .expect("ambient");
    let [floor, _] = floor_and_wall(&red);
    let [_, wall] = floor_and_wall(&rough());
    let pass = Pass::new("two ambients");
    pass.add(&floor).expect("floor");
    pass.add(&wall).expect("wall");
    if let Some(camera) = camera {
        pass.add(camera).expect("camera");
    }
    for light in lights {
        pass.add(light).expect("light");
    }
    pass
}

#[test]
fn a_reused_light_set_packs_its_new_shader() {
    pollster::block_on(async {
        // A pass that gains a Camera keys its sets again with no light change,
        // and each shader may take the idle set the other one left.
        let lights = seeded_lights(&mut Seeds(37));
        let camera = camera();
        let clean = draw(|| two_ambients(&lights, Some(&camera)), Mode::Clustered, 64).await;
        assert_varied(&clean.image, "two ambients");
        // HashMap order picks which idle set each shader takes, so try a few times.
        for attempt in 0..8 {
            let (renderer, target, _) = open(Mode::Clustered, 64).await;
            let pass = two_ambients(&lights, None);
            renderer
                .render(&pass, &target)
                .expect("render without a camera");
            pass.add(&camera).expect("camera");
            renderer
                .render(&pass, &target)
                .expect("render with the camera");
            let image = target.get_image().await.expect("read back");
            assert_close(&image, &clean.image, &format!("attempt {attempt}"));
        }
    });
}

/// A pass whose pipeline fails to build: its fragment output doesn't fit the target.
fn broken_pass() -> Pass {
    let shader = Shader::new(
        "@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<u32> { return vec4<u32>(1u); }",
    )
    .expect("shader");
    Pass::from_shader("broken", &shader)
}

#[test]
fn a_failed_frame_bins_again_on_the_next() {
    pollster::block_on(async {
        let lights = seeded_lights(&mut Seeds(1));
        let camera = camera();
        let build = || lit_pass("600 lights", Some(&camera), &lights);
        let clean = draw(build, Mode::Clustered, 96).await;
        assert_varied(&clean.image, "clean");

        // The first binning sits in a frame that a later pass fails.
        let (renderer, target, _) = open(Mode::Clustered, 96).await;
        let lit = build();
        let broken = broken_pass();
        let listed = [&lit, &broken];
        assert!(renderer.render(&listed.as_slice(), &target).is_err());
        renderer.render(&lit, &target).expect("render");
        let after = target.get_image().await.expect("read back");
        assert_close(&after, &clean.image, "after a failed first binning");

        // A rebin for a moved camera sits in a failed frame.
        let moved = Camera::perspective(1.0, 1.0, 0.1, 100.0)
            .expect("camera")
            .look_at([3.0, 5.0, 7.0], [-1.0, 0.5, -3.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let moved_clean = draw(
            || lit_pass("moved", Some(&moved), &lights),
            Mode::Clustered,
            96,
        )
        .await;
        let (renderer, target, _) = open(Mode::Clustered, 96).await;
        let moving = self::camera();
        let lit = lit_pass("moving", Some(&moving), &lights);
        renderer.render(&lit, &target).expect("render");
        moving
            .look_at([3.0, 5.0, 7.0], [-1.0, 0.5, -3.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let listed = [&lit, &broken];
        assert!(renderer.render(&listed.as_slice(), &target).is_err());
        renderer.render(&lit, &target).expect("render");
        let after = target.get_image().await.expect("read back");
        assert_close(&after, &moved_clean.image, "after a failed rebin");
    });
}

#[test]
fn the_uniform_path_caps_lights_at_its_block() {
    pollster::block_on(async {
        // Lights added before the context exists fail at render.
        let suns: Vec<Light> = (0..5)
            .map(|_| Light::directional([0.0, -1.0, 0.0], [0.2; 3]))
            .collect();
        let early = lit_pass("before the context", Some(&camera()), &suns);
        let (renderer, target, _) = open(Mode::Uniform(4), 32).await;
        match renderer.render(&early, &target) {
            Err(RendererError::PassError(PassError::LightCapReached { cap: 4 })) => {}
            other => panic!("expected LightCapReached at 4, got {other:?}"),
        }

        // With the context open, the fifth add fails and the four render.
        let pass = lit_pass("at the cap", Some(&camera()), &[]);
        for index in 0..4 {
            let light = bulb([index as f32 * 2.0 - 3.0, 1.0, 0.0], [0.5; 3], 3.0, 6.0);
            pass.add(&light).expect("under the cap");
        }
        let fifth = pass.add(&Light::directional([0.0, -1.0, 0.0], [1.0; 3]));
        assert!(
            matches!(fifth, Err(PassError::LightCapReached { cap: 4 })),
            "{fifth:?}"
        );
        renderer.render(&pass, &target).expect("render at the cap");
        assert_varied(&target.get_image().await.expect("read back"), "at the cap");

        // A Model whose shader would take more than four lights fails to join,
        // and changes nothing.
        let material = rough();
        let held = Pass::new("holds two");
        for model in floor_and_wall(&material) {
            held.add(&model).expect("model");
        }
        for index in 0..2 {
            held.add(&Light::point([index as f32, 1.0, 0.0], [1.0; 3]))
                .expect("light");
        }
        let three = Pass::new("holds three");
        for index in 0..3 {
            three
                .add(&Light::point([index as f32, 2.0, 0.0], [1.0; 3]))
                .expect("light");
        }
        let [joining, _] = floor_and_wall(&material);
        let joined = three.add(&joining);
        assert!(
            matches!(joined, Err(PassError::LightCapReached { cap: 4 })),
            "{joined:?}"
        );
        assert!(three.object.shaders.read().is_empty());
        assert_eq!(material.shader().object.gpu_lights().len(), 2);

        // The older `LightArray` shape caps at its own length.
        let older = Material::custom(fullscreen(
            "struct LightArray { count: u32, ambient: vec3<f32>, lights: array<Light, 2> }
@group(0) @binding(1) var<uniform> lights: LightArray;",
            "vec4<f32>(lights.lights[0].color, 1.0)",
        ));
        let small = Pass::from_shader("two lights", older.shader());
        for index in 0..2 {
            small
                .add(&Light::point([index as f32, 0.0, 0.0], [1.0; 3]))
                .expect("under the array length");
        }
        let third = small.add(&Light::point([2.0, 0.0, 0.0], [1.0; 3]));
        assert!(
            matches!(third, Err(PassError::LightCapReached { cap: 2 })),
            "{third:?}"
        );
    });
}

const LIGHT_TYPE: &str = "struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32, position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }";

/// A fullscreen shader that declares `declarations` and writes `color`.
fn fullscreen(declarations: &str, color: &str) -> Shader {
    Shader::new(
        format!(
            "{LIGHT_TYPE}
{declarations}
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {{
  let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(p[i], 0.0, 1.0);
}}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return {color}; }}"
        )
        .as_str(),
    )
    .expect("shader")
}

#[test]
fn custom_shaders_read_both_root_shapes() {
    pollster::block_on(async {
        let declarations = [
            // The older uniform `LightArray`.
            "struct LightArray { count: u32, ambient: vec3<f32>, lights: array<Light, 7> }
@group(0) @binding(1) var<uniform> lights: LightArray;",
            // The storage shape, read without a grid.
            "struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
@group(0) @binding(1) var<storage, read> lights: Lights;",
        ];
        let lights = [
            Light::point([0.0, 0.0, 0.0], [0.2, 0.4, 0.0]),
            Light::point([1.0, 0.0, 0.0], [0.0, 0.0, 0.6]),
        ];
        for declaration in declarations {
            for mode in [Mode::Clustered, Mode::Uniform(8)] {
                let build = || {
                    let shader = fullscreen(
                        declaration,
                        "vec4<f32>(lights.lights[0].color + lights.lights[1].color + lights.ambient, f32(lights.count) * 0.2)",
                    );
                    shader
                        .set("lights.ambient", [0.0, 0.0, 0.2])
                        .expect("ambient");
                    let pass = Pass::from_shader("roots", &shader);
                    for light in &lights {
                        pass.add(light).expect("light");
                    }
                    pass
                };
                let drawn = draw(build, mode, 4).await;
                assert_eq!(
                    drawn.image[..4],
                    [51, 102, 204, 102],
                    "two lights and the ambient, {mode:?}"
                );
            }
        }
    });
}

#[test]
fn the_older_light_array_reads_lights_in_attach_order() {
    pollster::block_on(async {
        // A bounded light added first stays at index 0, ahead of a light without a range.
        let lamp = bulb([0.0, 0.0, 0.0], [0.4, 0.0, 0.0], 1.0, 5.0);
        let sun = Light::directional([0.0, -1.0, 0.0], [0.0, 0.0, 0.8]);
        for mode in [Mode::Clustered, Mode::Uniform(8)] {
            let build = || {
                let shader = fullscreen(
                    "struct LightArray { count: u32, ambient: vec3<f32>, lights: array<Light, 4> }
@group(0) @binding(1) var<uniform> lights: LightArray;",
                    "vec4<f32>(lights.lights[0].color, 1.0)",
                );
                let pass = Pass::from_shader("attach order", &shader);
                pass.add(&lamp).expect("lamp");
                pass.add(&sun).expect("sun");
                pass
            };
            let drawn = draw(build, mode, 4).await;
            assert_eq!(
                drawn.image[..4],
                [102, 0, 0, 255],
                "the lamp first, {mode:?}"
            );
        }
    });
}

const STORAGE_ROOT: &str = "struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>, slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
struct Lights { count: u32, global_count: u32, promoted_count: u32, ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
@group(0) @binding(1) var<storage, read> lights: Lights;";

#[test]
fn a_set_without_light_grid_never_bins() {
    pollster::block_on(async {
        // A shader that loops over every light reads no grid, so its set skips the binning.
        let camera = camera();
        let lamps: Vec<Light> = (0..32)
            .map(|index| bulb([index as f32 * 0.3 - 4.8, 0.5, 0.0], [0.01; 3], 1.0, 2.0))
            .collect();
        let shader = fullscreen(
            STORAGE_ROOT,
            "vec4<f32>(0.0, 0.0, 0.0, f32(lights.count) / 255.0)",
        );
        let pass = Pass::from_shader("no grid", &shader);
        pass.add(&camera).expect("camera");
        for lamp in &lamps {
            pass.add(lamp).expect("light");
        }
        let (renderer, target, context) = open(Mode::Clustered, 4).await;
        renderer.render(&pass, &target).expect("render");
        camera
            .look_at([2.0, 4.0, 9.0], [0.0, 1.0, -2.0], [0.0, 1.0, 0.0])
            .expect("camera");
        renderer.render(&pass, &target).expect("render");
        let image = target.get_image().await.expect("read back");
        assert_eq!(image[3], 32, "the shader reads every light");
        assert_eq!(dispatches(&context), 0, "nothing reads a grid");
        let sets = context.light_sets.read();
        let set = sets.values().next().expect("a light set");
        assert!(set.light_grid.is_none() && set.bins.is_none());
    });
}

#[test]
fn a_shader_added_later_takes_the_pass_lights() {
    pollster::block_on(async {
        let lamp = Light::point([0.0, 0.0, 0.0], [0.2, 0.4, 0.0]);
        let build = || {
            let pass = Pass::new("lights first");
            pass.add(&lamp).expect("light");
            let shader = fullscreen(
                STORAGE_ROOT,
                "vec4<f32>(lights.lights[0].color, f32(lights.count) * 0.2)",
            );
            pass.add_shader(&shader)
                .expect("the shader takes the light");
            pass
        };
        let drawn = draw(build, Mode::Clustered, 4).await;
        assert_eq!(drawn.image[..4], [51, 102, 0, 51], "the pass's light");

        // A shader that can't take the pass's lights fails to join, and nothing changes.
        let pass = Pass::new("three lights");
        for index in 0..3 {
            pass.add(&Light::point([index as f32, 0.0, 0.0], [1.0; 3]))
                .expect("light");
        }
        let small = fullscreen(
            "struct LightArray { count: u32, ambient: vec3<f32>, lights: array<Light, 2> }
@group(0) @binding(1) var<uniform> lights: LightArray;",
            "vec4<f32>(lights.lights[0].color, 1.0)",
        );
        let joined = pass.add_shader(&small);
        assert!(
            matches!(joined, Err(PassError::LightCapReached { cap: 2 })),
            "{joined:?}"
        );
        assert!(pass.object.shaders.read().is_empty());
        assert!(small.object.gpu_lights().is_empty());
    });
}

#[test]
fn a_light_lights_after_its_handle_drops() {
    pollster::block_on(async {
        let center = |image: &[u8]| image[(16 * 32 + 16) * 4];
        let view = || {
            Camera::perspective(1.0, 1.0, 0.1, 100.0)
                .expect("camera")
                .look_at([0.0, 3.0, 2.0], [0.0, 3.0, -6.0], [0.0, 1.0, 0.0])
                .expect("camera")
        };
        let lamp = || bulb([0.0, 3.0, -4.0], [1.0; 3], 8.0, 6.0);
        let render_scene = |keep: bool| async move {
            let scene = crate::Scene::new();
            let [_, wall] = floor_and_wall(&rough());
            scene.add(&wall).expect("wall");
            scene.add(&view()).expect("camera");
            let light = lamp();
            scene.add(&light).expect("light");
            let handle = keep.then_some(light);
            let (renderer, target, _) = open(Mode::Clustered, 32).await;
            renderer.render(&scene, &target).expect("render");
            let image = target.get_image().await.expect("read back");
            drop(handle);
            image
        };
        let held = center(&render_scene(true).await);
        let dropped = center(&render_scene(false).await);
        assert!(held > 150, "the light should light the wall, got {held}");
        assert_eq!(
            dropped, held,
            "the scene keeps the light after its handle drops"
        );

        // A Model that joins after the handle dropped takes the pass's light.
        let late = || {
            let pass = Pass::new("late model");
            pass.add(&lamp()).expect("light");
            let [_, wall] = floor_and_wall(&rough());
            pass.add(&wall).expect("wall");
            pass.add(&view()).expect("camera");
            pass
        };
        let drawn = draw(late, Mode::Clustered, 32).await;
        assert_eq!(center(&drawn.image), held, "the pass keeps the light");
    });
}

/// The value of the WGSL constant `name` in `module`.
fn constant(module: &naga::Module, name: &str) -> naga::Literal {
    let (_, found) = module
        .constants
        .iter()
        .find(|(_, constant)| constant.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no constant {name}"));
    match module.global_expressions[found.init] {
        naga::Expression::Literal(literal) => literal,
        ref other => panic!("{name} is {other:?}"),
    }
}

/// The offset of member `member` in struct `name`, and the struct's span.
fn layout(module: &naga::Module, name: &str, member: &str) -> (u32, u32) {
    for (_, ty) in module.types.iter() {
        if ty.name.as_deref() != Some(name) {
            continue;
        }
        if let naga::TypeInner::Struct { members, span } = &ty.inner {
            let offset = members
                .iter()
                .find(|found| found.name.as_deref() == Some(member))
                .map(|found| found.offset)
                .unwrap_or_else(|| panic!("{name} has no {member}"));
            return (offset, *span);
        }
    }
    panic!("no struct {name}");
}

#[test]
fn constants_match() {
    let pbr = Material::pbr();
    let shading = &pbr.shader().object.module;
    let binning = binning_module().expect("binning module");
    for module in [shading, &binning] {
        assert_eq!(
            constant(module, "GRID_LONG"),
            naga::Literal::U32(grid::GRID_LONG)
        );
        assert_eq!(
            constant(module, "GRID_SHORT"),
            naga::Literal::U32(grid::GRID_SHORT)
        );
        assert_eq!(
            constant(module, "GRID_SLICES"),
            naga::Literal::U32(grid::GRID_SLICES)
        );
        assert_eq!(
            constant(module, "GRID_MARGIN"),
            naga::Literal::F32(GRID_MARGIN)
        );
        assert_eq!(layout(module, "Light", "color"), (48, LIGHT_SIZE));
        assert_eq!(layout(module, "LightGrid", "reach"), (104, 112));
    }
    assert_eq!(layout(shading, "Lights", "grid").0, 32);
    assert_eq!(layout(shading, "Lights", "lights").0, LIGHTS_HEADER);
    assert_eq!(u64::from(layout(&binning, "Bins", "bins").0), BINS_HEADER);
    assert_eq!(u64::from(layout(&binning, "Bin", "cells").1), BIN_SIZE);
    assert_eq!(GRID_CELLS % WORKGROUP, 0);
}
