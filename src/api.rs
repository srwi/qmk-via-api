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
    pub fn new(pid: u16, vid: u16, usage_page: u16) -> KeyboardApi {
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

    // async getByteBuffer(): Promise<Uint8Array> {
    //     return this.getHID().readP();
    //   }

    // async getProtocolVersion() {
    //     try {
    //       const [, hi, lo] = await this.hidCommand(APICommand.GET_PROTOCOL_VERSION);
    //       return shiftTo16Bit([hi, lo]);
    //     } catch (e) {
    //       return -1;
    //     }
    // }

    pub fn get_protocol_version(&self) -> Option<u16> {
        match self.hid_command(ApiCommand::GetProtocolVersion, vec![]) {
            Some(val) => Some(shift_to_16_bit(val[1], val[2])),
            None => None,
        }
    }

    // async getKey(layer: Layer, row: Row, col: Column) {
    //     const buffer = await this.hidCommand(
    //       APICommand.DYNAMIC_KEYMAP_GET_KEYCODE,
    //       [layer, row, col],
    //     );
    //     return shiftTo16Bit([buffer[4], buffer[5]]);
    // }

    pub fn get_key(&self, layer: Layer, row: Row, col: Column) -> Option<u16> {
        match self.hid_command(ApiCommand::DynamicKeymapGetKeycode, vec![layer, row, col]) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    //   async getLayerCount() {
    //     const version = await this.getProtocolVersion();
    //     if (version >= PROTOCOL_BETA) {
    //       const [, count] = await self.hid_command(
    //         APICommand.DYNAMIC_KEYMAP_GET_LAYER_COUNT,
    //       );
    //       return count;
    //     }

    //     return 4;
    //   }

    pub fn get_layer_count(&self) -> Option<u8> {
        match self.get_protocol_version() {
            Some(version) => {
                if version >= 0x0002 {
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

    //   async readRawMatrix(matrix: MatrixInfo, layer: number): Promise<Keymap> {
    //     const version = await this.getProtocolVersion();
    //     if (version >= PROTOCOL_BETA) {
    //       return this.fastReadRawMatrix(matrix, layer);
    //     }
    //     if (version === PROTOCOL_ALPHA) {
    //       return this.slowReadRawMatrix(matrix, layer);
    //     }
    //     throw new Error('Unsupported protocol version');
    //   }

    pub fn read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
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

    //   async getKeymapBuffer(offset: number, size: number): Promise<number[]> {
    //     if (size > 28) {
    //       throw new Error('Max data length is 28');
    //     }
    //     // id_dynamic_keymap_get_buffer <offset> <size> ^<data>
    //     // offset is 16bit. size is 8bit. data is 16bit keycode values, maximum 28 bytes.
    //     const res = await self.hid_command(APICommand.DYNAMIC_KEYMAP_GET_BUFFER, [
    //       ...shiftFrom16Bit(offset),
    //       size,
    //     ]);
    //     return [...res].slice(4, size + 4);
    //   }

    pub fn get_keymap_buffer(&self, offset: u16, size: u8) -> Option<Vec<u8>> {
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

    //   async fastReadRawMatrix(
    //     {rows, cols}: MatrixInfo,
    //     layer: number,
    //   ): Promise<number[]> {
    //     const length = rows * cols;
    //     const MAX_KEYCODES_PARTIAL = 14;
    //     const bufferList = new Array<number>(
    //       Math.ceil(length / MAX_KEYCODES_PARTIAL),
    //     ).fill(0);
    //     const {res: promiseRes} = bufferList.reduce(
    //       ({res, remaining}: {res: Promise<number[]>[]; remaining: number}) =>
    //         remaining < MAX_KEYCODES_PARTIAL
    //           ? {
    //               res: [
    //                 ...res,
    //                 this.getKeymapBuffer(
    //                   layer * length * 2 + 2 * (length - remaining),
    //                   remaining * 2,
    //                 ),
    //               ],
    //               remaining: 0,
    //             }
    //           : {
    //               res: [
    //                 ...res,
    //                 this.getKeymapBuffer(
    //                   layer * length * 2 + 2 * (length - remaining),
    //                   MAX_KEYCODES_PARTIAL * 2,
    //                 ),
    //               ],
    //               remaining: remaining - MAX_KEYCODES_PARTIAL,
    //             },
    //       {res: [], remaining: length},
    //     );
    //     const yieldedRes = await Promise.all(promiseRes);
    //     return yieldedRes.flatMap(shiftBufferTo16Bit);
    //   }

    pub fn fast_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
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

    //   async slowReadRawMatrix(
    //     {rows, cols}: MatrixInfo,
    //     layer: number,
    //   ): Promise<number[]> {
    //     const length = rows * cols;
    //     const res = new Array(length)
    //       .fill(0)
    //       .map((_, i) => this.getKey(layer, ~~(i / cols), i % cols));
    //     return Promise.all(res);
    //   }

    pub fn slow_read_raw_matrix(&self, matrix_info: MatrixInfo, layer: Layer) -> Option<Vec<u16>> {
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

    //   async writeRawMatrix(
    //     matrixInfo: MatrixInfo,
    //     keymap: number[][],
    //   ): Promise<void> {
    //     const version = await this.getProtocolVersion();
    //     if (version >= PROTOCOL_BETA) {
    //       return this.fastWriteRawMatrix(keymap);
    //     }
    //     if (version === PROTOCOL_ALPHA) {
    //       return this.slowWriteRawMatrix(matrixInfo, keymap);
    //     }
    //   }

    pub fn write_raw_matrix(&self, matrix_info: MatrixInfo, keymap: Vec<Vec<u16>>) -> Option<()> {
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

    //   async slowWriteRawMatrix(
    //     {cols}: MatrixInfo,
    //     keymap: number[][],
    //   ): Promise<void> {
    //     keymap.forEach(async (layer, layerIdx) =>
    //       layer.forEach(async (keycode, keyIdx) => {
    //         await this.setKey(layerIdx, ~~(keyIdx / cols), keyIdx % cols, keycode);
    //       }),
    //     );
    //   }

    pub fn slow_write_raw_matrix(
        &self,
        matrix_info: MatrixInfo,
        keymap: Vec<Vec<u16>>,
    ) -> Option<()> {
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

    //   async fastWriteRawMatrix(keymap: number[][]): Promise<void> {
    //     const data = keymap.flatMap((layer) => layer.map((key) => key));
    //     const shiftedData = shiftBufferFrom16Bit(data);
    //     const bufferSize = 28;
    //     for (let offset = 0; offset < shiftedData.length; offset += bufferSize) {
    //       const buffer = shiftedData.slice(offset, offset + bufferSize);
    //       await self.hid_command(APICommand.DYNAMIC_KEYMAP_SET_BUFFER, [
    //         ...shiftFrom16Bit(offset),
    //         buffer.length,
    //         ...buffer,
    //       ]);
    //     }
    //   }

    pub fn fast_write_raw_matrix(&self, keymap: Vec<Vec<u16>>) -> Option<()> {
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

    //   async getKeyboardValue(
    //     command: KeyboardValue,
    //     parameters: number[],
    //     resultLength = 1,
    //   ): Promise<number[]> {
    //     const bytes = [command, ...parameters];
    //     const res = await self.hid_command(APICommand.GET_KEYBOARD_VALUE, bytes);
    //     return res.slice(1 + bytes.length, 1 + bytes.length + resultLength);
    //   }

    pub fn get_keyboard_value(
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

    //   async setKeyboardValue(command: KeyboardValue, ...rest: number[]) {
    //     const bytes = [command, ...rest];
    //     await self.hid_command(APICommand.SET_KEYBOARD_VALUE, bytes);
    //   }

    pub fn set_keyboard_value(&self, command: KeyboardValue, rest: Vec<u8>) -> Option<()> {
        let mut bytes = vec![command as u8];
        bytes.extend(rest);
        match self.hid_command(ApiCommand::SetKeyboardValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async getEncoderValue(
    //     layer: number,
    //     id: number,
    //     isClockwise: boolean,
    //   ): Promise<number> {
    //     const bytes = [layer, id, +isClockwise];
    //     const res = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_GET_ENCODER,
    //       bytes,
    //     );
    //     return shiftTo16Bit([res[4], res[5]]);
    //   }

    pub fn get_encoder_value(&self, layer: Layer, id: u8, is_clockwise: bool) -> Option<u16> {
        match self.hid_command(
            ApiCommand::DynamicKeymapGetEncoder,
            vec![layer, id, is_clockwise as u8],
        ) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    //   async setEncoderValue(
    //     layer: number,
    //     id: number,
    //     isClockwise: boolean,
    //     keycode: number,
    //   ): Promise<void> {
    //     const bytes = [layer, id, +isClockwise, ...shiftFrom16Bit(keycode)];
    //     await self.hid_command(APICommand.DYNAMIC_KEYMAP_SET_ENCODER, bytes);
    //   }

    pub fn set_encoder_value(
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

    //   async getCustomMenuValue(commandBytes: number[]): Promise<number[]> {
    //     const res = await self.hid_command(
    //       APICommand.CUSTOM_MENU_GET_VALUE,
    //       commandBytes,
    //     );
    //     return res.slice(0 + commandBytes.length);
    //   }

    pub fn get_custom_menu_value(&self, command_bytes: Vec<u8>) -> Option<Vec<u8>> {
        let command_length = command_bytes.len();
        match self.hid_command(ApiCommand::CustomMenuGetValue, command_bytes) {
            Some(val) => Some(val[0..command_length].to_vec()),
            None => None,
        }
    }

    //   async setCustomMenuValue(...args: number[]): Promise<void> {
    //     await self.hid_command(APICommand.CUSTOM_MENU_SET_VALUE, args);
    //   }

    pub fn set_custom_menu_value(&self, args: Vec<u8>) -> Option<()> {
        match self.hid_command(ApiCommand::CustomMenuSetValue, args) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async getPerKeyRGBMatrix(ledIndexMapping: number[]): Promise<number[][]> {
    //     const res = await Promise.all(
    //       ledIndexMapping.map((ledIndex) =>
    //         self.hid_command(APICommand.CUSTOM_MENU_GET_VALUE, [
    //           ...PER_KEY_RGB_CHANNEL_COMMAND,
    //           ledIndex,
    //           1, // count
    //         ]),
    //       ),
    //     );
    //     return res.map((r) => [...r.slice(5, 7)]);
    //   }

    pub fn get_per_key_rgb_matrix(&self, led_index_mapping: Vec<u8>) -> Option<Vec<Vec<u8>>> {
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

    //   async setPerKeyRGBMatrix(
    //     index: number,
    //     hue: number,
    //     sat: number,
    //   ): Promise<void> {
    //     await self.hid_command(APICommand.CUSTOM_MENU_SET_VALUE, [
    //       ...PER_KEY_RGB_CHANNEL_COMMAND,
    //       index,
    //       1, // count
    //       hue,
    //       sat,
    //     ]);
    //   }

    pub fn set_per_key_rgb_matrix(&self, index: u8, hue: u8, sat: u8) -> Option<()> {
        let mut bytes = PER_KEY_RGB_CHANNEL_COMMAND.to_vec();
        bytes.extend(vec![index, 1, hue, sat]);
        // let bytes = [PER_KEY_RGB_CHANNEL_COMMAND, &vec![index, 1, hue, sat].as_slice()].concat();
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async getBacklightValue(
    //     command: LightingValue,
    //     resultLength = 1,
    //   ): Promise<number[]> {
    //     const bytes = [command];
    //     const res = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return res.slice(2, 2 + resultLength);
    //   }

    pub fn get_backlight_value(
        &self,
        command: ApiCommand,
        result_length: usize,
    ) -> Option<Vec<u8>> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![command as u8]) {
            Some(val) => Some(val[2..result_length + 2].to_vec()),
            None => None,
        }
    }

    //   async setBacklightValue(command: LightingValue, ...rest: number[]) {
    //     const bytes = [command, ...rest];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    pub fn set_backlight_value(&self, command: ApiCommand, rest: Vec<u8>) -> Option<()> {
        let mut bytes: Vec<u8> = vec![command as u8];
        bytes.extend(rest);
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async getRGBMode() {
    //     const bytes = [BACKLIGHT_EFFECT];
    //     const [, , val] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return val;
    //   }

    pub fn get_rgb_mode(&self) -> Option<u8> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![BACKLIGHT_EFFECT]) {
            Some(val) => Some(val[2]),
            None => None,
        }
    }

    //   async getBrightness() {
    //     const bytes = [BACKLIGHT_BRIGHTNESS];
    //     const [, , brightness] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return brightness;
    //   }

    pub fn get_brightness(&self) -> Option<u8> {
        match self.hid_command(ApiCommand::CustomMenuGetValue, vec![BACKLIGHT_BRIGHTNESS]) {
            Some(val) => Some(val[2]),
            None => None,
        }
    }

    //   async getColor(colorNumber: number) {
    //     const bytes = [colorNumber === 1 ? BACKLIGHT_COLOR_1 : BACKLIGHT_COLOR_2];
    //     const [, , hue, sat] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return {hue, sat};
    //   }

    pub fn get_color(&self, color_number: u8) -> Option<(u8, u8)> {
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

    //   async setColor(colorNumber: number, hue: number, sat: number) {
    //     const bytes = [
    //       colorNumber === 1 ? BACKLIGHT_COLOR_1 : BACKLIGHT_COLOR_2,
    //       hue,
    //       sat,
    //     ];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    pub fn set_color(&self, color_number: u8, hue: u8, sat: u8) -> Option<()> {
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

    //   async getCustomColor(colorNumber: number) {
    //     const bytes = [BACKLIGHT_CUSTOM_COLOR, colorNumber];
    //     const [, , , hue, sat] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return {hue, sat};
    //   }

    pub fn get_custom_color(&self, color_number: u8) -> Option<(u8, u8)> {
        let bytes = vec![BACKLIGHT_CUSTOM_COLOR, color_number];
        match self.hid_command(ApiCommand::CustomMenuGetValue, bytes) {
            Some(val) => Some((val[3], val[4])),
            None => None,
        }
    }

    //   async setCustomColor(colorNumber: number, hue: number, sat: number) {
    //     const bytes = [BACKLIGHT_CUSTOM_COLOR, colorNumber, hue, sat];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    pub fn set_custom_color(&self, color_number: u8, hue: u8, sat: u8) -> Option<()> {
        let bytes = vec![BACKLIGHT_CUSTOM_COLOR, color_number, hue, sat];
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async setRGBMode(effect: number) {
    //     const bytes = [BACKLIGHT_EFFECT, effect];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    pub fn set_rgb_mode(&self, effect: u8) -> Option<()> {
        let bytes = vec![BACKLIGHT_EFFECT, effect];
        match self.hid_command(ApiCommand::CustomMenuSetValue, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async commitCustomMenu(channel: number) {
    //     await self.hid_command(APICommand.CUSTOM_MENU_SAVE, [channel]);
    //   }

    pub fn commit_custom_menu(&self, channel: u8) -> Option<()> {
        let bytes = vec![channel];
        match self.hid_command(ApiCommand::CustomMenuSave, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async saveLighting() {
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SAVE);
    //   }

    pub fn save_lighting(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::CustomMenuSave, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async resetEEPROM() {
    //     await self.hid_command(APICommand.EEPROM_RESET);
    //   }

    pub fn reset_eeprom(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::EepromReset, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async jumpToBootloader() {
    //     await self.hid_command(APICommand.BOOTLOADER_JUMP);
    //   }

    pub fn jump_to_bootloader(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::BootloaderJump, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async setKey(layer: Layer, row: Row, column: Column, val: number) {
    //     const res = await self.hid_command(APICommand.DYNAMIC_KEYMAP_SET_KEYCODE, [
    //       layer,
    //       row,
    //       column,
    //       ...shiftFrom16Bit(val),
    //     ]);
    //     return shiftTo16Bit([res[4], res[5]]);
    //   }

    pub fn set_key(&self, layer: Layer, row: Row, column: Column, val: u16) -> Option<u16> {
        let val_bytes = shift_from_16_bit(val);
        let bytes = vec![layer, row, column, val_bytes.0, val_bytes.1];
        match self.hid_command(ApiCommand::DynamicKeymapSetKeycode, bytes) {
            Some(val) => Some(shift_to_16_bit(val[4], val[5])),
            None => None,
        }
    }

    //   async getMacroCount() {
    //     const [, count] = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_MACRO_GET_COUNT,
    //     );
    //     return count;
    //   }

    pub fn get_macro_count(&self) -> Option<u8> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroGetCount, bytes) {
            Some(val) => Some(val[1]),
            None => None,
        }
    }

    //   // size is 16 bit
    //   async getMacroBufferSize() {
    //     const [, hi, lo] = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_MACRO_GET_BUFFER_SIZE,
    //     );
    //     return shiftTo16Bit([hi, lo]);
    //   }

    pub fn get_macro_buffer_size(&self) -> Option<u16> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroGetBufferSize, bytes) {
            Some(val) => Some(shift_to_16_bit(val[1], val[2])),
            None => None,
        }
    }

    //   // From protocol: id_dynamic_keymap_macro_get_buffer <offset> <size> ^<data>
    //   // offset is 16bit. size is 8bit.
    //   async getMacroBytes(): Promise<number[]> {
    //     const macroBufferSize = await this.getMacroBufferSize();
    //     // Can only get 28 bytes at a time
    //     const size = 28;
    //     const bytesP = [];
    //     for (let offset = 0; offset < macroBufferSize; offset += 28) {
    //       bytesP.push(
    //         self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_GET_BUFFER, [
    //           ...shiftFrom16Bit(offset),
    //           size,
    //         ]),
    //       );
    //     }
    //     const allBytes = await Promise.all(bytesP);
    //     return allBytes.flatMap((bytes) => bytes.slice(4));
    //   }

    pub fn get_macro_bytes(&self) -> Option<Vec<u8>> {
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

    //   // From protocol: id_dynamic_keymap_macro_set_buffer <offset> <size> <data>
    //   // offset is 16bit. size is 8bit. data is ASCII characters and null (0x00) delimiters/terminator, maximum 28 bytes.
    //   // async setMacros(macros: Macros[]) {
    //   async setMacroBytes(data: number[]) {
    //     const macroBufferSize = await this.getMacroBufferSize();
    //     const size = data.length;
    //     if (size > macroBufferSize) {
    //       throw new Error(
    //         `Macro size (${size}) exceeds buffer size (${macroBufferSize})`,
    //       );
    //     }

    //     const lastOffset = macroBufferSize - 1;
    //     const lastOffsetBytes = shiftFrom16Bit(lastOffset);

    //     // Clear the entire macro buffer before rewriting
    //     await this.resetMacros();
    //     try {
    //       // set last byte in buffer to non-zero (0xFF) to indicate write-in-progress
    //       await self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_SET_BUFFER, [
    //         ...shiftFrom16Bit(lastOffset),
    //         1,
    //         0xff,
    //       ]);

    //       // Can only write 28 bytes at a time
    //       const bufferSize = 28;
    //       for (let offset = 0; offset < data.length; offset += bufferSize) {
    //         const buffer = data.slice(offset, offset + bufferSize);
    //         await self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_SET_BUFFER, [
    //           ...shiftFrom16Bit(offset),
    //           buffer.length,
    //           ...buffer,
    //         ]);
    //       }
    //     } finally {
    //       // set last byte in buffer to zero to indicate write finished
    //       await self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_SET_BUFFER, [
    //         ...lastOffsetBytes,
    //         1,
    //         0x00,
    //       ]);
    //     }
    //   }

    pub fn set_macro_bytes(&self, data: Vec<u8>) -> Option<()> {
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
        match self.hid_command(
            ApiCommand::DynamicKeymapMacroSetBuffer,
            vec![last_offset_bytes.0, last_offset_bytes.1, 1, 0x00],
        ) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   async resetMacros() {
    //     await self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_RESET);
    //   }

    pub fn reset_macros(&self) -> Option<()> {
        let bytes = vec![];
        match self.hid_command(ApiCommand::DynamicKeymapMacroReset, bytes) {
            Some(_) => Some(()),
            None => None,
        }
    }

    //   get commandQueueWrapper() {
    //     if (!globalCommandQueue[this.kbAddr]) {
    //       globalCommandQueue[this.kbAddr] = {isFlushing: false, commandQueue: []};
    //       return globalCommandQueue[this.kbAddr];
    //     }
    //     return globalCommandQueue[this.kbAddr];
    //   }

    //   async timeout(time: number) {
    //     return new Promise((res, rej) => {
    //       this.commandQueueWrapper.commandQueue.push({
    //         res,
    //         rej,
    //         args: () =>
    //           new Promise((r) =>
    //             setTimeout(() => {
    //               r();
    //               res(undefined);
    //             }, time),
    //           ),
    //       });
    //       if (!this.commandQueueWrapper.isFlushing) {
    //         this.flushQueue();
    //       }
    //     });
    //   }

    pub fn hid_command(&self, command: ApiCommand, bytes: Vec<u8>) -> Option<Vec<u8>> {
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

    //   async flushQueue() {
    //     if (this.commandQueueWrapper.isFlushing === true) {
    //       return;
    //     }
    //     this.commandQueueWrapper.isFlushing = true;
    //     while (this.commandQueueWrapper.commandQueue.length !== 0) {
    //       const {res, rej, args} =
    //         this.commandQueueWrapper.commandQueue.shift() as CommandQueueEntry;
    //       // This allows us to queue promises in between hid commands, useful for timeouts
    //       if (typeof args === 'function') {
    //         await args();
    //         res();
    //       } else {
    //         try {
    //           const ans = await this._hidCommand(...args);
    //           res(ans);
    //         } catch (e: any) {
    //           const deviceInfo = extractDeviceInfo(this.getHID());
    //           store.dispatch(
    //             logAppError({
    //               message: getMessageFromError(e),
    //               deviceInfo,
    //             }),
    //           );
    //           rej(e);
    //         }
    //       }
    //     }
    //     this.commandQueueWrapper.isFlushing = false;
    //   }

    //   getHID() {
    //     return cache[this.kbAddr].hid;
    //   }

    //   async _hidCommand(command: Command, bytes: Array<number> = []): Promise<any> {
    //     const commandBytes = [...[COMMAND_START, command], ...bytes];
    //     const paddedArray = new Array(33).fill(0);
    //     commandBytes.forEach((val, idx) => {
    //       paddedArray[idx] = val;
    //     });

    //     await this.getHID().write(paddedArray);

    //     const buffer = Array.from(await this.getByteBuffer());
    //     const bufferCommandBytes = buffer.slice(0, commandBytes.length - 1);
    //     logCommand(this.kbAddr, commandBytes, buffer);
    //     if (!eqArr(commandBytes.slice(1), bufferCommandBytes)) {
    //       console.error(
    //         `Command for ${this.kbAddr}:`,
    //         commandBytes,
    //         'Bad Resp:',
    //         buffer,
    //       );

    //       const deviceInfo = extractDeviceInfo(this.getHID());
    //       const commandName = APICommandValueToName[command];
    //       store.dispatch(
    //         logKeyboardAPIError({
    //           commandName,
    //           commandBytes: commandBytes.slice(1),
    //           responseBytes: buffer,
    //           deviceInfo,
    //         }),
    //       );

    //       throw new Error('Receiving incorrect response for command');
    //     }
    //     console.debug(
    //       `Command for ${this.kbAddr}`,
    //       commandBytes,
    //       'Correct Resp:',
    //       buffer,
    //     );
    //     return buffer;
    //   }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use mockall::predicate::*;

//     #[test]
//     fn test_get_protocol_version() {
//         let mut mock = MockKeyboardApi::new(0, 0, 0);
//         mock.expect_hid_command()
//             .times(1)
//             .with(eq(ApiCommand::GetProtocolVersion), eq(vec![]))
//             .returning(|_, _| Ok([0, 0, 0, 0, 0, 0, 0, 0]));
//         assert_eq!(mock.get_protocol_version(), Some(0));
//     }
// }
