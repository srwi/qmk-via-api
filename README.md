# qmk-via-api

[![Version](https://img.shields.io/crates/v/qmk-via-api.svg)](https://crates.io/crates/qmk-via-api)
[![image](https://img.shields.io/pypi/v/qmk-via-api.svg)](https://pypi.python.org/pypi/qmk-via-api)
[![image](https://img.shields.io/pypi/l/qmk-via-api.svg)](https://pypi.python.org/pypi/qmk-via-api)

`qmk-via-api` provides an implementation of the [VIA](https://www.caniusevia.com/docs/specification) API for [QMK](https://github.com/qmk/qmk_firmware) (Quantum Mechanical Keyboard) based keyboards. It allows developers to interact with QMK keyboards programmatically, enabling tasks such as configuring keymaps, macros, lighting effects and more.

Additionally, this library includes Python bindings for all API calls for integration of QMK keyboard configuration into Python-based applications or scripts.

# Usage

## Rust

Add dependency with Cargo:

```bash
cargo add qmk-via-api
```

Usage example:

```rust
use qmk_via_api::{api::KeyboardApi, scan::scan_keyboards};

fn main() {
    if let Some(dev) = scan_keyboards().first() {
        let api = KeyboardApi::new(dev.vendor_id, dev.product_id, dev.usage_page).unwrap();
        println!("Protocol version: {:?}", api.get_protocol_version());
        println!("Layer count: {:?}", api.get_layer_count());
    } else {
        println!("No devices found");
    }
}
```

## Python

Install with pip:

```bash
pip install qmk-via-api
```

Usage example:

```python
import qmk_via_api
from qmk_via_api import scan_keyboards

devices = scan_keyboards()
if devices:
    dev = devices[0]
    api = qmk_via_api.KeyboardApi(dev.vendor_id, dev.product_id, dev.usage_page)
    print(f"Protocol version {api.get_protocol_version()}")
    print(f"Layers count: {api.get_layer_count()}")
else:
    print("No devices found")
```

# License & Attribution

Parts of this project are based on code from [the VIA project](https://github.com/the-via/app), which is licensed under the GNU General Public License v3.0.