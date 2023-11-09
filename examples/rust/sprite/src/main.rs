use std::sync::{Arc, Mutex};

use instant::{Duration, Instant};
use plrender::{animation::Animator, app::Key, Event, Target};

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

fn main() {
    pollster::block_on(run());
}

async fn run() {
    use plrender::app::window::Window;

    // You can use the App to create multiple Windows...
    //   let app = plrender::App::default();
    //   let window1 = plrender::Window::new(&app, WindowOptions::default());
    //   let window2 = plrender::Window::new(&app, WindowOptions::default());
    //
    // ...or, if you are sure you'll use only one Window, you can create
    // a default singleton Window that contains an App instance inside it:
    //
    // Renderer is now implicitly created by Window::default() or App::add_window()
    // let mut renderer = Renderer::new(RenderOptions {
    //     targets: Some(vec![window]),
    //     ..Default::default()
    // })
    // .await
    // .unwrap();

    let window = Window::default().set_title("Sprite").set_size((800, 600));
    let mut scene = plrender::Scene::new();

    // @TODO scene will automatically create the camera below
    //       the aspect and stride will be inferred from the window resolution
    scene.add_target(Target::Window(window));

    // this has to be part of the scene and created
    // by default without user input.
    let renderer = window.app().unwrap().state().renderer();

    // It's currently like this:
    let image = renderer
        .unwrap()
        .load_image(format!(
            "{}/assets/images/pickachu.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

    let sprite = scene.add_sprite(image).build();

    // @TODO But should be like this:
    // let sprite = plrender::Sprite::new(
    //     &mut app,
    //     format!(
    //         "{}{}",
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/assets/images/pickachu.png"
    //     ),
    // );
    //
    // scene.add(sprite);

    let mut anim = Arc::new(Mutex::new(Animator {
        map: plrender::asset::SpriteMap {
            origin: mint::Point2 { x: 0, y: 0 },
            cell_size: mint::Vector2 { x: 96, y: 96 },
        },
        cell_counts: mint::Vector2 { x: 5, y: 13 },
        duration: Duration::from_secs_f64(0.1),
        current: mint::Point2 { x: 0, y: 0 },
        moment: Instant::now(),
        sprite,
    }));

    anim.try_lock()
        .unwrap()
        .switch::<usize>(State::Idle as usize, &mut scene);

    window.on(
        "keydown",
        Box::new(|event| match event {
            Event::Keyboard { key, pressed } => {
                if pressed {
                    let new_state = match key {
                        Key::Up => Some(State::Jump),
                        Key::Down => Some(State::Lie),
                        Key::Space => Some(State::Kick),
                        Key::Left => Some(State::MoveLeft),
                        Key::Right => Some(State::MoveRight),
                        _ => None,
                    };
                    if let Some(state) = new_state {
                        let anim = anim.try_lock().unwrap();
                        if anim.current.y != state as usize || state == State::Kick {
                            anim.switch::<usize>(state as usize, &mut scene);
                        }
                    }
                };
            }
            _ => {}
        }),
    );

    window.on("draw", |event| {
        let renderer = window.app().unwrap().state().renderer().unwrap();
        let mut anim = anim.try_lock().unwrap();
        anim.tick(&mut scene);
        renderer.render(&scene); //@todo remove camera, pick it with &scene.camera());
    });

    window.run();
}
