use crate::config::{Config, EXT_CMD_CODE};
use std::path::Path;
use crate::Error;
use crate::firmware::Firmware;
use crate::crc::crc32;
use crate::xcmd::ExtCmdProtocol;
use crate::protocol::generate_script;
use crate::script::Script;
use std::fs::File;
use std::io::Write;
use crate::header::Header;


pub fn merge_firmware(config: &mut Config, idx: usize) -> Result<Firmware, Error> {
    let app = load_app(config, idx)?;
    let btl = load_btl(config, idx)?;
    Firmware::merge(btl, app)
}

fn generate_script_filename(config: &Config) -> String {
    let mut parts= Vec::new();
    parts.push(format!("{}_{}", config.product_name.clone(), config.major_version));
    for fw_config in &config.images {
        parts.push(format!("_{}.{}", fw_config.version.minor, fw_config.version.build));
    }
    parts.push(".gctbtl".to_string());
    parts.join("")
}

pub fn create_script(config: &mut Config, output_dir: &Path) -> Result<(), Error> {
    let protocol = ExtCmdProtocol::new(EXT_CMD_CODE);
    let mut fws = Vec::new();
    for idx in 0..config.images.len() {
        fws.push(load_app(config, idx)?);
    }
    let cmds = generate_script(&protocol, &fws, config);
    let filename  = generate_script_filename(config);

    let script = Script::new(cmds);
    let path = output_dir.join(&filename);
    let mut file = File::create(path).map_err(Error::Io)?;
    file.write_all(script.serialize().as_bytes()).map_err(Error::Io)
}

pub fn write_crc(fw: &mut Firmware) {
    let crc = crc32(&fw.data[4..fw.image_length()]);
    fw.write_u32(0, crc);
}

pub fn merge_all(config: &mut Config) -> Result<Vec<Firmware>, Error> {
    let mut ret = Vec::new();
    for idx in 0 .. config.images.len() {
        let fw = merge_firmware(config, idx)?;
        ret.push(fw);
    }
    Ok(ret)
}

pub fn write_fws(config: &Config, fws: &[Firmware], target_folder: &Path) -> Result<(), Error> {
    for (fw, fw_config) in fws.iter().zip(config.images.iter()) {
        let file_name = format!("{}.{}", fw_config.designator(), fw_config.hex_file_format.file_extension());
        let fpath = target_folder.join(file_name);
        fw.write_to_file(&fpath, &fw_config.hex_file_format)?;
    }
    Ok(())
}

pub fn load_app(config: &mut Config, idx: usize) -> Result<Firmware, Error> {
    let path = Path::new(&config.images[idx].app_path);
    let mut fw = Firmware::load_from_file(&path, &config.images[idx].hex_file_format,
        &config.images[idx].device_config, &config.images[idx].app_address)?;
    write_crc(&mut fw);
    let mut header = Header::new(&mut fw, config.images[idx].header_offset);
    if config.product_id != 0 && config.product_id != header.product_id() {
        return Err(Error::InvalidConfig(
            format!("Product ID in firmware and config does not match: {} vs. {}",
                    config.product_id, header.product_id()) ) );
    } else if config.product_id == 0 {
        config.product_id = header.product_id();
    }
    if config.major_version != 0xFF && config.major_version != header.major_version() {
        return Err(Error::InvalidConfig(
            format!("Major version in firmware and config does not match: {} vs. {}",
                    config.major_version, header.major_version()) ) );
    } else if config.major_version == 0xFF {
        config.major_version = header.major_version();
    } else if header.major_version() == 0xFF {
        header.set_major_version(config.major_version);
    }

    let minor = config.images[idx].version.minor;
    if minor != 0xFF && minor != header.minor_version() {
        return Err(Error::InvalidConfig(
            format!("Minor version in firmware and config does not match: {} vs. {}",
                    minor, header.minor_version()) ) );
    } else if minor == 0xFF {
        config.images[idx].version.minor = header.minor_version();
    } else if header.minor_version() == 0xFF {
        header.set_minor_version(minor);
    }

    let build = config.images[idx].version.build;
    if build != 0xFF && build != header.build_version() {
        return Err(Error::InvalidConfig(
            format!("Build version in firmware and config does not match: {} vs. {}",
                    build, header.build_version()) ) );
    } else if build == 0xFF {
        config.images[idx].version.build = header.build_version();
    } else if header.build_version() == 0xFF {
        header.set_build_version(build);
    }

    let fw_id = config.images[idx].fw_id;
    if fw_id != 0 && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(
            format!("Firmware ID in firmware and config does not match: {} vs. {}",
                    build, header.fw_id()) ) );
    } else if fw_id == 0 {
        config.images[idx].fw_id = header.fw_id();
    } else if header.fw_id() == 0 {
        header.set_fw_id(fw_id);
    }

    Ok(fw)
}

pub fn load_btl(config: &Config, idx: usize) -> Result<Firmware, Error> {
    let path = Path::new(&config.images[idx].btl_path);
    let fw_config = &config.images[idx];
    Firmware::load_from_file(&path, &fw_config.hex_file_format, &fw_config.device_config, &fw_config.btl_address)
}

