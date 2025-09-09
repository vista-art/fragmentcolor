from fragmentcolor import Renderer

async fn run() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let target = renderer.create_texture_target([16, 16]).await?
renderer.render(fragmentcolor::Shader::default(), target)?

let image = target.get_image()

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }