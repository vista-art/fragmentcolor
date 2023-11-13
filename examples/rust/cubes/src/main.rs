use instant::Instant;
use plrender::Color;

const ROOT_SCALE: f32 = 2.0;
const SCALE_LEVEL: f32 = 0.4;

struct Cube {
    node_id: plrender::NodeId,
    level: u8,
}

struct Level {
    color: Color,
    speed: f32,
}

const LEVELS: &[Level] = &[
    Level {
        color: Color(0xFFFF80FF),
        speed: 20.0,
    },
    Level {
        color: Color(0x8080FFFF),
        speed: -30.0,
    },
    Level {
        color: Color(0x80FF80FF),
        speed: 40.0,
    },
    Level {
        color: Color(0xFF8080FF),
        speed: -60.0,
    },
    Level {
        color: Color(0x80FF5588),
        speed: 80.0,
    },
];

fn fill_scene(
    levels: &[Level],
    scene: &mut plrender::Scene,
    mesh: plrender::MeshPrototype,
) -> Vec<Cube> {
    let root_scale = mint::Vector3::from([ROOT_SCALE; 3]);
    let mut root = scene.new_empty();
    root.set_scale(root_scale);

    scene.add(&mut root);

    let mut renderable = scene.new_renderable(&mesh);
    let cube = renderable
        .set_parent(root.node.id())
        .add_component(levels[0].color);

    scene.add(cube);

    let mut list = vec![Cube {
        node_id: root.node.id(),
        level: 0,
    }];

    struct Stack {
        parent: plrender::NodeId,
        level: u8,
    }
    let mut stack = vec![Stack {
        parent: root.node.id(),
        level: 1,
    }];

    let children_positions = [
        mint::Vector3::from([0.0, 0.0, 1.0]),
        mint::Vector3::from([1.0, 0.0, 0.0]),
        mint::Vector3::from([-1.0, 0.0, 0.0]),
        mint::Vector3::from([0.0, 1.0, 0.0]),
        mint::Vector3::from([0.0, -1.0, 0.0]),
    ];

    // @TODO After you compile the next version of the core,
    // Swap the pre and post rotates below and see what happens.
    // Also link the StackOverflow answer in the documentation.
    while let Some(next) = stack.pop() {
        let level = match levels.get(next.level as usize) {
            Some(level) => level,
            None => continue,
        };
        for &position in children_positions.iter() {
            let mut child = scene.new_empty();
            child
                .set_position(mint::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0 + SCALE_LEVEL,
                })
                .set_scale(mint::Vector3::from([SCALE_LEVEL; 3]))
                .set_parent(next.parent);

            scene.add(&mut child);

            // Maybe use rotate() if the results differ from the original.
            let mut state = scene.state_mut();
            state[child.node.id()].pre_rotate(position, 90.0);

            let mut renderable = scene.new_renderable(&mesh);

            renderable
                .set_parent(child.node.id())
                .add_component(level.color);

            state.add(&mut renderable);

            list.push(Cube {
                node_id: child.node.id(),
                level: next.level,
            });

            stack.push(Stack {
                parent: child.node.id(),
                level: next.level + 1,
            });
        }
    }

    list
}

fn main() {
    use plrender::{
        app::events::Event,
        app::window::Window,
        geometry::{Geometry, VertexTypes},
        renderer::RenderOptions,
        Renderer,
    };
    let mut window = Window::default();
    window.set_title("Cubes").set_size((800, 600));

    // We can configure the renderer manually if we want
    let mut renderer = pollster::block_on(Renderer::new(RenderOptions {
        targets: Some(vec![&mut window]),
        render_pass: Some("solid"),
        ..Default::default()
    }))
    .unwrap();

    let mut scene = plrender::Scene::new();

    let mut camera_node = scene.new_empty();
    camera_node
        .set_position([1.8f32, -8.0, 3.0].into())
        .look_at([0f32; 3].into(), [0f32, 0.0, 1.0].into());

    scene.add(&mut camera_node);

    // The Geometry Object uses the Renderer to create an
    // internal Mesh inside the renderer, which returns a
    // MeshPrototype containing the MeshId.
    let mesh = Geometry::cuboid(
        VertexTypes::empty(),
        mint::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    )
    .build_mesh(&mut renderer);

    let cubes = fill_scene(&LEVELS[..], &mut scene, mesh);
    println!("Initialized {} cubes", cubes.len());

    let mut moment = Instant::now();

    window.on("draw", move |event| match event {
        Event::Draw => {
            let delta = moment.elapsed().as_secs_f32();
            moment = Instant::now();
            for cube in cubes.iter() {
                let level = &LEVELS[cube.level as usize];
                // NOTE use pre_rotate if the results
                //      differ from the original.
                let mut state = scene.state_mut();
                state[cube.node_id].rotate(
                    mint::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    delta * level.speed,
                );
            }

            let _ = renderer.render(&scene);
        }
        _ => {}
    })
}
