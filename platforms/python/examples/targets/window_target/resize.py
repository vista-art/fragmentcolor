from fragmentcolor import Renderer

async fn run() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let mut target = renderer.create_texture_target([64, 32]).await?

assert_eq!(target.size(), [64, 32])

target.resize([128, 64])
assert_eq!(target.size(), [128, 64])

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }