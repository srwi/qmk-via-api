pub mod api;
pub mod api_commands;
pub mod error;
pub mod keycodes;
pub mod scan;
pub mod utils;

#[cfg(feature = "python")]
use pyo3::create_exception;
#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::prelude::*;

pub use error::*;

#[cfg(feature = "python")]
create_exception!(qmk_via_api, QmkViaError, PyException);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, HidError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, DeviceNotFoundError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, UnsupportedProtocolError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, SizeMismatchError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, CommandResponseError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, InvalidArgumentError, QmkViaError);

#[cfg(feature = "python")]
#[pymodule]
fn qmk_via_api(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<api::KeyboardApi>()?;
    m.add_class::<api_commands::ViaCommandId>()?;
    m.add_class::<api::MatrixInfo>()?;
    m.add_class::<scan::KeyboardDeviceInfo>()?;
    m.add("QmkViaError", _py.get_type::<QmkViaError>())?;
    m.add("HidError", _py.get_type::<HidError>())?;
    m.add("DeviceNotFoundError", _py.get_type::<DeviceNotFoundError>())?;
    m.add(
        "UnsupportedProtocolError",
        _py.get_type::<UnsupportedProtocolError>(),
    )?;
    m.add("SizeMismatchError", _py.get_type::<SizeMismatchError>())?;
    m.add(
        "CommandResponseError",
        _py.get_type::<CommandResponseError>(),
    )?;
    m.add(
        "InvalidArgumentError",
        _py.get_type::<InvalidArgumentError>(),
    )?;
    m.add_function(wrap_pyfunction!(scan::scan_keyboards, m)?)?;
    Ok(())
}
