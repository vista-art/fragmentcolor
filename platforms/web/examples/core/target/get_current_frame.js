import { Renderer, Target } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;

const renderer = new Renderer();
let target = renderer.create_texture_target([16, 16]).await?;
let frame = target.get_current_frame()?; // Acquire a frame;
let _format = frame.format();

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };