use plr::app::{
    events::Event,
    window::{Window, WindowOptions},
    App,
};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

// @FIXME code generation works partially.
// It's still unrealiable for production.
//
// Example usage:
// use plrender_macros::wrap_py;
// wrap_py!(Camera);
// wrap_py!(Color);
// wrap_py!(Context);
// wrap_py!(Scene);

#[pymodule]
fn plrender(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyApp>()?;
    m.add_class::<PyWindow>()?;
    Ok(())
}

#[pyclass(name = "App")]
pub struct PyApp {
    inner: App,
}

#[pymethods]
impl PyApp {
    #[new] // @TODO config options
    pub fn new() -> PyResult<Self> {
        Ok(Self {
            inner: App::default(),
        })
    }

    pub fn run(&mut self) {
        pollster::block_on(self.inner.run());
    }
}

#[pyclass(name = "Window")]
pub struct PyWindow {
    inner: Window,
}

unsafe impl Send for PyWindow {}

// window = plr.Window(width=400, heigth=300,
//          title="Spritesheet Example", clear_color="#FFccffff")

// #[derive(FromPyObject)]
// pub enum ClearColor<'a> {
//     CssString(&'a str),
//     RgbaTuple(f32, f32, f32, f32),
//     RgbTuple(f32, f32, f32),
//     RgbaDict {
//         r: f32,
//         g: f32,
//         b: f32,
//         a: f32,
//     },
//     RgbDict {
//         r: f32,
//         g: f32,
//         b: f32,
//     },
//     RgbaFullDict {
//         red: f32,
//         green: f32,
//         blue: f32,
//         alpha: f32,
//     },
//     RgbFullDict {
//         red: f32,
//         green: f32,
//         blue: f32,
//     },
// }

#[derive(FromPyObject)]
pub enum WindowSize {
    SizeTuple(u32, u32),
    SizeDict { w: u32, h: u32 },
    SizeFullDict { width: u32, height: u32 },
}

#[pymethods]
impl PyWindow {
    #[new]
    #[pyo3(signature = (app=None, size=WindowSize::SizeTuple(800, 600), title="PLRender"))]
    pub fn new(app: Option<&PyApp>, size: WindowSize, title: &str) -> PyResult<Self> {
        let (width, height) = match size {
            WindowSize::SizeTuple(w, h) => (w, h),
            WindowSize::SizeDict { w, h } => (w, h),
            WindowSize::SizeFullDict { width, height } => (width, height),
        };

        let window = match app {
            Some(app) => Window::new(
                &app.inner,
                WindowOptions {
                    size: Some((width, height)),
                    title: Some(title.to_string()),
                    ..Default::default()
                },
            ),
            None => Ok(Window::default()),
        };

        if let Ok(window) = window {
            Ok(PyWindow { inner: window })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, &str>(
                "Failed to create window",
            ))
        }
    }

    pub fn run(&mut self) {
        pollster::block_on(self.inner.run());
    }

    pub fn on(&mut self, event: &str, callback: PyObject) -> PyResult<()> {
        let caller = move |event: Event| {
            let _ = Python::with_gil(|py| -> PyResult<()> {
                match event {
                    Event::Resize { width, height } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("width", width), ("height", height)].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::Keyboard { key, pressed } => {
                        let key = match key {
                            plr::app::events::Key::Digit(d) => format!("{}", d),
                            plr::app::events::Key::Letter(l) => format!("{}", l),
                            plr::app::events::Key::Function(f) => format!("F{}", f),
                            plr::app::events::Key::Up => "Up".to_string(),
                            plr::app::events::Key::Down => "Down".to_string(),
                            plr::app::events::Key::Left => "Left".to_string(),
                            plr::app::events::Key::Right => "Right".to_string(),
                            plr::app::events::Key::Space => "Space".to_string(),
                            plr::app::events::Key::Escape => "Escape".to_string(),
                            plr::app::events::Key::Other => "Other".to_string(),
                        };

                        let _ = callback.call(
                            py,
                            (),
                            Some(
                                [
                                    ("key", key.to_object(py)),
                                    ("pressed", pressed.to_object(py)),
                                ]
                                .into_py_dict(py),
                            ),
                        );
                        Ok(())
                    }

                    Event::Pointer { x, y } => {
                        let _ = callback.call(py, (), Some([("x", x), ("y", y)].into_py_dict(py)));
                        Ok(())
                    }

                    Event::Scroll { delta_x, delta_y } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("delta_x", delta_x), ("delta_y", delta_y)].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::Click { button, pressed } => {
                        let button = match button {
                            plr::app::events::Button::Left => "Left".to_string(),
                            plr::app::events::Button::Middle => "Middle".to_string(),
                            plr::app::events::Button::Right => "Right".to_string(),
                            plr::app::events::Button::Other(o) => format!("{}", o),
                        };

                        let _ = callback.call(
                            py,
                            (),
                            Some(
                                [
                                    ("button", button.to_object(py)),
                                    ("pressed", pressed.to_object(py)),
                                ]
                                .into_py_dict(py),
                            ),
                        );
                        Ok(())
                    }

                    Event::Draw => {
                        let _ = callback.call(py, (), None);
                        Ok(())
                    }

                    Event::Exit => {
                        let _ = callback.call(py, (), None);
                        Ok(())
                    }
                }
            });
        };
        self.inner.on(event, Box::new(caller));
        Ok(())
    }
}
