use fragmentcolor::{Frame, Pass, Renderer, Shader};

#[test]
fn test_api() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target(&[10, 10]))?;
    let object1 = Shader::default();
    let object2 = Shader::default();

    let pass = Pass::new("First Pass");
    pass.add_shader(&object1);
    pass.add_shader(&object2);

    renderer.render(&pass, &target)?;

    let pass2 = Pass::new("Second Pass");
    pass2.add_shader(&object1);
    pass2.add_shader(&object2);

    let mut frame = Frame::new();
    frame.add_pass(&pass);
    frame.add_pass(&pass2);

    renderer.render(&frame, &target)?;

    Ok(())
}
