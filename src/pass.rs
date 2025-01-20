// New API
enum PassType {
    Render,
    Compute,
}

/// Note to Self:
/// When I think about render passes, I think about the dataflow across the entire frame.
///
/// passes produce and consume Render Targets,
/// and inside of each Pass is a list of draw calls.
struct Pass {
    renderer: Arc<Renderer>,
    pass_type: PassType,
    name: String,
    shader: String,
    targets: Vec<Texture>,
}

impl Pass {
    fn new(renderer: Arc<Renderer>, pass_type: PassType, name: String, shader: String) -> Self {
        Self {
            renderer: renderer.clone(),
            pass_type,
            name,
            shader,
            targets: Vec::new(),
        }
    }

    fn add_target(&mut self, target: Texture) {
        self.targets.push(target);
    }

    fn draw(&self, scene: &Scene, camera: &Camera) {
        // Draw the pass
    }

    fn compute(&self, scene: &Scene, camera: &Camera) {
        // Compute the pass
    }
}
