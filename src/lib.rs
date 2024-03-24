use pyo3::prelude::*;

mod api;
mod api_commands;
mod testing;
mod testing2;
mod utils;

#[pymodule]
fn rust_via_api(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<api::KeyboardApi>()?;
    Ok(())
}
