use plr::app::{
    events::Event,
    window::{Window, WindowOptions},
    App, PLRender,
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
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}

#[pyfunction]
fn run() {
    pollster::block_on(PLRender::run());
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

#[derive(FromPyObject)]
pub enum WindowSize {
    SizeTuple(u32, u32),
    SizeDict { w: u32, h: u32 },
    SizeFullDict { width: u32, height: u32 },
}

#[pymethods]
impl PyWindow {
    #[new]
    #[pyo3(signature = (size=WindowSize::SizeTuple(800, 600), title="PLRender"))]
    pub fn new(size: WindowSize, title: &str) -> PyResult<Self> {
        let (width, height) = match size {
            WindowSize::SizeTuple(w, h) => (w, h),
            WindowSize::SizeDict { w, h } => (w, h),
            WindowSize::SizeFullDict { width, height } => (width, height),
        };

        let window = Window::new(WindowOptions {
            size: (width, height),
            title: title.to_string(),
            ..Default::default()
        });

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

    pub fn on(&mut self, event_name: &str, callback: PyObject) -> PyResult<()> {
        let caller = move |event: Event| {
            let _ = Python::with_gil(|py| -> PyResult<()> {
                match event {
                    Event::Resized { width, height } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("width", width), ("height", height)].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::Rescaled { width, height } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("width", width), ("height", height)].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::Moved { x, y } => {
                        let _ = callback.call(py, (), Some([("x", x), ("y", y)].into_py_dict(py)));
                        Ok(())
                    }

                    Event::KeyUp { key, scancode } => {
                        let key = if let Some(keycode) = key {
                            format!("{:?}", keycode)
                        } else {
                            "None".to_string()
                        };

                        let _ = callback.call(
                            py,
                            (),
                            Some(
                                [
                                    ("key", key.to_object(py)),
                                    ("scancode", scancode.to_object(py)),
                                ]
                                .into_py_dict(py),
                            ),
                        );
                        Ok(())
                    }

                    Event::KeyDown { key, scancode } => {
                        let key = if let Some(keycode) = key {
                            format!("{:?}", keycode)
                        } else {
                            "None".to_string()
                        };

                        let _ = callback.call(
                            py,
                            (),
                            Some(
                                [
                                    ("key", key.to_object(py)),
                                    ("scancode", scancode.to_object(py)),
                                ]
                                .into_py_dict(py),
                            ),
                        );
                        Ok(())
                    }

                    Event::Character { character } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("character", character.to_object(py))].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::CursorEntered => {
                        let _ = callback.call(py, (), None);
                        Ok(())
                    }

                    Event::CursorLeft => {
                        let _ = callback.call(py, (), None);
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
                        let button = format!("{:?}", button);

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

                    Event::FileHovered { handle } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("handle", handle.to_object(py))].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::FileDropped { handle } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("handle", handle.to_object(py))].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::FileHoverCancelled => {
                        let _ = callback.call(py, (), None);
                        Ok(())
                    }

                    Event::Focus { focused } => {
                        let _ = callback.call(
                            py,
                            (),
                            Some([("focused", focused.to_object(py))].into_py_dict(py)),
                        );
                        Ok(())
                    }

                    Event::Closed => {
                        let _ = callback.call(py, (), None);
                        Ok(())
                    }

                    Event::Destroyed => {
                        let _ = callback.call(py, (), None);
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
        self.inner.on(event_name, Box::new(caller));
        Ok(())
    }
}
