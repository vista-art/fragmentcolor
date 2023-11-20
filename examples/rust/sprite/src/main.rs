use instant::{Duration, Instant};
use plrender::{
    app::events::VirtualKey as Key,
    app::window::Window,
    components::{Animator, SpriteMap},
    math::cg::Pixel,
    scene::Scene,
    Event, IsWindow, Sprite,
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
    sprite.set_position([0.0, 0.0, 0.0]);
    sprite.set_scale([0.5, 0.5, 0.5]);
    scene.add(&mut sprite);

    let mut anim = Animator {
        scene: scene.state(),
        sprite_map: SpriteMap {
            origin: Pixel { x: 0, y: 0 },
            cell_size: Pixel { x: 96, y: 96 },
        },
        cell_counts: Pixel { x: 5, y: 13 },
        duration: Duration::from_secs_f64(0.1),
        current: Pixel { x: 0, y: 0 },
        moment: Instant::now(),
        sprite: sprite.id().unwrap(),
    };
    anim.switch(Pikachu::Idle as u16);

    let state = window.state();
    window.on("any", move |event| match event {
        Event::KeyDown {
            key: is_pressed,
            keycode: _,
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
                    if anim.current.y != state as u16 || state == Pikachu::Kick {
                        anim.switch(state as u16);
                    }
                }
            };
        }

        Event::Draw => {
            let window = state.read().unwrap();
            anim.tick();
            _ = scene.render();
            window.redraw();
        }
        _ => {}
    });

    window.run();
}
