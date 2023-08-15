use pl_video_processor::{run, Options};

fn main() {
    pollster::block_on(run(Options::default()));
}
