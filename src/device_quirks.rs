//! Device-specific differences from the standard QMK VIA implementation.

pub(crate) fn rgb_matrix_max_brightness(
    vid: u16,
    pid: u16,
    usage_page: u16,
    protocol_version: u16,
) -> u8 {
    match (vid, pid, usage_page, protocol_version) {
        // Crush 80 wired VIA: channel 3, value 1 has range [0, 9].
        // Restrict this to the verified interface and protocol.
        (0x320f, 0x5055, 0xff60, 11) => 9,
        _ => 255,
    }
}

#[cfg(test)]
mod tests {
    use crate::{KeyboardApi, Result, ViaTransport};

    struct BrightnessTransport {
        last: Vec<u8>,
        brightness: u8,
        protocol: u16,
    }

    impl ViaTransport for BrightnessTransport {
        fn write_report(&mut self, report: &[u8]) -> Result<()> {
            self.last = report.to_vec();
            if report.starts_with(&[7, 3, 1]) {
                self.brightness = report[3];
            }
            Ok(())
        }

        fn read_report(&mut self, _: Option<i32>) -> Result<Vec<u8>> {
            match self.last.as_slice() {
                [1] => Ok(vec![1, (self.protocol >> 8) as u8, self.protocol as u8]),
                [8, 3, 1] => Ok(vec![8, 3, 1, self.brightness]),
                _ => Ok(self.last.clone()),
            }
        }
    }

    #[test]
    fn brightness_normalization_uses_transport_identity_and_protocol() {
        let crush80 = Some((0x320f, 0x5055, 0xff60));
        for (identity, protocol, input, native, readback) in [
            (crush80, 11, 0, 0, 0),
            (crush80, 11, 228, 8, 227),
            (crush80, 11, 255, 9, 255),
            (None, 11, 228, 228, 228),
            (Some((0x1234, 0x5055, 0xff60)), 11, 228, 228, 228),
            (Some((0x320f, 0x5088, 0xff60)), 11, 228, 228, 228),
            (Some((0x320f, 0x5055, 0xffef)), 11, 228, 228, 228),
            (crush80, 12, 228, 228, 228),
        ] {
            let transport = Box::new(BrightnessTransport {
                last: Vec::new(),
                brightness: 0,
                protocol,
            });
            let api = match identity {
                Some((vid, pid, usage_page)) => KeyboardApi::from_transport_with_identity(
                    transport,
                    vid,
                    pid,
                    usage_page,
                    Some(100),
                ),
                None => KeyboardApi::from_transport(transport, Some(100)),
            }
            .unwrap();
            api.set_rgb_matrix_brightness(input).unwrap();
            assert_eq!(api.hid_read().unwrap(), [7, 3, 1, native]);
            assert_eq!(api.get_rgb_matrix_brightness().unwrap(), readback);
        }
    }
}
