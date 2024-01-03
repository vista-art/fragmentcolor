# Notes about Pyo3

Check the official docs on [how to customize Python classes](https://pyo3.rs/main/class)

Some quick examples:

```rust
#[pyclass]
struct MyClass {
    #[pyo3(get, set)] // this generates getters and setters
    num: i32,
}
```

Or the method annotation variant:

```rust
#[pyclass]
struct MyClass {
    num: i32,
}

#[pymethods]
impl MyClass {
    #[getter]
    fn num(&self) -> PyResult<i32> {
        Ok(self.num)
    }

    #[setter]
    fn set_num(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
    }
}
```
