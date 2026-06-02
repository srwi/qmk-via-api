use std::fmt::Debug;

use hidapi::HidError;

use crate::api_commands::ViaCommandId;

pub type Result<T> = core::result::Result<T, Error>;

pub const MAYBE_PERMISSION_DENIED_MESSAGE: &str = "No HID devices were visible through hidapi. This can mean no HID devices are connected, but it can also indicate that this process does not have permission to enumerate HID devices. On Linux, check that the user has access to hidraw devices or install the appropriate udev rules for the keyboard.";

#[derive(Debug)]
pub enum Error {
    Hid(String),
    MaybePermissionDenied(String),
    BadCommandResponse(ViaCommandId),
    SendCommand(ViaCommandId, String),
    NoSuchKeyboard {
        vid: u16,
        pid: u16,
        usage_page: u16,
    },
    UnsupportedProtocol(u16),
    UnsupportedFeature(&'static str),
    SizeMismatch {
        expected: usize,
        actual: usize,
        context: &'static str,
    },
    InvalidArgument(&'static str),
}

impl Error {
    pub fn size_mismatch(context: &'static str, expected: usize, actual: usize) -> Self {
        Error::SizeMismatch {
            expected,
            actual,
            context,
        }
    }

    pub fn maybe_permission_denied() -> Self {
        Error::MaybePermissionDenied(MAYBE_PERMISSION_DENIED_MESSAGE.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SizeMismatch {
                expected,
                actual,
                context,
            } => f.write_fmt(format_args!(
                "{}: expected size = {}, actual size = {}",
                context, expected, actual
            )),
            Error::UnsupportedProtocol(version) => {
                f.write_fmt(format_args!("unsupported protocol version: {}", version))
            }
            Error::UnsupportedFeature(feature) => {
                f.write_fmt(format_args!("unsupported feature: {}", feature))
            }
            Error::NoSuchKeyboard {
                vid,
                pid,
                usage_page,
            } => f.write_fmt(format_args!(
                "could not find keyboard: 0x{:04x}/0x{:04x}/0x{:04x} (vendor_id/product_id/usage_page)",
                vid, pid, usage_page
            )),
            Error::BadCommandResponse(cmd) => f.write_fmt(format_args!(
                "unexpected command response for command {:?}",
                cmd
            )),
            Error::InvalidArgument(arg) => f.write_fmt(format_args!("invalid argument: {}", arg)),
            Error::MaybePermissionDenied(msg) => f.write_str(msg),
            _ => Debug::fmt(&self, f),
        }
    }
}

impl From<HidError> for Error {
    fn from(value: HidError) -> Self {
        Error::Hid(value.to_string())
    }
}

// This makes it possible to use the `?` operator
// with `Error` in functions that return `Result<T, Error>`.
// We also need this for compatibility with crates like Anyhow.
impl std::error::Error for Error {}

#[cfg(feature = "python")]
impl From<Error> for pyo3::PyErr {
    fn from(err: Error) -> Self {
        match err {
            Error::Hid(msg) => pyo3::PyErr::new::<crate::HidError, _>(msg),
            Error::MaybePermissionDenied(msg) => {
                pyo3::PyErr::new::<crate::MaybePermissionDeniedError, _>(msg)
            }
            Error::NoSuchKeyboard {
                vid,
                pid,
                usage_page,
            } => pyo3::PyErr::new::<crate::DeviceNotFoundError, _>(format!(
                "could not find keyboard: 0x{:04x}/0x{:04x}/0x{:04x} (vendor_id/product_id/usage_page)",
                vid, pid, usage_page
            )),
            Error::UnsupportedProtocol(version) => {
                pyo3::PyErr::new::<crate::UnsupportedProtocolError, _>(format!(
                    "unsupported protocol version: {}",
                    version
                ))
            }
            Error::UnsupportedFeature(feature) => {
                pyo3::PyErr::new::<crate::UnsupportedProtocolError, _>(format!(
                    "unsupported feature: {}",
                    feature
                ))
            }
            Error::SizeMismatch {
                expected,
                actual,
                context,
            } => pyo3::PyErr::new::<crate::SizeMismatchError, _>(format!(
                "{}: expected size = {}, actual size = {}",
                context, expected, actual
            )),
            Error::BadCommandResponse(cmd) => {
                pyo3::PyErr::new::<crate::CommandResponseError, _>(format!(
                    "command failure for {:?}",
                    cmd
                ))
            }
            Error::SendCommand(cmd, msg) => pyo3::PyErr::new::<crate::CommandResponseError, _>(
                format!("failed sending command {:?}: {}", cmd, msg),
            ),
            Error::InvalidArgument(arg) => {
                pyo3::PyErr::new::<crate::InvalidArgumentError, _>(arg)
            }
        }
    }
}
