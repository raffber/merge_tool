use crate::config::{Config, FwConfig};
use std::path::Path;
use crate::Error;
use crate::firmware::Firmware;
use crate::crc::crc32;


pub fn merge_firmware(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    let app = load_app(config, fw_config)?;
    let btl = load_btl(config, fw_config)?;
    Firmware::merge(btl, app)
}

pub fn create_script(config: &Config, output_dir: &Path) -> Result<(), Error> {
    todo!()
}

pub fn write_crc(fw: &mut Firmware) {
    let crc = crc32(&fw.data[4..fw.image_length()]);
    fw.write_u32(0, crc);
}

pub fn merge_and_write_all(config: &Config, target_folder: &Path) -> Result<(), Error> {
    for fw_config in &config.images {
        let fw = merge_firmware(config, fw_config)?;
        let file_name = format!("{}.{}", fw_config.designator(), fw_config.hex_file_format.file_extension());
        let fpath = target_folder.join(file_name);
        fw.write_to_file(&fpath, &fw_config.hex_file_format)?;
    }
    Ok(())
}

pub fn load_app(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    todo!()
}

pub fn load_btl(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    todo!()
}


