use crate::api_commands::ApiCommand;

use crate::utils::{
    shift_buffer_from_16_bit, shift_buffer_to_16_bit, shift_from_16_bit, shift_to_16_bit,
};

use hidapi::HidApi;
use pyo3::prelude::*;
use std::vec;

type Layer = u8;
type Row = u8;
type Column = u8;

#[pyclass]
#[derive(Clone, Copy)]
pub struct MatrixInfo {
    pub rows: u8,
    pub cols: u8,
}

#[pyclass]
#[derive(Clone, Copy)]
pub enum KeyboardValue {
    Uptime = 0x01,
    LayoutOptions = 0x02,
    SwitchMatrixState = 0x03,
    FirmwareVersion = 0x04,
    DeviceIndication = 0x05,
}

const COMMAND_START: u8 = 0x00;
const PER_KEY_RGB_CHANNEL_COMMAND: &'static [u8] = &[0, 1];

const BACKLIGHT_BRIGHTNESS: u8 = 0x09;
const BACKLIGHT_EFFECT: u8 = 0x0a;
const BACKLIGHT_COLOR_1: u8 = 0x0c;
const BACKLIGHT_COLOR_2: u8 = 0x0d;
const BACKLIGHT_CUSTOM_COLOR: u8 = 0x17;

const PROTOCOL_ALPHA: u16 = 7;
const PROTOCOL_BETA: u16 = 8;
const PROTOCOL_GAMMA: u16 = 9;

#[pyclass]
pub struct KeyboardApi {
    device: hidapi::HidDevice,
}

#[pymethods]
impl KeyboardApi {
    #[new]
    pub fn new(vid: u16, pid: u16, usage_page: u16) -> Self {
        let api = HidApi::new().unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

        let device = api
            .device_list()
            .find(|device| {
                device.vendor_id() == vid
                    && device.product_id() == pid
                    && device.usage_page() == usage_page
            })
            .unwrap_or_else(|| {
                eprintln!("Could not find keyboard.");
                std::process::exit(1);
            })
            .open_device(&api)
            .unwrap_or_else(|_| {
                eprintln!("Could not open HID device.");
                std::process::exit(1);
            });

        KeyboardApi { device }
    }

    fn hid_command(&self, command: ApiCommand, bytes: Vec<u8>) -> Option<Vec<u8>> {
        let mut command_bytes: Vec<u8> = vec![COMMAND_START, command as u8];
        command_bytes.extend(bytes);

        let mut padded_array = vec![0; 33];
        for (idx, &val) in command_bytes.iter().enumerate() {
            padded_array[idx] = val;
        }

        let _ = self.device.write(&padded_array);

        let mut buffer = vec![0; 33];
        let _ = self.device.read(&mut buffer);

        let buffer_command_bytes = &buffer[0..command_bytes.len() - 1];

        if command_bytes[1..] != *buffer_command_bytes {
            return None;
        }

        Some(buffer) // TODO: If possible, return a type that can be destructured in a match block
    }

    fn get_protocol_version(&self) -> Option<u16> {
        match self.hid_command(ApiCommand::GetProtocolVersion, vec![]) {
            Some(val) => Some(shift_to_16_bit(val[1], val[2])),
            None => None,
        }
    }

    fn get_key(&self, layer: Layer, row: Row, col: Column) -> Option<u16> {
        match self.hid_command(ApiCommand::DynamicKeymapGetKeycode, vec![layer, row, col]) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    fn get_layer_count(&self) -> Option<u8> {
        match self.get_protocol_version() {
            Some(version) => {
                if version >= PROTOCOL_BETA {
                    match self.hid_command(ApiCommand::DynamicKeymapGetLayerCount, vec![]) {
                        Some(val) => Some(val[1]),
                        None => None,
                    }
                } else {
                    Some(4)
                }
            }
            None => None,
        }
    }

    fn read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        match self.get_protocol_version() {
            Some(version) => {
                if version >= PROTOCOL_BETA {
                    self.fast_read_raw_matrix(matrix_info, layer)
                } else if version == PROTOCOL_ALPHA {
                    self.slow_read_raw_matrix(matrix_info, layer)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn get_keymap_buffer(&self, offset: u16, size: u8) -> Option<Vec<u8>> {
        if size > 28 {
            return None;
        }
        let offset_bytes = shift_from_16_bit(offset);
        match self.hid_command(
            ApiCommand::DynamicKeymapGetBuffer,
            vec![offset_bytes.0, offset_bytes.1, size],
        ) {
            Some(val) => Some(val[4..(size as usize + 4)].to_vec()),
            None => None,
        }
    }

    fn fast_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        const MAX_KEYCODES_PARTIAL: usize = 14;
        let length = matrix_info.rows as usize * matrix_info.cols as usize;
        let buffer_list = vec![0; length.div_ceil(MAX_KEYCODES_PARTIAL) as usize];
        let mut remaining = length;
        let mut result = Vec::new();
        for _ in 0..buffer_list.len() {
            if remaining < MAX_KEYCODES_PARTIAL {
                match self.get_keymap_buffer(
                    layer as u16 * length as u16 * 2 + 2 * (length - remaining) as u16,
                    (remaining * 2) as u8,
                ) {
                    Some(val) => result.extend(val),
                    None => return None,
                }
                remaining = 0;
            } else {
                match self.get_keymap_buffer(
                    layer as u16 * length as u16 * 2 + 2 * (length - remaining) as u16,
                    (MAX_KEYCODES_PARTIAL * 2) as u8,
                ) {
                    Some(val) => result.extend(val),
                    None => return None,
                }
                remaining -= MAX_KEYCODES_PARTIAL;
            }
        }
        Some(shift_buffer_to_16_bit(&result))
    }

    fn slow_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        let length = matrix_info.rows as usize * matrix_info.cols as usize;
        let mut res = Vec::new();
        for i in 0..length {
            let row = (i as u16 / matrix_info.cols as u16) as u8;
            let col = (i as u16 % matrix_info.cols as u16) as u8;
            match self.get_key(layer, row, col) {
                Some(val) => res.push(val),
                None => return None,
            }
        }
        Some(res)
    }

    fn write_raw_matrix(&self, matrix_info: MatrixInfo, keymap: Vec<Vec<u16>>) -> Option<()> {
        match self.get_protocol_version() {
            Some(version) => {
                if version >= PROTOCOL_BETA {
                    self.fast_write_raw_matrix(keymap)
                } else if version == PROTOCOL_ALPHA {
                    self.slow_write_raw_matrix(matrix_info, keymap)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn slow_write_raw_matrix(&self, matrix_info: MatrixInfo, keymap: Vec<Vec<u16>>) -> Option<()> {
        for (layer_idx, layer) in keymap.iter().enumerate() {
            for (key_idx, keycode) in layer.iter().enumerate() {
                let row = (key_idx as u16 / matrix_info.cols as u16) as u8;
                let col = (key_idx as u16 % matrix_info.cols as u16) as u8;
                match self.set_key(layer_idx as u8, row, col, *keycode) {
                    Some(_) => (),
                    None => return None,
                }
            }
        }
        Some(())
    }

    fn fast_write_raw_matrix(&self, keymap: Vec<Vec<u16>>) -> Option<()> {
        let data: Vec<u16> = keymap
            .iter()
            .flat_map(|layer| layer.iter().cloned())
            .collect();
        let shifted_data = shift_buffer_from_16_bit(&data);
        let buffer_size = 28;
        for offset in (0..shifted_data.len()).step_by(buffer_size as usize) {
            let offset_bytes = shift_from_16_bit(offset as u16);
            let buffer = shifted_data[offset..offset + buffer_size].to_vec();
            let mut bytes = vec![offset_bytes.0, offset_bytes.1, buffer_size as u8];
            bytes.extend(buffer);
            match self.hid_command(ApiCommand::DynamicKeymapSetBuffer, bytes) {
                Some(_) => (),
                None => return None,
            }
        }
        Some(())
    }

    fn get_keyboard_value(
        &self,
        command: KeyboardValue,
        parameters: Vec<u8>,
        result_length: usize,
    ) -> Option<Vec<u8>> {
        let parameters_length = parameters.len();
        let mut bytes = vec![command as u8];
        bytes.extend(parameters);
        match self.hid_command(ApiCommand::GetKeyboardValue, bytes) {
            Some(val) => {
                Some(val[1 + parameters_length..1 + parameters_length + result_length].to_vec())
            }
            None => None,
        }
    }

    fn set_keyboard_value(&self, command: KeyboardValue, rest: Vec<u8>) -> Option<()> {
        let mut bytes = vec![command as u8];
        bytes.extend(rest);
        match self.hid_command(ApiCommand::SetKeyboardValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_encoder_value(&self, layer: Layer, id: u8, is_clockwise: bool) -> Option<u16> {
        match self.hid_command(
            ApiCommand::DynamicKeymapGetEncoder,
            vec![layer, id, is_clockwise as u8],
        ) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    fn set_encoder_value(
        &self,
        layer: Layer,
        id: u8,
        is_clockwise: bool,
        keycode: u16,
    ) -> Option<()> {
        let keycode_bytes = shift_from_16_bit(keycode);
        let bytes = vec![
            layer,
            id,
            is_clockwise as u8,
            keycode_bytes.0,
            keycode_bytes.1,
        ];
        match self.hid_command(ApiCommand::DynamicKeymapSetEncoder, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_custom_menu_value(&self, command_bytes: Vec<u8>) -> Option<Vec<u8>> {
        let command_length = command_bytes.len();
        match self.hid_command(ApiCommand::CustomMenuGetValue, command_bytes) {
            Some(val) => Some(val[0..command_length].to_vec()),
            None => None,
        }
    }

    fn set_custom_menu_value(&self, args: Vec<u8>) -> Option<()> {
        match self.hid_command(ApiCommand::CustomMenuSetValue, args) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_per_key_rgb_matrix(&self, led_index_mapping: Vec<u8>) -> Option<Vec<Vec<u8>>> {
        let mut res = Vec::new();
        for led_index in led_index_mapping {
            let mut bytes = PER_KEY_RGB_CHANNEL_COMMAND.to_vec();
            bytes.extend(vec![led_index, 1]);
            match self.hid_command(ApiCommand::CustomMenuGetValue, bytes) {
                Some(val) => res.push(val[5..7].to_vec()),
                None => return None,
            }
        }
        Some(res)
    }

    fn set_per_key_rgb_matrix(&self, index: u8, hue: u8, sat: u8) -> Option<()> {
        let mut bytes = PER_KEY_RGB_CHANNEL_COMMAND.to_vec();
        bytes.extend(vec![index, 1, hue, sat]);
        // let bytes = [PER_KEY_RGB_CHANNEL_COMMAND, &vec![index, 1, hue, sat].as_slice()].concat();
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_backlight_value(&self, command: ApiCommand, result_length: usize) -> Option<Vec<u8>> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![command as u8]) {
            Some(val) => Some(val[2..result_length + 2].to_vec()),
            None => None,
        }
    }

    fn set_backlight_value(&self, command: ApiCommand, rest: Vec<u8>) -> Option<()> {
        let mut bytes: Vec<u8> = vec![command as u8];
        bytes.extend(rest);
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_rgb_mode(&self) -> Option<u8> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![BACKLIGHT_EFFECT]) {
            Some(val) => Some(val[2]),
            None => None,
        }
    }

    fn get_brightness(&self) -> Option<u8> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![BACKLIGHT_BRIGHTNESS]) {
            Some(val) => Some(val[2]),
            None => None,
        }
    }

    fn get_color(&self, color_number: u8) -> Option<(u8, u8)> {
        let bytes = vec![if color_number == 1 {
            BACKLIGHT_COLOR_1
        } else {
            BACKLIGHT_COLOR_2
        }];
        match self.hid_command(ApiCommand::CustomMenuGetValue, bytes) {
            Some(val) => Some((val[2], val[3])),
            None => None,
        }
    }

    fn set_color(&self, color_number: u8, hue: u8, sat: u8) -> Option<()> {
        let bytes = vec![
            if color_number == 1 {
                BACKLIGHT_COLOR_1
            } else {
                BACKLIGHT_COLOR_2
            },
            hue,
            sat,
        ];
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn get_custom_color(&self, color_number: u8) -> Option<(u8, u8)> {
        let bytes = vec![BACKLIGHT_CUSTOM_COLOR, color_number];
        match self.hid_command(ApiCommand::CustomMenuGetValue, bytes) {
            Some(val) => Some((val[3], val[4])),
            None => None,
        }
    }

    fn set_custom_color(&self, color_number: u8, hue: u8, sat: u8) -> Option<()> {
        let bytes = vec![BACKLIGHT_CUSTOM_COLOR, color_number, hue, sat];
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn set_rgb_mode(&self, effect: u8) -> Option<()> {
        let bytes = vec![BACKLIGHT_EFFECT, effect];
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn commit_custom_menu(&self, channel: u8) -> Option<()> {
        let bytes = vec![channel];
        match self.hid_command(ApiCommand::CustomMenuSave, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn save_lighting(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::CustomMenuSave, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn reset_eeprom(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::EepromReset, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn jump_to_bootloader(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::BootloaderJump, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn set_key(&self, layer: Layer, row: Row, column: Column, val: u16) -> Option<u16> {
        let val_bytes = shift_from_16_bit(val);
        let bytes = vec![layer, row, column, val_bytes.0, val_bytes.1];
        match self.hid_command(ApiCommand::DynamicKeymapSetKeycode, bytes) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    fn get_macro_count(&self) -> Option<u8> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroGetCount, bytes) {
            Some(val) => Some(val[1]),
            None => None,
        }
    }

    fn get_macro_buffer_size(&self) -> Option<u16> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroGetBufferSize, bytes) {
            Some(val) => Some(shift_to_16_bit(val[1], val[2])),
            None => None,
        }
    }

    fn get_macro_bytes(&self) -> Option<Vec<u8>> {
        let macro_buffer_size = self.get_macro_buffer_size()?;
        let size: u8 = 28; // Can only get 28 bytes at a time
        let mut all_bytes = Vec::new();
        for offset in (0..macro_buffer_size).step_by(size as usize) {
            let offset_bytes = shift_from_16_bit(offset);
            let bytes = vec![offset_bytes.0, offset_bytes.1, size];
            match self.hid_command(ApiCommand::DynamicKeymapMacroGetBuffer, bytes) {
                Some(val) => all_bytes.extend(val[4..].to_vec()),
                None => return None,
            }
        }
        Some(all_bytes)
    }

    fn set_macro_bytes(&self, data: Vec<u8>) -> Option<()> {
        let macro_buffer_size = self.get_macro_buffer_size()?;
        let size = data.len();
        if size > macro_buffer_size as usize {
            return None;
        }

        let last_offset = macro_buffer_size - 1;
        let last_offset_bytes = shift_from_16_bit(last_offset);

        self.reset_macros()?;

        // Set last byte in buffer to non-zero (0xFF) to indicate write-in-progress
        self.hid_command(
            ApiCommand::DynamicKeymapMacroSetBuffer,
            vec![last_offset_bytes.0, last_offset_bytes.1, 1, 0xff],
        )?;

        let buffer_size: u8 = 28; // Can only write 28 bytes at a time
        for offset in (0..data.len()).step_by(buffer_size as usize) {
            let offset_bytes = shift_from_16_bit(offset as u16);
            let mut bytes = vec![offset_bytes.0, offset_bytes.1, buffer_size];
            bytes.extend(data[offset..offset + buffer_size as usize].to_vec());
            self.hid_command(ApiCommand::DynamicKeymapMacroSetBuffer, bytes)?;
        }

        // Set last byte in buffer to zero to indicate write finished
        self.hid_command(
            ApiCommand::DynamicKeymapMacroSetBuffer,
            vec![last_offset_bytes.0, last_offset_bytes.1, 1, 0x00],
        )?;

        Some(())
    }

    fn reset_macros(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroReset, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }
}
