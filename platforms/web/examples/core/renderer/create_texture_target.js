import { Renderer, Shader } from "fragmentcolor";

async fn run() -> Result<(), Box<dyn std::error::Error>> {;
const renderer = new Renderer();
let target = renderer.create_texture_target([64, 64]).await?;
const shader = exampleShader();
renderer.render(shader, target)?;
let _image = target.get_image();
Ok(());
};
fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) };