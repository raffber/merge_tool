#[macro_use]
extern crate lazy_static;

use serde_json::Error as JsonError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{fmt, io};

pub mod app_package;
pub mod blocking_ddp;
pub mod changelog;
pub mod config;
pub mod crc;
pub mod ddp;
pub mod firmware;
pub mod git_description;
pub mod header;
pub mod intel_hex;
pub mod process;
pub mod protocol;
pub mod script;
pub mod script_cmd;
pub mod srecord;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    AddressRangeNotAlignedToPage,
    ImageTooShortForHeader,
    InvalidDataLength,
    InvalidAddress,
    Io(std::io::Error),
    InvalidHexFile,
    InvalidConfig(String),
    CannotParseConfig(JsonError),
    CannotFindGitRepo,
    InvalidProductName,
    CannotParseChangelog,
    Git(anyhow::Error),
    InvalidInfoFile(anyhow::Error),
}

impl From<io::Error> for Error {
    fn from(x: io::Error) -> Self {
        Error::Io(x)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn swap_bytearray(data: &mut Vec<u8>) {
    for k in (0..data.len()).step_by(2) {
        data.swap(k, k + 1)
    }
}

pub fn load_lines(path: &Path) -> Result<Vec<String>, Error> {
    let file = File::open(path).map_err(Error::Io)?;
    let lines = BufReader::new(file).lines();
    let mut ret = Vec::new();
    for line in lines {
        let x = line.map_err(Error::Io)?.trim().to_string();
        if !x.is_empty() {
            ret.push(x);
        }
    }
    Ok(ret)
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
