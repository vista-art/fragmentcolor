// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis/

/// A Frame is a collection of passes that are executed in sequence.
///
struct Frame {
    renderer: Arc<Renderer>,
    passes: Vec<Pass>, // @TODO check your solution to Photoroom Challenge for DAG
}

impl Frame {
    fn new(renderer: Arc<Renderer>) -> Self {
        Self {
            renderer: renderer.clone(),
            passes: Vec::new(),
        }
    }

    fn create_pass(&mut self, pass_type: PassType, name: String, shader: String) {
        let pass = Pass::new(self.renderer.clone(), pass_type, name, shader);
        self.passes.push(pass);
    }

    fn create_render_pass(&mut self, name: String, shader: String) {
        self.create_pass(PassType::Render, name, shader);
    }

    fn create_compute_pass(&mut self, name: String, shader: String) {
        self.create_pass(PassType::Compute, name, shader);
    }

    fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }
}
