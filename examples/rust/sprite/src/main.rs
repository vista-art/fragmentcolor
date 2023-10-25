use instant::{Duration, Instant};
use plrender::{window::WindowOptions, RenderOptions, RenderPass};

//TODO: a mechanism like this should be a part of the engine
struct Animator {
    map: plrender::loader::SpriteMap,
    cell_counts: mint::Vector2<usize>,
    duration: Duration,
    sprite: plrender::EntityId,
    current: mint::Point2<usize>,
    moment: Instant,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle = 0,
    MoveRight = 9,
    MoveLeft = 8,
    Kick = 4,
    Jump = 10,
    Lie = 12,
}
impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

impl Animator {
    fn update_uv(&mut self, scene: &mut plrender::Scene) {
        let uv_range = self.map.at(self.current);
        scene
            .world
            .get::<&mut plrender::Sprite>(self.sprite)
            .unwrap()
            .uv = Some(uv_range);
    }

    fn switch(&mut self, state: State, scene: &mut plrender::Scene) {
        self.moment = Instant::now();
        self.current.x = 0;
        self.current.y = state as usize;
        self.update_uv(scene);
    }

    fn tick(&mut self, scene: &mut plrender::Scene) {
        if self.moment.elapsed() < self.duration {
            return;
        }

        self.current.x += 1;
        self.moment = Instant::now();
        if self.current.x == self.cell_counts.x {
            self.current.x = 0;
            self.current.y = State::Idle as usize;
            // don't update the scene here, so that
            // input can have a chance to transition
            // to something other than `Idle`.
        } else {
            self.update_uv(scene);
        }
    }
}

fn main() {
    use plrender::window::{Event, Key, Window};

    // This will be created by the user in their envieronment
    // or provided by our library if the "Window" feature is enabled
    //
    // **Example JS:**
    // ```javascript
    //     let canvas = document.getElementById("canvas");
    //     // or
    //     let canvas = plrender::Canvas();
    //     // or
    //     let canvas = plrender::Canvas("#canvas");
    //     let canvas = plrender::Canvas({width: 800, height: 600}});
    // ````
    //
    // **Example Python:**
    // ```python
    //     import pygame
    //     pygame.init()
    //     pygame.display.set_mode((800, 600))
    //     window = pygame.display.get_surface()
    //     # or
    //     window = plrender::Window()
    //     # or
    //     window = plrender::Window(size: (800, 600), title: "My Window")
    // ```
    let window = Window::new(WindowOptions {
        title: Some("Sprite".to_string()),
        size: Some((400, 300)),
        ..Default::default()
    })
    .unwrap();

    let mut renderer = pollster::block_on(plrender::Renderer::new(RenderOptions {
        targets: vec![window],
    }))
    .unwrap();

    let mut scene = plrender::Scene::new();

    // this has to be part of the scene and created
    // by default without user input
    let camera = plrender::Camera {
        projection: plrender::Projection::Orthographic {
            // the sprite configuration is not centered
            center: [0.0, -10.0].into(),
            extent_y: 40.0,
        },
        ..Default::default()
    };

    // this needs to be abstracted away. Ideally, we should have some
    // preset render passes as attributes of the renderer
    let mut pass = plrender::renderpass::Flat::new(&mut renderer);

    let image = renderer
        .load_image(format!(
            "{}/assets/images/pickachu.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

    let sprite = scene.add_sprite(image).build();

    let mut anim = Animator {
        map: plrender::loader::SpriteMap {
            origin: mint::Point2 { x: 0, y: 0 },
            cell_size: mint::Vector2 { x: 96, y: 96 },
        },
        cell_counts: mint::Vector2 { x: 5, y: 13 },
        duration: Duration::from_secs_f64(0.1),
        current: mint::Point2 { x: 0, y: 0 },
        moment: Instant::now(),
        sprite,
    };
    anim.switch(State::Idle, &mut scene);

    window.on_resize(|width, height| {
        // renderer.resize(width, height);
    });

    window.run(move |event| match event {
        Event::Resize { width, height } => {
            // renderer.resize(width, height);
        }
        Event::Keyboard { key, pressed: true } => {
            let new_state = match key {
                Key::Up => Some(State::Jump),
                Key::Down => Some(State::Lie),
                Key::Space => Some(State::Kick),
                Key::Left => Some(State::MoveLeft),
                Key::Right => Some(State::MoveRight),
                _ => None,
            };
            if let Some(state) = new_state {
                if anim.current.y != state as usize || state == State::Kick {
                    anim.switch(state, &mut scene);
                }
            }
        }
        Event::Draw => {
            anim.tick(&mut scene);
            //                                 @TODO
            renderer.render(&mut pass, &scene, &scene.camera());
        }
        _ => {}
    })
}
