use plrender::{
    controllers::{gaze::GazeOptions, ControllerOptions},
    Options, PLRender,
};

fn main() {
    pollster::block_on(init());
}

async fn init() {
    use rand::Rng;

    let mut plrender = PLRender::new();

    plrender
        .config(Options {
            controllers: {
                Some(ControllerOptions {
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
