use fragmentcolor::{Pass, Renderer, Shader};

// Story: Render a single pass holding two shaders, then render a sequence of two passes.
// Arrange / Act / Assert structure keeps the flow readable.
#[test]
fn renders_single_pass_then_sequence_of_passes() -> Result<(), Box<dyn std::error::Error>> {
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

    // Act 2: build a second pass and render the sequence directly (Pass slice is Renderable).
    let pass2 = Pass::new("Second Pass");
    pass2.add_shader(&s1);
    pass2.add_shader(&s2);

    // Assert: render the sequence without errors
    renderer.render(&vec![pass1, pass2], &target)?;

    Ok(())
}
