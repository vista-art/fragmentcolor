#[cfg(not(target_arch = "wasm32"))]
use pl_video_processor::{
    enrichments::{gaze::Gaze, Enrichments},
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
            Enrichments {
                gaze: Some(Gaze {
                    radius: 0.2,
                    border: 0.05,
                    color: "#ff0000ff".to_string(),
                    alpha: 1.0,
                }),
                ..Default::default()
            }
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
