use crate::{Error, Result};
use hidapi::HidApi;

#[cfg(feature = "python")]
use pyo3::prelude::*;

const VIA_USAGE_PAGE: u16 = 0xff60;

/// Information about a connected VIA-compatible keyboard.
#[cfg_attr(feature = "python", pyclass(get_all, from_py_object))]
#[derive(Clone, Debug)]
pub struct KeyboardDeviceInfo {
    /// USB vendor ID
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
    /// HID usage page (expected to be 0xFF60 for VIA)
    pub usage_page: u16,
    /// Optional manufacturer string
    pub manufacturer: Option<String>,
    /// Optional product string
    pub product: Option<String>,
    /// Optional serial number string
    pub serial_number: Option<String>,
}

/// Scan for connected VIA keyboards.
#[cfg_attr(feature = "python", pyfunction)]
pub fn scan_keyboards() -> Result<Vec<KeyboardDeviceInfo>> {
    let api = HidApi::new()?;

    Ok(api
        .device_list()
        .filter(|d| d.usage_page() == VIA_USAGE_PAGE)
        .map(|d| KeyboardDeviceInfo {
            vendor_id: d.vendor_id(),
            product_id: d.product_id(),
            usage_page: d.usage_page(),
            manufacturer: d.manufacturer_string().map(|s| s.to_string()),
            product: d.product_string().map(|s| s.to_string()),
            serial_number: d.serial_number().map(|s| s.to_string()),
        })
        .collect())
}

fn try_open_device(api: &HidApi, info: &KeyboardDeviceInfo) -> Result<()> {
    let device = api
        .device_list()
        .find(|d| {
            d.usage_page() == VIA_USAGE_PAGE
                && d.vendor_id() == info.vendor_id
                && d.product_id() == info.product_id
        })
        .ok_or(Error::NoSuchKeyboard {
            vid: info.vendor_id,
            pid: info.product_id,
            usage_page: info.usage_page,
        })?;

    device.open_device(api)?;
    Ok(())
}

/// Check for HID permissions.
/// On linux, this checks if the HID device list is empty and optionally tries to open a specific device to verify permissions.
/// On windows and macOS, it simply checks if the HID API can be initialized and optionally tries to open a specific device.
/// On unsupported platforms, it returns an error indicating that the feature is not implemented.
#[cfg_attr(feature = "python", pyfunction)]
pub fn check_hid_permissions(filter: Option<KeyboardDeviceInfo>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        match HidApi::new() {
            Ok(api) => {
                if api.device_list().count() == 0 {
                    return Err(Error::maybe_permission_denied());
                }
                if let Some(f) = filter {
                    try_open_device(&api, &f)?;
                }
                Ok(())
            }
            Err(e) => Err(Error::Hid(format!("Failed to initialize HID API: {}", e))),
        }
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        match HidApi::new() {
            Ok(api) => {
                if let Some(f) = filter {
                    try_open_device(&api, &f)?;
                }
                Ok(())
            }
            Err(e) => Err(Error::Hid(format!("Failed to initialize HID API: {}", e))),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(Error::UnsupportedFeature(
            "HID permission checking is not implemented for this platform",
        ))
    }
}
