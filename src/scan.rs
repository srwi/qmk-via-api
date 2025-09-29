use hidapi::HidApi;

#[cfg(feature = "python")]
use pyo3::prelude::*;

const VIA_USAGE_PAGE: u16 = 0xff60;

/// Information about a connected VIA-compatible keyboard.
#[cfg_attr(feature = "python", pyclass(get_all))]
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
pub fn scan_keyboards() -> Vec<KeyboardDeviceInfo> {
    let api = match HidApi::new() {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    api.device_list()
        .filter(|d| d.usage_page() == VIA_USAGE_PAGE)
        .map(|d| {
            KeyboardDeviceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                usage_page: d.usage_page(),
                manufacturer: d.manufacturer_string().map(|s| s.to_string()),
                product: d.product_string().map(|s| s.to_string()),
                serial_number: d.serial_number().map(|s| s.to_string()),
            }
        })
        .collect()
}
