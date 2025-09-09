from fragmentcolor import Renderer, Pass, Shader, Region

async fn run() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let target = renderer.create_texture_target([64, 64]).await?

shader = example_shader()
pass = Pass("clipped")
pass.add_shader(&shader)

pass.set_viewport(Region::from_region(0, 0, 32, 32))

renderer.render(pass, target)?

Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }