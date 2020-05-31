#![allow(dead_code)]

use std::fmt;

mod config;
mod firmware;
mod intel_hex;
mod srecord;
mod crc;
mod command;
mod script;
mod protocol;
mod xcmd;
mod process;
mod header;

#[derive(Debug)]
pub enum Error {
    AddressRangeNotAlignedToPage,
    InvalidAddress,
    Io(std::io::Error),
    InvalidHexFile,
    InvalidConfig(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub fn swap_bytearray(data: &mut Vec<u8>) {
    for k in (0..data.len()).step_by(2) {
        data.swap(k, k+1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_bytearray() {
        let mut arr: Vec<u8> = vec![1, 2, 3, 4, 5, 6];
        swap_bytearray(&mut arr);
        assert_eq!(&arr, &[2, 1, 4, 3, 6, 5]);
    }
}