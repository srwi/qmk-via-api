use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Copy)]
pub enum ApiCommand {
    GetProtocolVersion = 0x01,
    GetKeyboardValue = 0x02,
    SetKeyboardValue = 0x03,
    DynamicKeymapGetKeycode = 0x04,
    DynamicKeymapSetKeycode = 0x05,
    DynamicKeymapClearAll = 0x06,
    CustomMenuSetValue = 0x07, // Deprecated alias: BACKLIGHT_CONFIG_SET_VALUE
    CustomMenuGetValue = 0x08, // Deprecated alias: BACKLIGHT_CONFIG_GET_VALUE
    CustomMenuSave = 0x09,     // Deprecated alias: BACKLIGHT_CONFIG_SAVE
    EepromReset = 0x0a,
    BootloaderJump = 0x0b,
    DynamicKeymapMacroGetCount = 0x0c,
    DynamicKeymapMacroGetBufferSize = 0x0d,
    DynamicKeymapMacroGetBuffer = 0x0e,
    DynamicKeymapMacroSetBuffer = 0x0f,
    DynamicKeymapMacroReset = 0x10,
    DynamicKeymapGetLayerCount = 0x11,
    DynamicKeymapGetBuffer = 0x12,
    DynamicKeymapSetBuffer = 0x13,
    DynamicKeymapGetEncoder = 0x14,
    DynamicKeymapSetEncoder = 0x15,
}
