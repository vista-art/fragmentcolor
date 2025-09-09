import { Renderer } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;

const renderer = new Renderer();
let mut target = renderer.create_texture_target([64, 64]).await?;

assert_eq!(target.size(), [64, 64]);

target.resize([128, 32]);
assert_eq!(target.size(), [128, 32]);

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };