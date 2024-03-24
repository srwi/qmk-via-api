trait KeyboardApi {
    fn get_protocol_version(&self) -> u8 {
        // self.send("protocol_version");
        8
    }
    fn get_keyboard_layout(&self) -> Vec<u8> {
        // self.send("keyboard_layout");
        vec![0, 1, 2, 3]
    }
}

trait Device {
    fn setup(&self);
    fn send(&self, data: &str) -> Vec<u8>;
}

struct QmkKeyboard {
    device: u16,
}

struct MockKeyboard {
    response: Vec<u8>,
}

impl Device for QmkKeyboard {
    fn setup(&self) {
        println!("Setting up QMK keyboard {}", self.device);
    }

    fn send(&self, data: &str) -> Vec<u8> {
        println!("Actually sending data {}", data);
        vec![0; 128]
    }
}
impl KeyboardApi for QmkKeyboard {}

impl Device for MockKeyboard {
    fn setup(&self) {}

    fn send(&self, data: &str) -> Vec<u8> {
        self.response.clone()
    }
}
impl KeyboardApi for MockKeyboard {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_protocol_version() {
        let keyboard = MockKeyboard {
            response: vec![0, 8],
        };
        assert_eq!(keyboard.get_protocol_version(), 8);
    }
}
