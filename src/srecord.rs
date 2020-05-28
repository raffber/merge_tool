use std::path::Path;
use crate::Error;
use crate::config::{DeviceConfig, AddressRange};

pub fn load(path: &Path, config: &DeviceConfig, range: &AddressRange) -> Result<Vec<u8>, Error> {
    todo!()
}

pub fn save(path: &Path, config: &DeviceConfig, range: &AddressRange, data: &Vec<u8>) -> Result<(), Error> {
    todo!()
}
