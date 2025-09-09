async fn run() -> Result<(), Box<dyn std::error::Error>> {;

import { Renderer, Pass, Shader } from "fragmentcolor";

const renderer = new Renderer();
let target = renderer.create_texture_target([64, 64]).await?;

const shader = exampleShader();
const pass = new Pass("blend with previous");
pass.add_shader(&shader);
pass.load_previous();

renderer.render(pass, target)?;

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };