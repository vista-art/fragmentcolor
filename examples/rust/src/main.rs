use plrender::{entities::shapes::Circle, entities::Display, PLRender};

fn main() {
    pollster::block_on(init());
}

async fn init() {
    use rand::Rng;

    let mut plrender = PLRender::default();

    let scene = plrender.create_scene();

    let target = scene.create_target();

    plrender
        .config(Options {
            scenes: {
                Some(SceneOptions {
                    gaze: Some(GazeOptions::default()),
                    ..Default::default()
                })
            },
            ..Default::default()
        })
        .await;

    let plrender1 = plrender.clone();
    std::thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            use log::info;

            std::thread::sleep(std::time::Duration::from_secs(1));
            let x = rng.gen::<f32>() * 2.0 - 1.0;
            let y = rng.gen::<f32>() * 2.0 - 1.0;

            info!("from main: x: {}, y: {}", &x, &y);

            plrender1.trigger("gaze", "set_position", vec![x.to_string(), y.to_string()]);
        }
    });

    plrender.run();
}
