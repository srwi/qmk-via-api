use pyo3::prelude::*;

pub mod api;
pub mod api_commands;
pub mod keycodes;
pub mod utils;

#[pymodule]
fn qmk_via_api(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<api::KeyboardApi>()?;
    Ok(())
}
