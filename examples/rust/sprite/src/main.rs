use instant::{Duration, Instant};
use plrender::{
    animation::Animator,
    renderer::{RenderOptions, Renderer},
};

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
    use plrender::app::window::{IsWindow, Window};

    // You can use the App to create multiple Windows
    // let app = plrender::App::default();
    // let window1 = plrender::Window::new(&app, WindowOptions::default());
    // let window2 = plrender::Window::new(&app, WindowOptions::default());

    let window = Window::default().set_title("Sprite").set_size((800, 600));
    let mut scene = plrender::Scene::new();

    scene.add_target(window);

    // Renderer is implicitly created by Window::default() or App::add_window()
    // let mut renderer = Renderer::new(RenderOptions {
    //     targets: Some(vec![window]),
    //     ..Default::default()
    // })
    // .await
    // .unwrap();

    // this has to be part of the scene and created
    // by default without user input.
    //
    // but how to pass the right arguments to it?
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
    let mut pass = plrender::renderer::renderpass::Flat2D::new(&mut renderer);

    // It's currently like this:
    let image = renderer
        .load_image(format!(
            "{}/assets/images/pickachu.png",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

    let sprite = scene.add_sprite(image).build();

    // @TODO But should be like this:
    // let sprite = plrender::Sprite::new(
    //     &mut renderer,
    //     format!(
    //         "{}{}",
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/assets/images/pickachu.png"
    //     ),
    // );
    //
    // scene.add(sprite);

    let mut anim = Animator {
        map: plrender::asset::SpriteMap {
            origin: mint::Point2 { x: 0, y: 0 },
            cell_size: mint::Vector2 { x: 96, y: 96 },
        },
        cell_counts: mint::Vector2 { x: 5, y: 13 },
        duration: Duration::from_secs_f64(0.1),
        current: mint::Point2 { x: 0, y: 0 },
        moment: Instant::now(),
        sprite,
    };
    anim.switch(State::Idle as usize, &mut scene);

    // @TODO
    window.on("resize", |Event::Resize { width, height }| {
        let target = renderer.targets().get_mut(window.id()).unwrap();
        target.resize(
            &renderer,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    });

    window.on("keydown", |Event::Keyboard { key, pressed: true }| {
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
    });

    window.on("draw", || {
        anim.tick(&mut scene);
        //                                 @TODO
        renderer.render(&mut pass, &scene, &scene.camera());
    });
}
