pub mod api;
pub mod api_commands;
pub mod error;
pub mod keycodes;
pub mod quantum;
pub mod scan;
pub mod utils;

#[cfg(feature = "python")]
use pyo3::create_exception;
#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::prelude::*;

pub use api::{KeyboardApi, MatrixInfo, QmkFeatures};
pub use error::*;
pub use keycodes::{Keycode, KeycodeCategory};
pub use quantum::{
    encode_layer_mod, encode_layer_tap, encode_mod_combo, encode_mod_tap, encode_one_shot_mod,
    ranges, QmkKeycode, QmkLayerOp, QmkModMask,
};

#[cfg(feature = "python")]
create_exception!(qmk_via_api, QmkViaError, PyException);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, HidError, QmkViaError);
#[cfg(feature = "python")]
create_exception!(qmk_via_api, MaybePermissionDeniedError, QmkViaError);
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
    m.add_class::<api::QmkFeatures>()?;
    m.add_class::<scan::KeyboardDeviceInfo>()?;
    m.add_class::<keycodes::Keycode>()?;
    m.add_class::<keycodes::KeycodeCategory>()?;
    m.add_class::<quantum::QmkModMask>()?;
    m.add_class::<quantum::QmkLayerOp>()?;
    m.add_class::<quantum::QmkKeycode>()?;
    m.add("QmkViaError", _py.get_type::<QmkViaError>())?;
    m.add("HidError", _py.get_type::<HidError>())?;
    m.add(
        "MaybePermissionDeniedError",
        _py.get_type::<MaybePermissionDeniedError>(),
    )?;
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
    m.add_function(wrap_pyfunction!(scan::check_hid_permissions, m)?)?;
    m.add_function(wrap_pyfunction!(quantum::encode_mod_combo, m)?)?;
    m.add_function(wrap_pyfunction!(quantum::encode_mod_tap, m)?)?;
    m.add_function(wrap_pyfunction!(quantum::encode_layer_tap, m)?)?;
    m.add_function(wrap_pyfunction!(quantum::encode_layer_mod, m)?)?;
    m.add_function(wrap_pyfunction!(quantum::encode_one_shot_mod, m)?)?;
    Ok(())
}
