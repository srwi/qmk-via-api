use pyo3::prelude::*;

mod api;
mod api_commands;
mod utils;

#[pymodule]
fn rust_via_api(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<api::QmkKeyboardApi>()?;
    Ok(())
}
