> [!WARNING]  
> `qmk-via-api` is currently in very early development and mostly untested. Use at your own risk!

# qmk-via-api

`qmk-via-api` provides an implementation of the [VIA](https://www.caniusevia.com/docs/specification) API for [QMK](https://github.com/qmk/qmk_firmware) (Quantum Mechanical Keyboard) firmware-based keyboards. It allows developers to interact with QMK keyboards programmatically, enabling tasks such as configuring keymaps, macros, lighting effects and more.

Additionally, this library includes Python bindings for all API calls for integration of QMK keyboard configuration into Python-based applications or scripts.

# Usage

## Rust

Add to `Cargo.toml` dependencies:

```toml
[dependencies]
qmk-via-api = { git = "https://github.com/srwi/qmk-via-api" }
```

Usage example:

```rust
use qmk_via_api::api::KeyboardApi;

const PRODUCT_VID: u16 = 0x594D;
const PRODUCT_PID: u16 = 0x604D;
const USAGE_PAGE: u16 = 0xff60;

fn main() {
    let api = KeyboardApi::new(PRODUCT_VID, PRODUCT_PID, USAGE_PAGE);
    println!("Protocol version: {:?}", api.get_protocol_version());
    println!("Layer count: {:?}", api.get_layer_count());
}
```

## Python

Install with pip:

```bash
pip install git+https://github.com/srwi/qmk-via-api.git
```

Usage example:

```python
import qmk_via_api

PRODUCT_VID = 0x594D
PRODUCT_PID = 0x604D
USAGE_PAGE = 0xff60

if __name__ == "__main__":
    api = qmk_via_api.KeyboardApi(PRODUCT_VID, PRODUCT_PID, USAGE_PAGE)
    print(f"Protocol version {api.get_protocol_version()}")
    print(f"Layers count: {api.get_layer_count()}")
```

# License & Attribution

The Rust code in this project is based on code from [the VIA project](https://github.com/the-via/app), which is licensed under the GNU General Public License v3.0.