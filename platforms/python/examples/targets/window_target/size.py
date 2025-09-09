from fragmentcolor import Renderer, Shader

async fn run() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let target = renderer.create_texture_target([64, 32]).await?
assert_eq!(target.size(), [64, 32])

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }