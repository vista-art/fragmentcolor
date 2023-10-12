static API_MAP: phf::Map<&'static str, &'static str> = {
    phf::phf_map! {
        "WindowBuilder" => "fn title(title: &str) -> Self; fn size(width: u32, height: u32) -> Self; fn vsync(vsync: bool) -> Self; fn build(self) -> Result<Window, WindowBuildError>;",
        "Window" => "fn run(&mut self, runner: impl FnMut(Event) + 'static) -> !;",
        "Event" => "fn resize(width: u32, height: u32); fn mouse_move(x: f32, y: f32); fn mouse_down(button: MouseButton); fn mouse_up(button: MouseButton); fn scroll(delta: mint::Vector2<f32>); fn draw; fn exit;",
        "MouseButton" => "fn Left; fn Right; fn Middle;",
        "WindowBuildError" => "fn OsError(std::io::Error); fn GlError(glutin::ContextError);",
    }
};
