use instant::Instant;

struct Cube {
    node: plrender::NodeId,
    level: u8,
}

const SCALE_ROOT: f32 = 2.0;
const SCALE_LEVEL: f32 = 0.4;

struct Level {
    color: plrender::Color,
    speed: f32,
}

fn fill_scene(
    levels: &[Level],
    scene: &mut plrender::Scene,
    prototype: &plrender::Bundle,
) -> Vec<Cube> {
    let root_node = scene.add_node().scale(SCALE_ROOT).build();

    let mut cube = scene.add_entity(prototype);
    let _cube = cube.parent(root_node).component(levels[0].color).build();

    let mut list = vec![Cube {
        node: root_node,
        level: 0,
    }];

    struct Stack {
        parent: plrender::NodeId,
        level: u8,
    }
    let mut stack = vec![Stack {
        parent: root_node,
        level: 1,
    }];

    let children = [
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
        for &child in children.iter() {
            let node = scene
                .add_node()
                .position(mint::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0 + SCALE_LEVEL,
                })
                .scale(SCALE_LEVEL)
                .parent(next.parent)
                .build();

            // OPEN QUESTION:
            // Why does it use POST rotate here
            // and pre_rotate in the other transform?
            scene[node].post_rotate(child, 90.0);

            scene
                .add_entity(prototype)
                .parent(node)
                .component(level.color)
                .build();

            list.push(Cube {
                node,
                level: next.level,
            });

            stack.push(Stack {
                parent: node,
                level: next.level + 1,
            });
        }
    }

    list
}

const LEVELS: &[Level] = &[
    Level {
        color: plrender::Color(0xFFFFFF80),
        speed: 20.0,
    },
    Level {
        color: plrender::Color(0xFF8080FF),
        speed: -30.0,
    },
    Level {
        color: plrender::Color(0xFF80FF80),
        speed: 40.0,
    },
    Level {
        color: plrender::Color(0xFFFF8080),
        speed: -60.0,
    },
    Level {
        color: plrender::Color(0x8880FF55),
        speed: 80.0,
    },
];

fn main() {
    use plrender::{
        geometry::{Geometry, Streams},
        renderer::RenderOptions,
        target::events::Event,
        target::window::Window,
        Renderer,
    };

    let window = Window::default().set_title("Cubes").set_size((800, 600));

    let mut renderer = pollster::block_on(
        Renderer::new(RenderOptions {
            targets: Some(vec![window]),
            ..Default::default()
        })
        .unwrap(),
    );

    let mut scene = plrender::Scene::new();

    let camera = plrender::Camera {
        projection: plrender::Projection::Perspective { fov_y: 45.0 },
        depth: 1.0..10.0,
        node: scene
            .add_node()
            .position([1.8f32, -8.0, 3.0].into())
            .look_at([0f32; 3].into(), [0f32, 0.0, 1.0].into())
            .build(),
        background: plrender::Color(0xFF203040),
    };

    let prototype = Geometry::cuboid(
        Streams::empty(),
        mint::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    )
    .bake(&mut renderer);

    let cubes = fill_scene(&LEVELS[..], &mut scene, &prototype);
    println!("Initialized {} cubes", cubes.len());

    let mut pass = plrender::renderer::renderpass::Solid::new(
        &plrender::renderer::renderpass::SolidConfig {
            cull_back_faces: true,
        },
        &renderer,
    );

    let mut moment = Instant::now();

    window.run(move |event| match event {
        Event::Resize { width, height } => {
            renderer.resize(width, height);
        }
        Event::Draw => {
            let delta = moment.elapsed().as_secs_f32();
            moment = Instant::now();
            for cube in cubes.iter() {
                let level = &LEVELS[cube.level as usize];
                // OPEN QUESTION:
                // Why does it use PRE rotate here
                // and post_rotate in the first transform?
                scene[cube.node].pre_rotate(
                    mint::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    delta * level.speed,
                );
            }

            renderer.render(&mut pass, &scene, &camera);
        }
        _ => {}
    })
}
