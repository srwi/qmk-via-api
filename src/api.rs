use crate::api_commands::{
    ViaChannelId, ViaCommandId, ViaQmkAudioValue, ViaQmkBacklightValue, ViaQmkLedMatrixValue,
    ViaQmkRgbMatrixValue, ViaQmkRgblightValue,
};
use crate::utils;
use core::result::Result;
use hidapi::HidApi;
use std::str::FromStr;
use std::vec;

#[cfg(feature = "python")]
use pyo3::prelude::*;

const COMMAND_START: u8 = 0x00;

pub const RAW_EPSIZE: usize = 32;
pub const DATA_BUFFER_SIZE: usize = 28;

pub const PROTOCOL_ALPHA: u16 = 7;
pub const PROTOCOL_BETA: u16 = 8;
pub const PROTOCOL_GAMMA: u16 = 9;

pub type Layer = u8;
pub type Row = u8;
pub type Column = u8;

#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Copy, Debug)]
pub struct MatrixInfo {
    pub rows: u8,
    pub cols: u8,
}

#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Copy, Debug)]
pub enum KeyboardValue {
    Uptime = 0x01,
    LayoutOptions = 0x02,
    SwitchMatrixState = 0x03,
    FirmwareVersion = 0x04,
    DeviceIndication = 0x05,
}

impl FromStr for KeyboardValue {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Uptime" => Ok(KeyboardValue::Uptime),
            "LayoutOptions" => Ok(KeyboardValue::LayoutOptions),
            "SwitchMatrixState" => Ok(KeyboardValue::SwitchMatrixState),
            "FirmwareVersion" => Ok(KeyboardValue::FirmwareVersion),
            "DeviceIndication" => Ok(KeyboardValue::DeviceIndication),
            _ => Err("Invalid KeyboardValue"),
        }
    }
}

#[cfg_attr(feature = "python", pyclass(unsendable))]
pub struct KeyboardApi {
    device: hidapi::HidDevice,
}

#[cfg(feature = "python")]
#[pymethods]
impl KeyboardApi {
    #[new]
    pub fn py_new(vid: u16, pid: u16, usage_page: u16) -> Result<Self, Error> {
        KeyboardApi::new(vid, pid, usage_page)
    }
}

#[cfg(feature = "python")]
impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.0)
    }
}

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error(err.to_string())
    }
}

impl KeyboardApi {
    pub fn new(vid: u16, pid: u16, usage_page: u16) -> Result<KeyboardApi, Error> {
        let api = HidApi::new().map_err(|e| format!("Error: {e}"))?;

        let device = api
            .device_list()
            .find(|device| {
                device.vendor_id() == vid
                    && device.product_id() == pid
                    && device.usage_page() == usage_page
            })
            .ok_or("Could not find keyboard.")?
            .open_device(&api)
            .map_err(|_| "Could not open HID device.")?;

        Ok(KeyboardApi { device })
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl KeyboardApi {
    /// Sends a raw HID command prefixed with the command byte and returns the response if successful.
    pub fn hid_command(&self, command: ViaCommandId, bytes: Vec<u8>) -> Option<Vec<u8>> {
        let mut command_bytes: Vec<u8> = vec![command as u8];
        command_bytes.extend(bytes);

        self.hid_send(command_bytes.clone())?;

        let buffer = self.hid_read()?;
        if buffer.starts_with(&command_bytes) {
            Some(buffer)
        } else {
            None
        }
    }

    /// Reads from the HID device. Returns None if the read fails.
    pub fn hid_read(&self) -> Option<Vec<u8>> {
        let mut buffer = vec![0; RAW_EPSIZE];
        match self.device.read(&mut buffer) {
            Ok(_) => Some(buffer),
            Err(_) => None,
        }
    }

    /// Sends a raw HID command prefixed with the command byte. Returns None if the send fails.
    pub fn hid_send(&self, bytes: Vec<u8>) -> Option<()> {
        if bytes.len() > RAW_EPSIZE {
            return None;
        }

        let mut command_bytes: Vec<u8> = vec![COMMAND_START];
        command_bytes.extend(bytes);

        let mut padded_array = vec![0; RAW_EPSIZE + 1];
        for (idx, &val) in command_bytes.iter().enumerate() {
            padded_array[idx] = val;
        }

        if self.device.write(&padded_array).ok()? == RAW_EPSIZE + 1 {
            return Some(());
        }

        None
    }

    /// Returns the protocol version of the keyboard.
    pub fn get_protocol_version(&self) -> Option<u16> {
        self.hid_command(ViaCommandId::GetProtocolVersion, vec![])
            .map(|val| utils::shift_to_16_bit(val[1], val[2]))
    }

    /// Returns the number of layers on the keyboard.
    pub fn get_layer_count(&self) -> Option<u8> {
        match self.get_protocol_version() {
            Some(version) if version >= PROTOCOL_BETA => self
                .hid_command(ViaCommandId::DynamicKeymapGetLayerCount, vec![])
                .map(|val| val[1]),
            Some(_) => Some(4),
            _ => None,
        }
    }

    /// Returns the keycode at the given layer, row, and column.
    pub fn get_key(&self, layer: Layer, row: Row, col: Column) -> Option<u16> {
        self.hid_command(ViaCommandId::DynamicKeymapGetKeycode, vec![layer, row, col])
            .map(|val| utils::shift_to_16_bit(val[4], val[5]))
    }

    /// Sets the keycode at the given layer, row, and column.
    pub fn set_key(&self, layer: Layer, row: Row, column: Column, val: u16) -> Option<u16> {
        let val_bytes = utils::shift_from_16_bit(val);
        let bytes = vec![layer, row, column, val_bytes.0, val_bytes.1];
        self.hid_command(ViaCommandId::DynamicKeymapSetKeycode, bytes)
            .map(|val| utils::shift_to_16_bit(val[4], val[5]))
    }

    /// Returns the keycodes for the given matrix info (number of rows and columns) and layer.
    pub fn read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        match self.get_protocol_version() {
            Some(version) if version >= PROTOCOL_BETA => {
                self.fast_read_raw_matrix(matrix_info, layer)
            }
            Some(version) if version == PROTOCOL_ALPHA => {
                self.slow_read_raw_matrix(matrix_info, layer)
            }
            _ => None,
        }
    }

    fn get_keymap_buffer(&self, offset: u16, size: u8) -> Option<Vec<u8>> {
        if size > DATA_BUFFER_SIZE as u8 {
            return None;
        }
        let offset_bytes = utils::shift_from_16_bit(offset);
        self.hid_command(
            ViaCommandId::DynamicKeymapGetBuffer,
            vec![offset_bytes.0, offset_bytes.1, size],
        )
        .map(|val| val[4..(size as usize + 4)].to_vec())
    }

    fn fast_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        const MAX_KEYCODES_PARTIAL: usize = 14;
        let length = matrix_info.rows as usize * matrix_info.cols as usize;
        let buffer_len = length.div_ceil(MAX_KEYCODES_PARTIAL);
        let mut remaining = length;
        let mut result = Vec::new();
        for _ in 0..buffer_len {
            if remaining < MAX_KEYCODES_PARTIAL {
                if let Some(val) = self.get_keymap_buffer(
                    layer as u16 * length as u16 * 2 + 2 * (length - remaining) as u16,
                    (remaining * 2) as u8,
                ) {
                    result.extend(val)
                }
                remaining = 0;
            } else {
                if let Some(val) = self.get_keymap_buffer(
                    layer as u16 * length as u16 * 2 + 2 * (length - remaining) as u16,
                    (MAX_KEYCODES_PARTIAL * 2) as u8,
                ) {
                    result.extend(val)
                }
                remaining -= MAX_KEYCODES_PARTIAL;
            }
        }
        Some(utils::shift_buffer_to_16_bit(&result))
    }

    fn slow_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
        let length = matrix_info.rows as usize * matrix_info.cols as usize;
        let mut res = Vec::new();
        for i in 0..length {
            let row = (i as u16 / matrix_info.cols as u16) as u8;
            let col = (i as u16 % matrix_info.cols as u16) as u8;
            if let Some(val) = self.get_key(layer, row, col) {
                res.push(val)
            }
        }
        Some(res)
    }

    /// Writes a keymap to the keyboard for the given matrix info (number of rows and columns).
    pub fn write_raw_matrix(&self, matrix_info: MatrixInfo, keymap: Vec<Vec<u16>>) -> Option<()> {
        match self.get_protocol_version() {
            Some(version) if version >= PROTOCOL_BETA => self.fast_write_raw_matrix(keymap),
            Some(version) if version == PROTOCOL_ALPHA => {
                self.slow_write_raw_matrix(matrix_info, keymap)
            }
            _ => None,
        }
    }

    fn slow_write_raw_matrix(&self, matrix_info: MatrixInfo, keymap: Vec<Vec<u16>>) -> Option<()> {
        for (layer_idx, layer) in keymap.iter().enumerate() {
            for (key_idx, keycode) in layer.iter().enumerate() {
                let row = (key_idx as u16 / matrix_info.cols as u16) as u8;
                let col = (key_idx as u16 % matrix_info.cols as u16) as u8;
                self.set_key(layer_idx as u8, row, col, *keycode);
            }
        }
        Some(())
    }

    fn fast_write_raw_matrix(&self, keymap: Vec<Vec<u16>>) -> Option<()> {
        let data: Vec<u16> = keymap
            .iter()
            .flat_map(|layer| layer.iter().cloned())
            .collect();
        let shifted_data = utils::shift_buffer_from_16_bit(&data);
        for offset in (0..shifted_data.len()).step_by(DATA_BUFFER_SIZE) {
            let offset_bytes = utils::shift_from_16_bit(offset as u16);
            let end = std::cmp::min(offset + DATA_BUFFER_SIZE, shifted_data.len());
            let buffer = shifted_data[offset..end].to_vec();
            let mut bytes = vec![offset_bytes.0, offset_bytes.1, buffer.len() as u8];
            bytes.extend(buffer);
            self.hid_command(ViaCommandId::DynamicKeymapSetBuffer, bytes);
        }
        Some(())
    }

    /// Returns a keyboard value. This can be used to retrieve keyboard information like uptime, layout options, switch matrix state and firmware version.
    pub fn get_keyboard_value(
        &self,
        command: KeyboardValue,
        parameters: Vec<u8>,
        result_length: usize,
    ) -> Option<Vec<u8>> {
        let parameters_length = parameters.len();
        let mut bytes = vec![command as u8];
        bytes.extend(parameters);
        self.hid_command(ViaCommandId::GetKeyboardValue, bytes)
            .map(|val| val[1 + parameters_length..1 + parameters_length + result_length].to_vec())
    }

    /// Sets a keyboard value. This can be used to set keyboard values like layout options or device indication.
    pub fn set_keyboard_value(&self, command: KeyboardValue, parameters: Vec<u8>) -> Option<()> {
        let mut bytes = vec![command as u8];
        bytes.extend(parameters);
        self.hid_command(ViaCommandId::SetKeyboardValue, bytes)
            .map(|_| ())
    }

    /// Gets the encoder value for the given layer, id, and direction.
    pub fn get_encoder_value(&self, layer: Layer, id: u8, is_clockwise: bool) -> Option<u16> {
        self.hid_command(
            ViaCommandId::DynamicKeymapGetEncoder,
            vec![layer, id, is_clockwise as u8],
        )
        .map(|val| utils::shift_to_16_bit(val[4], val[5]))
    }

    /// Sets the encoder value for the given layer, id, direction, and keycode.
    pub fn set_encoder_value(
        &self,
        layer: Layer,
        id: u8,
        is_clockwise: bool,
        keycode: u16,
    ) -> Option<()> {
        let keycode_bytes = utils::shift_from_16_bit(keycode);
        let bytes = vec![
            layer,
            id,
            is_clockwise as u8,
            keycode_bytes.0,
            keycode_bytes.1,
        ];
        self.hid_command(ViaCommandId::DynamicKeymapSetEncoder, bytes)
            .map(|_| ())
    }

    /// Get a custom menu value. This is a generic function that can be used to get any value specific to arbitrary keyboard functionalities.
    pub fn get_custom_menu_value(&self, command_bytes: Vec<u8>) -> Option<Vec<u8>> {
        let command_length = command_bytes.len();
        self.hid_command(ViaCommandId::CustomMenuGetValue, command_bytes)
            .map(|val| val[0..command_length].to_vec())
    }

    /// Set a custom menu value. This is a generic function that can be used to set any value specific to arbitrary keyboard functionalities.
    pub fn set_custom_menu_value(&self, args: Vec<u8>) -> Option<()> {
        self.hid_command(ViaCommandId::CustomMenuSetValue, args)
            .map(|_| ())
    }

    /// Saves the custom menu values for the given channel id.
    pub fn save_custom_menu(&self, channel: u8) -> Option<()> {
        let bytes = vec![channel];
        self.hid_command(ViaCommandId::CustomMenuSave, bytes)
            .map(|_| ())
    }

    /// Gets the backlight brightness.
    pub fn get_backlight_brightness(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkBacklightChannel as u8,
                ViaQmkBacklightValue::IdQmkBacklightBrightness as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the backlight brightness.
    pub fn set_backlight_brightness(&self, brightness: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkBacklightChannel as u8,
                ViaQmkBacklightValue::IdQmkBacklightBrightness as u8,
                brightness,
            ],
        )
        .map(|_| ())
    }

    /// Gets the backlight effect.
    pub fn get_backlight_effect(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkBacklightChannel as u8,
                ViaQmkBacklightValue::IdQmkBacklightEffect as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the backlight effect.
    pub fn set_backlight_effect(&self, effect: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkBacklightChannel as u8,
                ViaQmkBacklightValue::IdQmkBacklightEffect as u8,
                effect,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB light brightness.
    pub fn get_rgblight_brightness(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightBrightness as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB light brightness.
    pub fn set_rgblight_brightness(&self, brightness: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightBrightness as u8,
                brightness,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB light effect.
    pub fn get_rgblight_effect(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightEffect as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB light effect.
    pub fn set_rgblight_effect(&self, effect: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightEffect as u8,
                effect,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB light effect speed.
    pub fn get_rgblight_effect_speed(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightEffectSpeed as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB light effect speed.
    pub fn set_rgblight_effect_speed(&self, speed: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightEffectSpeed as u8,
                speed,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB light color.
    pub fn get_rgblight_color(&self) -> Option<(u8, u8)> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightColor as u8,
            ],
        )
        .map(|val| (val[3], val[4]))
    }

    /// Sets the RGB light color.
    pub fn set_rgblight_color(&self, hue: u8, sat: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgblightChannel as u8,
                ViaQmkRgblightValue::IdQmkRgblightColor as u8,
                hue,
                sat,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB matrix brightness.
    pub fn get_rgb_matrix_brightness(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixBrightness as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB matrix brightness.
    pub fn set_rgb_matrix_brightness(&self, brightness: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixBrightness as u8,
                brightness,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB matrix effect.
    pub fn get_rgb_matrix_effect(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixEffect as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB matrix effect.
    pub fn set_rgb_matrix_effect(&self, effect: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixEffect as u8,
                effect,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB matrix effect speed.
    pub fn get_rgb_matrix_effect_speed(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixEffectSpeed as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the RGB matrix effect speed.
    pub fn set_rgb_matrix_effect_speed(&self, speed: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixEffectSpeed as u8,
                speed,
            ],
        )
        .map(|_| ())
    }

    /// Gets the RGB matrix color.
    pub fn get_rgb_matrix_color(&self) -> Option<(u8, u8)> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixColor as u8,
            ],
        )
        .map(|val| (val[3], val[4]))
    }

    /// Sets the RGB matrix color.
    pub fn set_rgb_matrix_color(&self, hue: u8, sat: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkRgbMatrixChannel as u8,
                ViaQmkRgbMatrixValue::IdQmkRgbMatrixColor as u8,
                hue,
                sat,
            ],
        )
        .map(|_| ())
    }

    /// Gets the LED matrix brightness.
    pub fn get_led_matrix_brightness(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixBrightness as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the LED matrix brightness.
    pub fn set_led_matrix_brightness(&self, brightness: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixBrightness as u8,
                brightness,
            ],
        )
        .map(|_| ())
    }

    /// Gets the LED matrix effect.
    pub fn get_led_matrix_effect(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixEffect as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the LED matrix effect.
    pub fn set_led_matrix_effect(&self, effect: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixEffect as u8,
                effect,
            ],
        )
        .map(|_| ())
    }

    /// Gets the LED matrix effect speed.
    pub fn get_led_matrix_effect_speed(&self) -> Option<u8> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixEffectSpeed as u8,
            ],
        )
        .map(|val| val[3])
    }

    /// Sets the LED matrix effect speed.
    pub fn set_led_matrix_effect_speed(&self, speed: u8) -> Option<()> {
        self.hid_command(
            ViaCommandId::CustomMenuSetValue,
            vec![
                ViaChannelId::IdQmkLedMatrixChannel as u8,
                ViaQmkLedMatrixValue::IdQmkLedMatrixEffectSpeed as u8,
                speed,
            ],
        )
        .map(|_| ())
    }

    /// Saves the lighting settings.
    pub fn save_lighting(&self) -> Option<()> {
        self.hid_command(ViaCommandId::CustomMenuSave, vec![])
            .map(|_| ())
    }

    /// Gets the audio enabled state.
    pub fn get_audio_enabled(&self) -> Option<bool> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkAudioChannel as u8,
                ViaQmkAudioValue::IdQmkAudioEnable as u8,
            ],
        )
        .map(|val| val[3] == 1)
    }

    /// Sets the audio enabled state.
    pub fn set_audio_enabled(&self, enabled: bool) -> Option<()> {
        let bytes = vec![
            ViaChannelId::IdQmkAudioChannel as u8,
            ViaQmkAudioValue::IdQmkAudioEnable as u8,
            enabled as u8,
        ];
        self.hid_command(ViaCommandId::CustomMenuSetValue, bytes)
            .map(|_| ())
    }

    /// Gets the audio clicky enabled state.
    pub fn get_audio_clicky_enabled(&self) -> Option<bool> {
        self.hid_command(
            ViaCommandId::CustomMenuGetValue,
            vec![
                ViaChannelId::IdQmkAudioChannel as u8,
                ViaQmkAudioValue::IdQmkAudioClickyEnable as u8,
            ],
        )
        .map(|val| val[3] == 1)
    }

    /// Sets the audio clicky enabled state.
    pub fn set_audio_clicky_enabled(&self, enabled: bool) -> Option<()> {
        let bytes = vec![
            ViaChannelId::IdQmkAudioChannel as u8,
            ViaQmkAudioValue::IdQmkAudioClickyEnable as u8,
            enabled as u8,
        ];
        self.hid_command(ViaCommandId::CustomMenuSetValue, bytes)
            .map(|_| ())
    }

    /// Gets the macro count.
    pub fn get_macro_count(&self) -> Option<u8> {
        let bytes = vec![];
        self.hid_command(ViaCommandId::DynamicKeymapMacroGetCount, bytes)
            .map(|val| val[1])
    }

    fn get_macro_buffer_size(&self) -> Option<u16> {
        let bytes = vec![];
        self.hid_command(ViaCommandId::DynamicKeymapMacroGetBufferSize, bytes)
            .map(|val| utils::shift_to_16_bit(val[1], val[2]))
    }

    /// Gets the macro bytes. All macros are separated by 0x00.
    pub fn get_macro_bytes(&self) -> Option<Vec<u8>> {
        let macro_buffer_size = self.get_macro_buffer_size()? as usize;
        let mut all_bytes = Vec::new();
        for offset in (0..macro_buffer_size).step_by(DATA_BUFFER_SIZE) {
            let offset_bytes = utils::shift_from_16_bit(offset as u16);
            let remaining_bytes = macro_buffer_size - offset;
            let bytes = vec![offset_bytes.0, offset_bytes.1, DATA_BUFFER_SIZE as u8];
            match self.hid_command(ViaCommandId::DynamicKeymapMacroGetBuffer, bytes) {
                Some(val) => {
                    if remaining_bytes < DATA_BUFFER_SIZE {
                        all_bytes.extend(val[4..(4 + remaining_bytes)].to_vec())
                    } else {
                        all_bytes.extend(val[4..].to_vec())
                    }
                }
                None => return None,
            }
        }
        Some(all_bytes)
    }

    /// Sets the macro bytes.
    pub fn set_macro_bytes(&self, data: Vec<u8>) -> Option<()> {
        let macro_buffer_size = self.get_macro_buffer_size()?;
        let size = data.len();
        if size > macro_buffer_size as usize {
            return None;
        }

        self.reset_macros()?;

        let last_offset = macro_buffer_size - 1;
        let last_offset_bytes = utils::shift_from_16_bit(last_offset);

        // Set last byte in buffer to non-zero (0xFF) to indicate write-in-progress
        self.hid_command(
            ViaCommandId::DynamicKeymapMacroSetBuffer,
            vec![last_offset_bytes.0, last_offset_bytes.1, 1, 0xff],
        )?;

        for offset in (0..data.len()).step_by(DATA_BUFFER_SIZE) {
            let offset_bytes = utils::shift_from_16_bit(offset as u16);
            let end = std::cmp::min(offset + DATA_BUFFER_SIZE, data.len());
            let buffer = data[offset..end].to_vec();
            let mut bytes = vec![offset_bytes.0, offset_bytes.1, buffer.len() as u8];
            bytes.extend(buffer);
            self.hid_command(ViaCommandId::DynamicKeymapMacroSetBuffer, bytes)?;
        }

        // Set last byte in buffer to zero to indicate write finished
        self.hid_command(
            ViaCommandId::DynamicKeymapMacroSetBuffer,
            vec![last_offset_bytes.0, last_offset_bytes.1, 1, 0x00],
        )?;

        Some(())
    }

    /// Resets all saved macros.
    pub fn reset_macros(&self) -> Option<()> {
        let bytes = vec![];
        self.hid_command(ViaCommandId::DynamicKeymapMacroReset, bytes)
            .map(|_| ())
    }

    /// Resets the EEPROM, clearing all settings like keymaps and macros.
    pub fn reset_eeprom(&self) -> Option<()> {
        let bytes = vec![];
        self.hid_command(ViaCommandId::EepromReset, bytes)
            .map(|_| ())
    }

    /// Jumps to the bootloader.
    pub fn jump_to_bootloader(&self) -> Option<()> {
        let bytes = vec![];
        self.hid_command(ViaCommandId::BootloaderJump, bytes)
            .map(|_| ())
    }
}
