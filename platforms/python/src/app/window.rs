use plr::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(name = "Window")]
pub struct PyWindow {
    inner: Window,
}

unsafe impl Send for PyWindow {}

#[derive(FromPyObject)]
pub enum WindowSize {
    SizeTuple(u32, u32),
    SizeArray([u32; 2]),
    SizeDict { width: u32, height: u32 },
}

#[pymethods]
impl PyWindow {
    #[new]
    #[pyo3(signature = (size=WindowSize::SizeTuple(800, 600), title="FragmentColor"))]
    pub fn new(size: WindowSize, title: &str) -> PyResult<Self> {
        let (width, height) = match size {
            WindowSize::SizeTuple(w, h) => (w, h),
            WindowSize::SizeArray([w, h]) => (w, h),
            WindowSize::SizeDict { width, height } => (width, height),
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
        self.inner.run();
    }

    pub fn on(&mut self, event_name: &str, callback: PyObject) -> PyResult<()> {
        let window_id = self.inner.id();
        let caller = move |event: Event| {
            let _ = Python::with_gil(|py| -> PyResult<PyObject> {
                match event {
                    Event::Resized { width, height } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("width", width)?;
                        kwargs.set_item("height", height)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Rescaled {
                        scale,
                        width,
                        height,
                    } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("scale", scale)?;
                        kwargs.set_item("width", width)?;
                        kwargs.set_item("height", height)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Moved { x, y } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("x", x)?;
                        kwargs.set_item("y", y)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::KeyUp { key, keycode } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("key", parse(key))?;
                        kwargs.set_item("scancode", keycode)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::KeyDown { key, keycode } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("key", parse(key))?;
                        kwargs.set_item("scancode", keycode)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Character { character } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("character", character)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::CursorEntered => callback.call(py, (), None),

                    Event::CursorLeft => callback.call(py, (), None),

                    Event::Pointer { x, y } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("x", x)?;
                        kwargs.set_item("y", y)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Scroll { delta_x, delta_y } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("delta_x", delta_x)?;
                        kwargs.set_item("delta_y", delta_y)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Click { button, pressed } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("button", format!("{:?}", button))?;
                        kwargs.set_item("pressed", pressed)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::FileHovered { handle } => {
                        let filename = get_hovered_filename(&window_id, handle);
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("filename", filename)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::FileDropped { handle } => {
                        let filename = get_dropped_filename(&window_id, handle);
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("filename", filename)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::FileHoverCancelled => callback.call(py, (), None),

                    Event::Focus { focused } => {
                        let kwargs = PyDict::new(py);
                        kwargs.set_item("focused", focused)?;

                        callback.call(py, (), Some(kwargs))
                    }

                    Event::Closed => callback.call(py, (), None),

                    Event::Destroyed => callback.call(py, (), None),

                    Event::Draw => callback.call(py, (), None),

                    Event::Exit => callback.call(py, (), None),

                    Event::Command(command) => match command {
                        plr::Command::Log { level, message } => {
                            let kwargs = PyDict::new(py);
                            kwargs.set_item("level", level)?;
                            kwargs.set_item("message", message)?;

                            callback.call(py, (), Some(kwargs))
                        }
                        plr::Command::Print { message } => {
                            let kwargs = PyDict::new(py);
                            kwargs.set_item("message", message)?;

                            callback.call(py, (), Some(kwargs))
                        }
                    },
                }
            });
        };
        self.inner.on(event_name, std::boxed::Box::new(caller));
        Ok(())
    }
}

// Converts a virtual KeyCode to String
fn parse(key: Option<VirtualKey>) -> String {
    if let Some(keycode) = key {
        format!("{:?}", keycode)
    } else {
        "None".to_string()
    }
}

// Note about the conversions below:
//
// FragmentColor can't send filenames directly to callbacks because
// Events must be safe to copy with `memcpy` and share between
// threads. This means their data must be know at compile time,
// so the Window produces a u128 nanosecond timestamp instead
// of a filename, and uses this timestamp as a key to a map
// of filenames.
//
// Here we can safely convert the timestamp back to a filename
// and send to Python back as strings.

/// Helper method to get the hovered filename by handle.
/// Ugly, but avoids panics & deadlocks, and covers all edge cases.
fn get_hovered_filename(window_id: &WindowId, handle: u128) -> String {
    let app = FragmentColor::app()
        .read()
        .expect("Failed to acquire App Read Lock");
    let windows = app
        .windows()
        .read()
        .expect("Failed to acquire Windows collection Read Lock");

    let filename = if let Some(window) = windows.get(window_id) {
        if let Some(filename) = window.get_hovered_file(handle) {
            filename
        } else {
            app.error("fragmentcolor-py: Cannot read filename: ignoring dropped file!");
            "None".to_string()
        }
    } else {
        app.error("fragmentcolor-py: Cannot read Window: ignoring hovered file!");
        "None".to_string()
    };

    filename
}

// Same as above, but for dropped files.
// The difference is that it needs to take a mutable reference to the window,
// because the file is removed from the internal list after it's read.
fn get_dropped_filename(window_id: &WindowId, handle: u128) -> String {
    // @TODO Windows and Scenes should be globally accessible by name.
    // Example: FragmentColor::get_window(name)
    let app = FragmentColor::app()
        .read()
        .expect("Failed to acquire App Read Lock");
    let windows = app
        .windows()
        .write()
        .expect("Failed to acquire Windows collection Write Lock");

    let filename = if let Some(mut window) = windows.get_mut(window_id) {
        if let Some(filename) = window.get_dropped_file(handle) {
            filename.to_string_lossy().to_string()
        } else {
            app.error("fragmentcolor-py: Cannot read filename: ignoring dropped file!");
            "None".to_string()
        }
    } else {
        app.error("fragmentcolor-py: Cannot read Window: ignoring dropped file!");
        "None".to_string()
    };

    filename
}
