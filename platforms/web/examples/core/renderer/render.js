import { Renderer, Shader } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;

const renderer = new Renderer();
let target = renderer.create_texture_target([10, 10]).await?;
const shader = exampleShader();
renderer.render(shader, target)?;

Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };