use itertools::*;
use std::{
    borrow::BorrowMut,
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex},
};

use constant::{DATA_PREFIX, DATA_SUFFIX};

#[rustfmt::skip]
mod constant {
    pub const DATA_PREFIX: u8 = 0x9b;
    pub const DATA_SUFFIX: u8 = 0x9d;

    pub const DIGIT_0: u8 = 0x3f;
    pub const DIGIT_1: u8 = 0x06;
    pub const DIGIT_2: u8 = 0x5b;
    pub const DIGIT_3: u8 = 0x4f;
    pub const DIGIT_4: u8 = 0x66;
    pub const DIGIT_5: u8 = 0x6d;
    pub const DIGIT_6: u8 = 0x7c;
    pub const DIGIT_7: u8 = 0x07;
    pub const DIGIT_8: u8 = 0x7f;
    pub const DIGIT_9: u8 = 0x6f;
    pub const DIGIT_BYTES: &[u8] = &[
        DIGIT_0,
        DIGIT_1,
        DIGIT_2,
        DIGIT_3,
        DIGIT_4,
        DIGIT_5,
        DIGIT_6,
        DIGIT_7,
        DIGIT_8,
        DIGIT_9,
    ];

    pub const MASK_DIGIT: u8 = 0x7f;
    pub const MASK_DOT  : u8 = 0x80;
}

#[derive(Debug)]
pub enum FlexispotQueryError {
    DeviceIsSleeped,
    InvalidDigitByte(u8),
}
impl Display for FlexispotQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for FlexispotQueryError {}

#[derive(Debug)]
pub struct FlexispotQueryResult(Vec<Vec<u8>>);

impl FlexispotQueryResult {
    pub fn new(bytes_with_prefix_and_suffix: &[u8]) -> Self {
        let bytes_split = bytes_with_prefix_and_suffix
            .split(|&b| b == DATA_PREFIX || b == DATA_SUFFIX)
            .filter(|bs| !bs.is_empty())
            .map(|bs| bs.to_vec())
            .collect_vec();
        Self(bytes_split)
    }
    pub fn parse(&self) -> Vec<FlexispotPacket> {
        self.0
            .iter()
            .map(|bytes| FlexispotPacket::from_bytes_without_sep(bytes))
            .collect_vec()
    }
}

#[derive(Debug, PartialEq)]
pub enum FlexispotPacket {
    Unknown,
    Sleep,
    CurrentHeight(FlexispotCurrentHeight),
}
impl FlexispotPacket {
    pub fn from_bytes_without_sep(bytes: &[u8]) -> Self {
        if bytes.len() != 7 {
            return Self::Unknown;
        }
        match FlexispotCurrentHeight::from_digit_bytes(&[bytes[2], bytes[3], bytes[4]]) {
            Ok(height) => Self::CurrentHeight(height),
            Err(FlexispotQueryError::DeviceIsSleeped) => Self::Sleep,
            Err(_) => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FlexispotCurrentHeight(f32);

impl FlexispotCurrentHeight {
    fn digit_byte_to_u32(digit_byte: u8) -> Option<u32> {
        constant::DIGIT_BYTES
            .iter()
            .position(|&d| d == (digit_byte & constant::MASK_DIGIT))
            .map(|u| u as u32)
    }
    pub fn from_digit_bytes(digit_bytes: &[u8; 3]) -> Result<Self, FlexispotQueryError> {
        if digit_bytes == &[0x00, 0x00, 0x00] {
            return Err(FlexispotQueryError::DeviceIsSleeped);
        }

        let mut decoded: u32 = 0;
        for &digit_byte in digit_bytes {
            decoded *= 10;
            decoded += Self::digit_byte_to_u32(digit_byte)
                .ok_or(FlexispotQueryError::InvalidDigitByte(digit_byte))?;
        }
        let mut decoded: f32 = decoded as f32;

        let contains_decimal_point = (digit_bytes[1] & constant::MASK_DOT) != 0x00;
        if contains_decimal_point {
            decoded /= 10.0;
        }
        Ok(Self(decoded))
    }
    pub fn to_f32(&self) -> f32 {
        self.0
    }
}

#[derive(Debug)]
pub struct FlexispotQueryProcessor {
    uart: Arc<Mutex<rppal::uart::Uart>>,
}

impl FlexispotQueryProcessor {
    pub fn new(uart: Arc<Mutex<rppal::uart::Uart>>) -> Self {
        Self { uart }
    }
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        let len = self
            .uart
            .lock()
            .unwrap()
            .borrow_mut()
            .read(buf)
            .map_err(|e| e.to_string())?;
        Ok(len)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_data_bytes() {
        let bytes = &[
            0xC3, 0x9D, // broken bytes
            0x9B, 0x04, 0x11, 0x7C, 0xC3, 0x9D, // unknown
            0x9B, 0x04, 0x11, 0x7C, 0xC3, 0x9D, // unknown
            0x9B, 0x04, 0x11, 0x7C, 0xC3, 0x9D, // unknown
            0x9B, 0x07, 0x12, 0x07, 0xED, 0x6F, 0x05, 0x28, 0x9D, // 75.9
            0x9B, 0x04, 0x11, 0x7C, 0xC3, 0x9D, // unknown
            0x9B, 0x07, 0x12, 0x00, 0x00, 0x00, 0xB8, 0x94, 0x9D, // sleeping
            0x9B, // broken bytes
        ];
        let res = FlexispotQueryResult::new(bytes).parse();
        let expected = [
            FlexispotPacket::Unknown,
            FlexispotPacket::Unknown,
            FlexispotPacket::Unknown,
            FlexispotPacket::Unknown,
            FlexispotPacket::CurrentHeight(FlexispotCurrentHeight(75.9)),
            FlexispotPacket::Unknown,
            FlexispotPacket::Sleep,
        ];
        assert_eq!(res, expected);
    }
}
