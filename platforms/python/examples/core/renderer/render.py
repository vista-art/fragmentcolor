from fragmentcolor import Renderer, Shader

async fn run() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let target = renderer.create_texture_target([10, 10]).await?
shader = example_shader()
renderer.render(shader, target)?

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }