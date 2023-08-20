#[cfg(not(target_arch = "wasm32"))]
use pl_video_processor::{
    enrichments::{Enrichments, Gaze},
    Options, Vip,
};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(init());
}

#[cfg(not(target_arch = "wasm32"))]
async fn init() {
    let vip = Vip::new();

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

    vip.run();
}
