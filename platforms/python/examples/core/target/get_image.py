from fragmentcolor import Renderer, Target

fn main() -> Result<(), Box<dyn std::error::Error>> {

renderer = Renderer()
let target = pollster::block_on(renderer.create_texture_target([16, 16]))?
renderer.render(fragmentcolor::Shader::default(), target)?

let image = target.get_image()
Ok(())
}