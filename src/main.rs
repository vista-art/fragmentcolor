#[cfg(not(wasm))]
use pl_video_processor::{
    controllers::{gaze::GazeOptions, ControllerOptions},
    Options, Vip,
};

fn main() {
    #[cfg(not(wasm))]
    pollster::block_on(init());
}

#[cfg(not(wasm))]
async fn init() {
    use rand::Rng;

    let mut vip = Vip::new();

    vip.config(Options {
        controllers: {
            Some(ControllerOptions {
                gaze: Some(GazeOptions::default()),
                ..Default::default()
            })
        },
        ..Default::default()
    })
    .await;

    let vip1 = vip.clone();
    std::thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let x = rng.gen::<f32>() * 2.0 - 1.0;
            let y = rng.gen::<f32>() * 2.0 - 1.0;

            vip1.set_normalized_position(x, y);
        }
    });

    vip.run();
}
