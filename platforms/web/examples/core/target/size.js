import { Renderer, Target } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;

const renderer = new Renderer();
let target = renderer.create_texture_target([64, 32]).await?;
let size: [u32; 2] = target.size().into();
assert_eq!(size, [64, 32]);

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };