> [!WARNING]  
> `rust-via-api` is currently in very early development and mostly untested. Use at your own risk!

# rust-via-api

This Rust library provides an implementation of [VIA](https://www.caniusevia.com/docs/specification) for [QMK](https://github.com/qmk/qmk_firmware) (Quantum Mechanical Keyboard) firmware-based keyboards. It allows developers to interact with QMK keyboards programmatically through the VIA API, enabling tasks such as configuring keymaps, macros, lighting effects, and more.

Additionally, this library includes Python bindings for all API calls for integration of QMK keyboard configuration into Python-based applications or scripts.

# Usage

## Python

Install with pip:

```bash
pip install git+https://github.com/srwi/rust-via-api.git
```

Usage example:

```python
import rust_via_api

PRODUCT_VID = 0x604D
PRODUCT_PID = 0x594D
USAGE_PAGE = 0xff60

if __name__ == "__main__":
    api = rust_via_api.KeyboardApi(PRODUCT_VID, PRODUCT_PID, USAGE_PAGE)
    print(f"Protocol version {api.get_protocol_version()}")
    print(f"Layers count: {api.get_layer_count()}")
```

# License & Attribution

The Rust code in this project is based on code from [the VIA project](https://github.com/the-via/app), which is licensed under the GNU General Public License v3.0.