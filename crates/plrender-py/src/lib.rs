pub use pl::{
    window::Window, Camera, Color, Entity, EntityRef, Light, LightBuilder, LightRef, MeshBuilder,
    MeshRef, Node, NodeRef, Projection, Prototype, RenderPass, Renderer, Scene, Sprite,
    SpriteBuilder, TargetInfo, TargetRef, TextureRef, UvRange,
};
use pyo3::prelude::*;

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
fn plrender(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyWindow>()?;
    Ok(())
}

// Context
#[pyclass(name = "Window")]
pub struct PyWindow {
    inner: Window,
}

unsafe impl Send for PyWindow {}

// window = plr.Window(width=400, heigth=300,
//          title="Spritesheet Example", clear_color="#FFccffff")

#[derive(FromPyObject)]
enum ClearColor {
    CssString(String),
    RgbaTuple(f32, f32, f32, f32),
    RgbTuple(f32, f32, f32),
    RgbaDict {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    RgbDict {
        r: f32,
        g: f32,
        b: f32,
    },
    RgbaFullDict {
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    },
    RgbFullDict {
        red: f32,
        green: f32,
        blue: f32,
    },
}

#[derive(FromPyObject)]
enum WindowSize {
    SizeTuple(u32, u32),
    SizeDict { w: u32, h: u32 },
    SizeFullDict { width: u32, height: u32 },
}

#[pymethods]
impl PyWindow {
    #[new]
    #[pyo3(signature = (size=(800, 600), title="PLRender", clear_color="#aaccffff"))]
    fn new(size: Py, title: &str, clear_color: PyAny) -> PyResult<Self> {
        let (width, height) = match size {
            WindowSize::SizeTuple(w, h) => (w, h),
            WindowSize::SizeDict { w, h } => (w, h),
            WindowSize::SizeFullDict { width, height } => (width, height),
        };

        let window = Window::new().title("PLRender").build();

        Ok(PyWindow { inner: window })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.inner.resize(width, height);
    }
}

// PYO3 EXAMPLES

// Conversion from Python union types
// #[derive(FromPyObject)]
// enum RustyEnum<'a> {
//     Int(usize),                    // input is a positive int
//     String(String),                // input is a string
//     IntTuple(usize, usize),        // input is a 2-tuple with positive ints
//     StringIntTuple(String, usize), // input is a 2-tuple with String and int
//     Coordinates3d {
//         // needs to be in front of 2d
//         x: usize,
//         y: usize,
//         z: usize,
//     },
//     Coordinates2d {
//         // only gets checked if the input did not have `z`
//         #[pyo3(attribute("x"))]
//         a: usize,
//         #[pyo3(attribute("y"))]
//         b: usize,
//     },
//     #[pyo3(transparent)]
//     CatchAll(&'a PyAny), // This extraction never fails
// }

// use pyo3::types::{PyDict, PyTuple};
// #[pymethods]
// impl MyClass {
//     #[new]
//     #[pyo3(signature = (num=-1))]
//     fn new(num: i32) -> Self {
//         MyClass { num }
//     }

//     #[pyo3(signature = (num=10, *py_args, name="Hello", **py_kwargs))]
//     fn method(
//         &mut self,
//         num: i32,
//         py_args: &PyTuple,
//         name: &str,
//         py_kwargs: Option<&PyDict>,
//     ) -> String {
//         let num_before = self.num;
//         self.num = num;
//         format!(
//             "num={} (was previously={}), py_args={:?}, name={}, py_kwargs={:?} ",
//             num, num_before, py_args, name, py_kwargs,
//         )
//     }

//     fn make_change(&mut self, num: i32) -> PyResult<String> {
//         self.num = num;
//         Ok(format!("num={}", self.num))
//     }
// }

//  * Just like in Python, the following constructs can be part of the signature:

//     /:  positional-only arguments separator, each parameter defined before /
//         is a positional-only parameter.

//     *:  var arguments separator, each parameter defined after *
//         is a keyword-only parameter.

//     *args: "args" is var args. Type of the args parameter has to be &PyTuple.

//     **kwargs:   "kwargs" receives keyword arguments. The type of the kwargs
//                 parameter has to be Option<&PyDict>.

//     arg=Value:  arguments with default value. If the arg argument is defined
//                 after var arguments, it is treated as a keyword-only argument. Note that Value has to be valid rust code, PyO3 just inserts it into the generated code unmodified.
