use std::fmt;

mod config;
mod base;
mod intel_hex;
mod srecord;
mod crc;

#[derive(Debug)]
pub enum Error {
    AddressRangeNotAlignedToPage,
    InvalidAddress,
    Io(std::io::Error),
    InvalidHexFile,
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