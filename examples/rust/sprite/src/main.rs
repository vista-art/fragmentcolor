use std::sync::{Arc, Mutex, OnceLock};

use instant::{Duration, Instant};
use plrender::{
    app::{window::Window, Key, PLRender},
    components::animation::Animator,
    scene::Scene,
    Event,
};

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum AnimationState {
    Idle = 0,
    MoveRight = 9,
    MoveLeft = 8,
    Kick = 4,
    Jump = 10,
    Lie = 12,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

static SCENE: OnceLock<Arc<Mutex<Scene>>> = OnceLock::new();

static ANIMATOR: OnceLock<Arc<Mutex<Animator>>> = OnceLock::new();

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let mut window = Window::default()
        .set_title("Sprite")
        .set_size((800, 600))
        .clone();

    // @TODO Scene::new() should register itself in the App
    //       like the Window does.
    let mut scene = SCENE
        .get_or_init(|| Arc::new(Mutex::new(Scene::new())))
        .lock()
        .unwrap();

    // @TODO Renderer has to be accessed internally
    //       by the scene without user input.
    let app = PLRender::app();
    let state = app.state();
    let mut renderer = state.renderer::<Window>();

    // @TODO Resources loading could come from the Sprite itself
    let image = renderer
        .load_image(format!(
            "{}/assets/images/pickachu.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

    let mut sprite = scene.new_sprite(image);
    scene.add(&mut sprite);

    let anim = ANIMATOR.get_or_init(|| {
        Arc::new(Mutex::new(Animator {
            sprite_map: plrender::asset::SpriteMap {
                origin: mint::Point2 { x: 0, y: 0 },
                cell_size: mint::Vector2 { x: 96, y: 96 },
            },
            cell_counts: mint::Vector2 { x: 5, y: 13 },
            duration: Duration::from_secs_f64(0.1),
            current: mint::Point2 { x: 0, y: 0 },
            moment: Instant::now(),
            sprite: plrender::ObjectId::DANGLING,
        }))
    });

    let mut anim = anim.lock().unwrap();
    anim.sprite = sprite.id().unwrap();
    anim.switch::<usize>(AnimationState::Idle as usize, &mut scene);

    window.on("keydown", on_keydown);

    window.on("draw", update);

    window.run().await;
}

fn on_keydown(event: Event) {
    match event {
        Event::Keyboard { key, pressed } => {
            if pressed {
                let new_state = match key {
                    Key::Up => Some(AnimationState::Jump),
                    Key::Down => Some(AnimationState::Lie),
                    Key::Space => Some(AnimationState::Kick),
                    Key::Left => Some(AnimationState::MoveLeft),
                    Key::Right => Some(AnimationState::MoveRight),
                    _ => None,
                };

                if let Some(_state) = new_state {
                    // let mut scene = SCENE.get_mut().unwrap();
                    // let anim = ANIMATOR.get_mut().unwrap();
                    // let mut anim = anim.lock().unwrap();

                    // if anim.current.y != state as usize || state == AnimationState::Kick {
                    //     //anim.switch::<usize>(state as usize, &mut scene);
                    // }
                }
            };
        }
        _ => {}
    }
}

fn update(_: Event) {
    // let app = PLRender::app();
    // let state = app.state();
    // let mut _renderer = state.renderer::<Window>();
    // let mut _scene = SCENE.get_mut().unwrap();
    // let mut _anim = ANIMATOR.get_mut().unwrap().lock().unwrap();

    // @TODO get it back
    // anim.tick(&mut scene);
    // renderer.render(&scene);
}
