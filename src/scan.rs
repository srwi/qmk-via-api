use hidapi::HidApi;
use pyo3::prelude::*;

const VIA_USAGE_PAGE: u16 = 0xff60;

/// Information about a connected VIA-compatible keyboard.
#[pyclass]
#[derive(Clone, Debug)]
pub struct KeyboardDeviceInfo {
    /// USB vendor ID
    #[pyo3(get)]
    pub vendor_id: u16,
    /// USB product ID
    #[pyo3(get)]
    pub product_id: u16,
    /// HID usage page (expected to be 0xFF60 for VIA)
    #[pyo3(get)]
    pub usage_page: u16,
    /// Optional manufacturer string
    #[pyo3(get)]
    pub manufacturer: Option<String>,
    /// Optional product string
    #[pyo3(get)]
    pub product: Option<String>,
    /// Optional serial number string
    #[pyo3(get)]
    pub serial_number: Option<String>,
}

/// Scan for connected VIA keyboards.
#[pyfunction]
pub fn scan_keyboards() -> Vec<KeyboardDeviceInfo> {
    let api = match HidApi::new() {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    api.device_list()
        .filter(|d| d.usage_page() == VIA_USAGE_PAGE)
        .filter_map(|d| {
            Some(KeyboardDeviceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                usage_page: d.usage_page(),
                manufacturer: d.manufacturer_string().map(|s| s.to_string()),
                product: d.product_string().map(|s| s.to_string()),
                serial_number: d.serial_number().map(|s| s.to_string()),
            })
        })
        .collect()
}
