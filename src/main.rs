#[cfg(not(target_arch = "wasm32"))]
use pl_video_processor::{
    enrichments::{gaze::GazeOptions, EnrichmentOptions},
    Options, Vip,
};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(init());
}

#[cfg(not(target_arch = "wasm32"))]
async fn init() {
    use rand::Rng;

    let mut vip = Vip::new();
    let vip1 = vip.clone();

    vip.config(Options {
        enrichments: {
            Some(EnrichmentOptions {
                gaze: Some(GazeOptions::default()),
                ..Default::default()
            })
        },
        ..Default::default()
    })
    .await;

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
