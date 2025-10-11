use std::fmt::Debug;

use hidapi::HidError;

use crate::api_commands::ViaCommandId;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Hid(String),
    BadCommandResponse(ViaCommandId),
    SendCommand(ViaCommandId, String),
    NoSuchKeyboard {
        vid: u16,
        pid: u16,
        usage_page: u16,
    },
    UnsupportedProtocol(u16),
    SizeMismatch {
        expected: usize,
        actual: usize,
        context: &'static str,
    },
}

impl Error {
    pub fn size_mismatch(context: &'static str, expected: usize, actual: usize) -> Self {
        Error::SizeMismatch {
            expected,
            actual,
            context,
        }
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
            _ => Debug::fmt(&self, f),
        }
    }
}

impl From<HidError> for Error {
    fn from(value: HidError) -> Self {
        Error::Hid(value.to_string())
    }
}

#[cfg(feature = "python")]
impl From<Error> for pyo3::PyErr {
    fn from(err: Error) -> Self {
        pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}
