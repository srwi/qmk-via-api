use itertools::Itertools;

pub fn get_bit_at_index(byte: u8, bit_index: u8) -> bool {
    let mask = 0b10000000 >> bit_index;

    mask & byte != 0
}

pub fn set_bit_at_index(byte: u8, bit_index: u8, enabled: bool) -> u8 {
    let mask = 0b10000000 >> bit_index;

    if enabled {
        mask | byte
    } else {
        (mask ^ 0b11111111) & byte
    }
}

pub fn shift_to_16_bit(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | lo as u16
}

pub fn shift_from_16_bit(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

pub fn shift_buffer_to_16_bit(buffer: &[u8]) -> Vec<u16> {
    let mut shifted_buffer = Vec::new();
    for (hi, lo) in buffer.iter().tuples() {
        shifted_buffer.push(shift_to_16_bit(*hi, *lo));
    }
    shifted_buffer
}

pub fn shift_buffer_from_16_bit(buffer: &[u16]) -> Vec<u8> {
    let mut flattened = Vec::new();
    for value in buffer.iter() {
        let (hi, lo) = shift_from_16_bit(*value);
        flattened.push(hi);
        flattened.push(lo);
    }
    flattened
}
