use hidapi::{DeviceInfo, HidApi};

mod api;
mod api_commands;
mod utils;

// set values from config.h
const VENDOR_ID: u16 = 0x594D;
const PRODUCT_ID: u16 = 0x604D;
const USAGE_PAGE: u16 = 0xff60;

fn is_my_device(device: &DeviceInfo) -> bool {
    device.vendor_id() == VENDOR_ID
        && device.product_id() == PRODUCT_ID
        && device.usage_page() == USAGE_PAGE
}

fn main() {
    let api = HidApi::new().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let device = api
        .device_list()
        .find(|device| is_my_device(device))
        .unwrap_or_else(|| {
            eprintln!("Could not find keyboard");
            std::process::exit(1);
        })
        .open_device(&api)
        .unwrap_or_else(|_| {
            eprintln!("Could not open HID device");
            std::process::exit(1);
        });

    // let layer = 3;
    // let _ = device.write(&[0, 9, 0, 1, layer]);

    let keyboard_api = api::KeyboardApi::new(device);

    // let response = keyboard_api.hid_command(
    //     api_commands::ApiCommand::CUSTOM_MENU_SAVE,
    //     vec![0, 1, layer],
    // );
    // print!("Response: {:?}", response);

    let protocol_version = keyboard_api.get_protocol_version();
    println!("Protocol version: {:?}", protocol_version.unwrap());

    std::process::exit(0);
}
