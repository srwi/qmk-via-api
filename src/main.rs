use crate::api::MatrixInfo;

mod api;
mod api_commands;
mod utils;

// set values from config.h
const VENDOR_ID: u16 = 0x594D;
const PRODUCT_ID: u16 = 0x604D;
const USAGE_PAGE: u16 = 0xff60;

fn main() {
    let keyboard_api = api::KeyboardApi::new(PRODUCT_ID, VENDOR_ID, USAGE_PAGE);

    let protocol_version = keyboard_api.get_protocol_version();
    println!("Protocol version: {:?}", protocol_version.unwrap());
    let key = keyboard_api.get_key(0, 0, 0);
    println!("Key: {:?}", key.unwrap());
    let key = keyboard_api.get_layer_count();
    println!("Layer count: {:?}", key.unwrap());
    let key = keyboard_api.read_raw_matrix(MatrixInfo { rows: 5, cols: 14}, 0);
    println!("Matrix: {:?}", key.unwrap());

    std::process::exit(0);
}
