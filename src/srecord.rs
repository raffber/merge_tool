use std::path::Path;
use crate::Error;
use crate::config::AddressRange;

pub fn load(path: &Path, word_addressing: bool, range: &AddressRange) -> Result<Vec<u8>, Error> {
    todo!()
}

pub fn save(path: &Path, word_addressing: bool, range: &AddressRange, data: &Vec<u8>) -> Result<(), Error> {
    todo!()
}