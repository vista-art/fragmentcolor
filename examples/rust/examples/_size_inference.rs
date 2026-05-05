use fragmentcolor::{Renderer, TextureFormat};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let r = Renderer::new();
    // Test 1: bare integer literals in tuple form
    let _t1 = r
        .create_storage_texture(([64, 64], TextureFormat::Rgba))
        .await?;
    // Test 2: with pixels
    let pixels: &[u8] = &[255, 255, 255, 255];
    let _t2 = r.create_texture((pixels, [1, 1])).await?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(run())
}
