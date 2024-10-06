pub fn u8_to_i8(value: u8) -> i8 {
    i8::from_ne_bytes(value.to_ne_bytes())
}

pub fn i8_to_u8(value: i8) -> u8 {
    u8::from_ne_bytes(value.to_ne_bytes())
}

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

pub fn sign_extend_12bit_to_16bit(value: u16) -> i16 {
    ((value << 4) as i16) >> 4
}

pub fn sign_extend_12bit_to_32bit(value: u16) -> i32 {
    (((value as u32) << 20) as i32) >> 20
}

pub fn sign_extend_5bit_to_32bit(value: u8) -> u32 {
    ((((value as u32) << 27) as i32) >> 27) as u32
}
