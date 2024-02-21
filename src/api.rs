use crate::api_commands::ApiCommand;
use core::panic;
use std::error::Error;
use std::{error, io};

use crate::utils::shift_to_16_bit;

pub struct KeyboardApi {
    device: hidapi::HidDevice,
}

const COMMAND_START: u8 = 0x00;

impl KeyboardApi {
    pub fn new(device: hidapi::HidDevice) -> KeyboardApi {
        KeyboardApi { device }
    }

    pub fn send_data(&self, data: &[u8]) {
        let _ = self.device.write(data);
    }

    // async getByteBuffer(): Promise<Uint8Array> {
    //     return this.getHID().readP();
    //   }

    pub fn get_protocol_version(&self) -> Option<u16> {
        match self.hid_command(ApiCommand::GET_PROTOCOL_VERSION, vec![]) {
            Some(received_bytes) => Some(shift_to_16_bit(received_bytes[1], received_bytes[2])),
            None => None,
        }
    }

    //   async getKey(layer: Layer, row: Row, col: Column) {
    //     const buffer = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_GET_KEYCODE,
    //       [layer, row, col],
    //     );
    //     return shiftTo16Bit([buffer[4], buffer[5]]);
    //   }

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

    //   async getKeyboardValue(
    //     command: KeyboardValue,
    //     parameters: number[],
    //     resultLength = 1,
    //   ): Promise<number[]> {
    //     const bytes = [command, ...parameters];
    //     const res = await self.hid_command(APICommand.GET_KEYBOARD_VALUE, bytes);
    //     return res.slice(1 + bytes.length, 1 + bytes.length + resultLength);
    //   }

    //   async setKeyboardValue(command: KeyboardValue, ...rest: number[]) {
    //     const bytes = [command, ...rest];
    //     await self.hid_command(APICommand.SET_KEYBOARD_VALUE, bytes);
    //   }

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

    //   async setEncoderValue(
    //     layer: number,
    //     id: number,
    //     isClockwise: boolean,
    //     keycode: number,
    //   ): Promise<void> {
    //     const bytes = [layer, id, +isClockwise, ...shiftFrom16Bit(keycode)];
    //     await self.hid_command(APICommand.DYNAMIC_KEYMAP_SET_ENCODER, bytes);
    //   }

    //   async getCustomMenuValue(commandBytes: number[]): Promise<number[]> {
    //     const res = await self.hid_command(
    //       APICommand.CUSTOM_MENU_GET_VALUE,
    //       commandBytes,
    //     );
    //     return res.slice(0 + commandBytes.length);
    //   }

    //   async setCustomMenuValue(...args: number[]): Promise<void> {
    //     await self.hid_command(APICommand.CUSTOM_MENU_SET_VALUE, args);
    //   }

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

    //   async setBacklightValue(command: LightingValue, ...rest: number[]) {
    //     const bytes = [command, ...rest];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    //   async getRGBMode() {
    //     const bytes = [BACKLIGHT_EFFECT];
    //     const [, , val] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return val;
    //   }

    //   async getBrightness() {
    //     const bytes = [BACKLIGHT_BRIGHTNESS];
    //     const [, , brightness] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return brightness;
    //   }

    //   async getColor(colorNumber: number) {
    //     const bytes = [colorNumber === 1 ? BACKLIGHT_COLOR_1 : BACKLIGHT_COLOR_2];
    //     const [, , hue, sat] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return {hue, sat};
    //   }

    //   async setColor(colorNumber: number, hue: number, sat: number) {
    //     const bytes = [
    //       colorNumber === 1 ? BACKLIGHT_COLOR_1 : BACKLIGHT_COLOR_2,
    //       hue,
    //       sat,
    //     ];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    //   async getCustomColor(colorNumber: number) {
    //     const bytes = [BACKLIGHT_CUSTOM_COLOR, colorNumber];
    //     const [, , , hue, sat] = await self.hid_command(
    //       APICommand.BACKLIGHT_CONFIG_GET_VALUE,
    //       bytes,
    //     );
    //     return {hue, sat};
    //   }

    //   async setCustomColor(colorNumber: number, hue: number, sat: number) {
    //     const bytes = [BACKLIGHT_CUSTOM_COLOR, colorNumber, hue, sat];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    //   async setRGBMode(effect: number) {
    //     const bytes = [BACKLIGHT_EFFECT, effect];
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SET_VALUE, bytes);
    //   }

    //   async commitCustomMenu(channel: number) {
    //     await self.hid_command(APICommand.CUSTOM_MENU_SAVE, [channel]);
    //   }

    //   async saveLighting() {
    //     await self.hid_command(APICommand.BACKLIGHT_CONFIG_SAVE);
    //   }

    //   async resetEEPROM() {
    //     await self.hid_command(APICommand.EEPROM_RESET);
    //   }

    //   async jumpToBootloader() {
    //     await self.hid_command(APICommand.BOOTLOADER_JUMP);
    //   }

    //   async setKey(layer: Layer, row: Row, column: Column, val: number) {
    //     const res = await self.hid_command(APICommand.DYNAMIC_KEYMAP_SET_KEYCODE, [
    //       layer,
    //       row,
    //       column,
    //       ...shiftFrom16Bit(val),
    //     ]);
    //     return shiftTo16Bit([res[4], res[5]]);
    //   }

    //   async getMacroCount() {
    //     const [, count] = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_MACRO_GET_COUNT,
    //     );
    //     return count;
    //   }

    //   // size is 16 bit
    //   async getMacroBufferSize() {
    //     const [, hi, lo] = await self.hid_command(
    //       APICommand.DYNAMIC_KEYMAP_MACRO_GET_BUFFER_SIZE,
    //     );
    //     return shiftTo16Bit([hi, lo]);
    //   }

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

    //   async resetMacros() {
    //     await self.hid_command(APICommand.DYNAMIC_KEYMAP_MACRO_RESET);
    //   }

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
            eprintln!(
                "Command for kb_addr: {:?}, Bad Resp: {:?}",
                command_bytes, buffer
            );
            return None;
        }

        println!(
            "Command for kb_addr: {:?}, Correct Resp: {:?}",
            command_bytes, buffer
        );
        Some(buffer)
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
