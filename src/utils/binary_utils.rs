pub fn u16_to_i16(value: u16) -> i16 {
    i16::from_ne_bytes(value.to_ne_bytes())
}

pub fn i16_to_u16(value: i16) -> u16 {
    u16::from_ne_bytes(value.to_ne_bytes())
}

pub fn u32_to_i32(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

pub fn i32_to_u32(value: i32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

pub fn sign_extend_12bit_to_16bit(value: u16) -> i16{
    let positive: bool = (value & (0b1 << 11)) == 0;
    match positive {
        true => u16_to_i16(value),
        false => u16_to_i16(value | (0b1111 << 12))
    }
}

pub fn sign_extend_12bit_to_32bit(value: u16) -> i32{
    let positive: bool = (value & (0b1 << 11)) == 0;
    let padding: u32 = match positive {
        true => 0x0,
        false => 0xFFFFF000,
    };
    u32_to_i32(padding | value as u32)
}
