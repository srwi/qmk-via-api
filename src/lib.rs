pub mod api;
pub mod api_commands;
pub mod error;
pub mod keycodes;
pub mod scan;
pub mod utils;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub use error::*;

#[cfg(feature = "python")]
#[pymodule]
fn qmk_via_api(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<api::KeyboardApi>()?;
    m.add_class::<scan::KeyboardDeviceInfo>()?;
    m.add_function(wrap_pyfunction!(scan::scan_keyboards, m)?)?;
    Ok(())
}