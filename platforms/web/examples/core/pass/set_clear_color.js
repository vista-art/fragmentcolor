import { Renderer, Pass, Shader } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;

const renderer = new Renderer();
let target = renderer.create_texture_target([64, 64]).await?;

const shader = exampleShader();
const pass = new Pass("solid background");
pass.add_shader(&shader);

pass.set_clear_color([0.1, 0.2, 0.3, 1.0]);

renderer.render(pass, target)?;

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };