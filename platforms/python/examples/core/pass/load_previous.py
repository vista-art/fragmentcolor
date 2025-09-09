async fn run() -> Result<(), Box<dyn std::error::Error>> {

from fragmentcolor import Renderer, Pass, Shader

renderer = Renderer()
let target = renderer.create_texture_target([64, 64]).await?

shader = example_shader()
pass = Pass("blend with previous")
pass.add_shader(&shader)
pass.load_previous()

renderer.render(pass, target)?

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }