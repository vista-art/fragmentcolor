use std::sync::{Arc, Mutex, OnceLock};

use instant::{Duration, Instant};
use plrender::{
    app::events::VirtualKey as Key,
    app::window::Window,
    components::{Animator, SpriteMap},
    math::linear_algebra::{Point2, Vec2},
    scene::{ObjectId, Scene},
    Event, Sprite,
};

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Pikachu {
    Idle = 0,
    MoveRight = 9,
    MoveLeft = 8,
    Kick = 4,
    Jump = 10,
    Lie = 12,
}

impl Default for Pikachu {
    fn default() -> Self {
        Self::Idle
    }
}

static SCENE: OnceLock<Scene> = OnceLock::new();
static ANIMATOR: OnceLock<Arc<Mutex<Animator>>> = OnceLock::new();

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let mut window = Window::default()
        .set_title("Sprite")
        .set_size((800, 600))
        .clone();

    let mut scene = Scene::new();

    let mut sprite = Sprite::new("assets/images/pikachu.png");
    sprite.set_position([0.0, 0.0, 0.0].into());
    sprite.set_scale([0.5, 0.5, 0.5].into());
    scene.add(&mut sprite);

    let anim = ANIMATOR.get_or_init(|| {
        Arc::new(Mutex::new(Animator {
            scene: scene.state(),
            sprite_map: SpriteMap {
                origin: Point2 { x: 0, y: 0 },
                cell_size: Vec2 { x: 96, y: 96 },
            },
            cell_counts: Vec2 { x: 5, y: 13 },
            duration: Duration::from_secs_f64(0.1),
            current: Point2 { x: 0, y: 0 },
            moment: Instant::now(),
            sprite: ObjectId::DANGLING,
        }))
    });

    let mut anim = anim.lock().unwrap();
    anim.sprite = sprite.id().unwrap();
    anim.switch::<usize>(Pikachu::Idle as usize);

    window.on("keydown", on_keydown);
    window.on("draw", on_draw);

    window.run().await;
}

fn on_keydown(event: Event) {
    let anim = ANIMATOR.get().unwrap();
    let mut anim = anim.lock().unwrap();

    match event {
        Event::KeyDown {
            key: is_pressed, ..
        } => {
            if let Some(key) = is_pressed {
                let new_state = match key {
                    Key::Up => Some(Pikachu::Jump),
                    Key::Down => Some(Pikachu::Lie),
                    Key::Space => Some(Pikachu::Kick),
                    Key::Left => Some(Pikachu::MoveLeft),
                    Key::Right => Some(Pikachu::MoveRight),
                    _ => None,
                };

                if let Some(state) = new_state {
                    if anim.current.y != state as usize || state == Pikachu::Kick {
                        anim.switch::<usize>(state as usize);
                    }
                }
            }
        }
        _ => {}
    }
}

fn on_draw(_: Event) {
    let anim = ANIMATOR.get().unwrap();
    let mut anim = anim.lock().unwrap();
    anim.tick();

    let scene = SCENE.get().unwrap();
    _ = scene.render();
}
