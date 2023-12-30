use instant::Instant;
use plrender::{
    app::events::Event,
    app::window::{IsWindow, Window, WindowOptions},
    app::AppOptions,
    components::{self},
    components::{Camera, CameraOptions, Color, Empty, Mesh, Projection},
    math::cg::{Vec3, ORIGIN, UP_VECTOR},
    math::geometry::Primitive,
    renderer::RendererOptions,
    scene::{transform::TransformId, Object},
    PLRender,
};

const ROOT_SCALE: f32 = 2.0;
const SCALE_LEVEL: f32 = 0.4;

#[derive(Clone)]
struct CubePosition {
    object: Object<Empty>,
    level: u8,
}

#[derive(Clone, Copy, Debug)]
struct Level {
    color: Color,
    speed: f32,
}

const LEVELS: &[Level] = &[
    Level {
        color: Color(0xFFFFFF80),
        speed: 20.0,
    },
    Level {
        color: Color(0xFF8080FF),
        speed: -30.0,
    },
    Level {
        color: Color(0xFF80FF80),
        speed: 40.0,
    },
    Level {
        color: Color(0xFFFF8080),
        speed: -60.0,
    },
    Level {
        color: Color(0xFF80FFFF),
        speed: 80.0,
    },
];

fn main() {
    // Configures the Renderer to Solid 3D RenderPass.
    PLRender::config(AppOptions {
        log_level: "info".to_string(),
        renderer: RendererOptions {
            render_pass: "solid".to_string(),
            ..Default::default()
        },
    });

    // Creates a new Scene.
    let mut scene = plrender::Scene::new();

    // Creates two Windows.
    let window = Window::new(WindowOptions {
        title: "Cubes 1".to_string(),
        size: (400, 300),
        framerate: Some(15),
        ..Default::default()
    })
    .unwrap();

    // Creates a new Camera (necessary for 3D scenes).
    let mut camera = Camera::new(CameraOptions {
        projection: Projection::perspective(45.0),
        z_near: 1.0,
        z_far: 10.0,
    });
    camera.set_position(Vec3::from([1.8, -8.0, 3.0]));
    camera.look_at(ORIGIN, UP_VECTOR);

    // Adds the Camera to the Scene.
    scene.add(&mut camera);

    // Attaches this Camera to the Window.
    scene.target_with_camera(&window, &camera);

    // Creates a cube mesh.
    let cube = Primitive::cube(1.0).create_mesh();

    // Creates all the cubes and adds them to the Scene.
    let mut cubes = fill_scene(&LEVELS[..], &mut scene, cube.unwrap());
    println!("Initialized {} cubes", cubes.len());

    // Let's go!
    let mut moment = Instant::now();
    let state = window.state();
    window.on("draw", move |event| match event {
        Event::Draw => {
            let window = state.read().unwrap();
            println!("FPS: {}", 1.0 / moment.elapsed().as_secs_f32());

            let delta = moment.elapsed().as_secs_f32();
            moment = Instant::now();

            for cube in cubes.iter_mut() {
                let level = &LEVELS[cube.level as usize];

                // println!("=> updating transform id: {:?}...", cube.transform_id);
                // println!(
                //     "=> transform in scene: {:?}...",
                //     scene.read_state()[cube.transform_id]
                //
                // ); @TODO investigate why TransformId is None
                cube.object.pre_rotate(
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    delta * level.speed,
                );
            }

            _ = scene.render();

            window.redraw();

            println!("======= Redraw finished =======");
            println!();
        }
        _ => {}
    });

    // Runs the application.
    PLRender::run();
}

struct Stack {
    parent: TransformId,
    level: u8,
}

fn fill_scene(
    levels: &[Level],
    scene: &mut plrender::Scene,
    mesh: plrender::resources::mesh::BuiltMesh,
) -> Vec<CubePosition> {
    let root_scale = Vec3::from([ROOT_SCALE; 3]);
    let mut root = components::Empty::new();
    root.set_scale(root_scale);
    scene.add(&mut root);

    let mut cube = Mesh::new(Some(mesh.clone()));
    cube.add_component(levels[0].color);
    cube.set_scale(root_scale);
    scene.add(&mut cube);

    let mut stack = vec![Stack {
        parent: root.parent(),
        level: 1,
    }];

    let mut list = vec![CubePosition {
        object: root,
        level: 0,
    }];

    let children_positions = [
        Vec3::from([0.0, 0.0, 1.0]),
        Vec3::from([1.0, 0.0, 0.0]),
        Vec3::from([-1.0, 0.0, 0.0]),
        Vec3::from([0.0, 1.0, 0.0]),
        Vec3::from([0.0, -1.0, 0.0]),
    ];

    // @TODO After you compile the next version of the core,
    // Swap the pre and post rotates below and see what happens.
    while let Some(next) = stack.pop() {
        let level = match levels.get(next.level as usize) {
            Some(level) => level,
            None => continue,
        };
        for &child_position in children_positions.iter() {
            let mut child = components::Empty::new();
            child
                .set_position(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0 + SCALE_LEVEL,
                })
                .set_scale(Vec3::from([SCALE_LEVEL; 3]))
                .set_parent_transform(next.parent);

            scene.add(&mut child);

            child.pre_rotate(child_position, 90.0);

            let mut child_cube = Mesh::new(Some(mesh.clone()));
            child_cube.set_parent(&child).add_component(level.color);

            scene.add(&mut child_cube);

            stack.push(Stack {
                parent: child.parent(),
                level: next.level + 1,
            });

            list.push(CubePosition {
                object: child,
                level: next.level,
            });
        }
    }

    list
}
