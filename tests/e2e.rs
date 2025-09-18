use fragmentcolor::{Frame, Pass, Renderer, Shader};

// Story: Render two shaders in a first pass, then render the same two in a second pass within a frame.
// Arrange / Act / Assert structure keeps the flow readable.
#[test]
fn renders_two_passes_then_frame() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target([10, 10]))?;
    let s1 = Shader::default();
    let s2 = Shader::default();

    // Act 1: render a single pass with two shaders
    let pass1 = Pass::new("First Pass");
    pass1.add_shader(&s1);
    pass1.add_shader(&s2);
    renderer.render(&pass1, &target)?;

    // Act 2: build a second pass and a frame containing both
    let pass2 = Pass::new("Second Pass");
    pass2.add_shader(&s1);
    pass2.add_shader(&s2);

    let mut frame = Frame::new();
    frame.add_pass(&pass1);
    frame.add_pass(&pass2);

    // Assert: render the frame without errors
    renderer.render(&frame, &target)?;

    Ok(())
}
