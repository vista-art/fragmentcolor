use instant::Instant;
use plrender::{
    app::events::Event,
    app::AppOptions,
    app::{window::Window, WindowOptions},
    components::{self},
    components::{Camera, CameraOptions, Color, Empty, Mesh},
    math::geometry::Primitive,
    math::linear_algebra::{Vec3, ORIGIN, UP_VECTOR},
    renderer::{RenderOptions, RenderTargetDescription},
    scene::SceneObjectEntry,
    IsWindow, NodeId, PLRender, Projection, SceneObject,
};

const ROOT_SCALE: f32 = 2.0;
const SCALE_LEVEL: f32 = 0.4;

#[derive(Clone)]
struct CubePosition {
    object: SceneObject<Empty>,
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
        renderer: RenderOptions {
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
        framerate: Some(1),
        ..Default::default()
    })
    .unwrap();
    let window2 = Window::new(WindowOptions {
        title: "Cubes 2".to_string(),
        size: (400, 300),
        framerate: Some(1),
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
    scene.add_target(
        RenderTargetDescription::from_window(&window)
            .set_camera(&camera)
            .set_clear_color(Color(0xe0a0f0FF)),
    );
    scene.add_target(
        RenderTargetDescription::from_window(&window2)
            .set_camera(&camera)
            .set_clear_color(Color(0x50a0f0FF)),
    );

    // Creates a cube mesh.
    let cube = Primitive::cube(1.0).create_mesh();

    // Creates all the cubes and adds them to the Scene.
    let mut cubes = fill_scene(&LEVELS[..], &mut scene, cube);
    println!("Initialized {} cubes", cubes.len());

    // Let's go!
    let mut moment = Instant::now();
    let state = window.state();
    let mut cubes2 = cubes.clone();
    window.on("draw", move |event| match event {
        Event::Draw => {
            println!();
            println!("======= Redraw called on Window 1!!!!!!! =======");

            let window = state.read().unwrap();
            println!("FPS: {}", 1.0 / moment.elapsed().as_secs_f32());

            let delta = moment.elapsed().as_secs_f32();
            moment = Instant::now();

            for cube in cubes.iter_mut() {
                let level = &LEVELS[cube.level as usize];

                // NOTE use pre_rotate if the results
                //      differ from the original.

                // println!("=> updating node id: {:?}...", cube.node_id);
                // println!(
                //     "=> node in scene: {:?}...",
                //     scene.read_state()[cube.node_id]
                // ); @TODO investigate why NodeId is None
                cube.object.pre_rotate(
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    delta * level.speed,
                );
            }

            println!("Rendering scene...");
            _ = scene.render();

            println!("Redrawing window...");
            window.redraw();

            println!("======= Redraw finished =======");
            println!();
        }
        _ => {}
    });

    let state2 = window2.state();
    window2.on("draw", move |event| match event {
        Event::Draw => {
            println!();
            println!("======= Redraw called on Window 2!!!!!!! =======");

            let window = state2.read().unwrap();
            println!("FPS: {}", 1.0 / moment.elapsed().as_secs_f32());

            let delta = moment.elapsed().as_secs_f32();
            moment = Instant::now();

            for cube in cubes2.iter_mut() {
                let level = &LEVELS[cube.level as usize];

                // NOTE use pre_rotate if the results
                //      differ from the original.

                // println!("=> updating node id: {:?}...", cube.node_id);
                // println!(
                //     "=> node in scene: {:?}...",
                //     scene.read_state()[cube.node_id]
                // ); @TODO investigate why NodeId is None
                cube.object.rotate(
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    delta * level.speed,
                );
            }

            // @TODO currently scene will render to every
            //       window, but I should optionally have
            //       control over which window it renders.
            // println!("Rendering scene...");
            // _ = scene.render();

            println!("Redrawing window...");
            window.redraw();

            println!("======= Redraw finished =======");
            println!();
        }
        _ => {}
    });

    window.on("resize", move |event| match event {
        Event::Resized { width, height } => {
            println!("Resized to {}x{}", width, height);
        }
        _ => {}
    });

    // Runs the application.
    pollster::block_on(PLRender::run());
}

struct Stack {
    parent: NodeId,
    level: u8,
}

fn fill_scene(
    levels: &[Level],
    scene: &mut plrender::Scene,
    mesh: plrender::BuiltMesh,
) -> Vec<CubePosition> {
    let root_scale = Vec3::from([ROOT_SCALE; 3]);
    let mut root = components::Empty::new();
    root.set_scale(root_scale);
    scene.add(&mut root);

    let mut cube = Mesh::new(&mesh);
    cube.add_component(levels[0].color);
    cube.set_scale(root_scale);
    scene.add(&mut cube);

    let mut stack = vec![Stack {
        parent: root.node_id(),
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
                .set_parent_node(next.parent);

            scene.add(&mut child);

            child.pre_rotate(child_position, 90.0);

            let mut child_cube = Mesh::new(&mesh);
            child_cube.set_parent(&child).add_component(level.color);

            scene.add(&mut child_cube);

            stack.push(Stack {
                parent: child.node_id(),
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
