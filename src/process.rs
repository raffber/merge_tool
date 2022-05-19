use std::fs::{canonicalize, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::default;
use crate::config::{Config, FwConfig, DDP_CMD_CODE};
use crate::crc::crc32;
use crate::ddp::DdpProtocol;
use crate::firmware::Firmware;
use crate::header::Header;
use crate::protocol::generate_script;
use crate::script::Script;
use crate::Error;

use crate::blocking_ddp::BlockingDdpProtocol;
use serde::{Deserialize, Serialize};

pub fn merge_firmware(
    config: &mut Config,
    idx: usize,
    config_dir: &Path,
) -> Result<Firmware, Error> {
    let app = load_app(config, idx, config_dir)?;
    let btl = load_btl(config, idx, config_dir)?;
    Firmware::merge(btl, app)
}

fn generate_script_filename(config: &Config) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "{}_{}",
        config.product_name.clone(),
        config.major_version
    ));
    for fw_config in &config.images {
        parts.push(format!(
            "_{}.{}",
            fw_config.version.minor, fw_config.version.build
        ));
    }
    parts.push(".gctbtl".to_string());
    parts.join("")
}

pub fn create_script(
    config: &mut Config,
    config_dir: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Error> {
    config.transform_to_byte_addrs();
    let mut fws = Vec::new();
    for idx in 0..config.images.len() {
        fws.push(load_app(config, idx, config_dir)?);
    }

    let cmds = if !config.blocking {
        let protocol = DdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, &fws, config)
    } else {
        let protocol = BlockingDdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, &fws, config)
    };

    let filename = generate_script_filename(config);
    let script = Script::new(cmds);
    let path = output_dir.join(&filename);
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(script.serialize().as_bytes())
        .map_err(Error::Io)?;
    Ok(path)
}

pub fn merge_all(config: &mut Config, config_dir: &Path) -> Result<Vec<Firmware>, Error> {
    let mut ret = Vec::new();
    config.transform_to_byte_addrs();
    for idx in 0..config.images.len() {
        let fw = merge_firmware(config, idx, config_dir)?;
        ret.push(fw);
    }
    Ok(ret)
}

fn get_fw_hex_filename(fw_config: &FwConfig) -> String {
    format!(
        "{}.{}",
        fw_config.designator(),
        fw_config.hex_file_format.file_extension()
    )
}

pub fn write_fws(
    config: &Config,
    fws: &[Firmware],
    target_folder: &Path,
) -> Result<Vec<PathBuf>, Error> {
    let mut ret = Vec::new();
    for (fw, fw_config) in fws.iter().zip(config.images.iter()) {
        let file_name = get_fw_hex_filename(fw_config);
        let fpath = target_folder.join(file_name);
        fw.write_to_file(&fpath, &fw_config.hex_file_format)?;
        ret.push(fpath);
    }
    Ok(ret)
}

fn configure_header(mut fw: Firmware, config: &mut Config, idx: usize) -> Result<Firmware, Error> {
    let image_length = fw.image_length();
    let mut header = Header::new(&mut fw, config.images[idx].header_offset)?;
    if config.product_id != default::product_id() && config.product_id != header.product_id() {
        return Err(Error::InvalidConfig(format!(
            "Product ID in firmware and config does not match: {} vs. {}",
            config.product_id,
            header.product_id()
        )));
    } else if config.product_id == default::product_id() {
        config.product_id = header.product_id();
    }
    if config.major_version != default::major_version()
        && config.major_version != header.major_version()
    {
        return Err(Error::InvalidConfig(format!(
            "Major version in config ({}) and firmware ({}) does not match",
            config.major_version,
            header.major_version()
        )));
    } else if config.major_version == default::major_version() {
        config.major_version = header.major_version();
    } else if header.major_version() == default::major_version() {
        header.set_major_version(config.major_version);
    }

    let minor = config.images[idx].version.minor;
    if minor != default::minor_version() && minor != header.minor_version() {
        return Err(Error::InvalidConfig(format!(
            "Minor version in firmware and config does not match: {} vs. {}",
            minor,
            header.minor_version()
        )));
    } else if minor == default::minor_version() {
        config.images[idx].version.minor = header.minor_version();
    } else if header.minor_version() == default::minor_version() {
        header.set_minor_version(minor);
    }

    let build = config.images[idx].version.build;
    if build != default::build_version() && build != header.build_version() {
        return Err(Error::InvalidConfig(format!(
            "Build version in firmware and config does not match: {} vs. {}",
            build,
            header.build_version()
        )));
    } else if build == default::build_version() {
        config.images[idx].version.build = header.build_version();
    } else if header.build_version() == default::build_version() {
        header.set_build_version(build);
    }

    let fw_id = config.images[idx].fw_id;
    if fw_id != default::fw_id() && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(format!(
            "Firmware ID in firmware and config does not match: {} vs. {}",
            build,
            header.fw_id()
        )));
    } else if fw_id == default::fw_id() {
        config.images[idx].fw_id = header.fw_id();
    } else if header.fw_id() == default::fw_id() {
        header.set_fw_id(fw_id);
    }
    header.set_length(image_length as u32);
    Ok(fw)
}

pub fn load_app(config: &mut Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].app_path, config_dir)?;
    let fw = Firmware::load_from_file(
        &path,
        &config.images[idx].hex_file_format,
        &config.images[idx].device_config,
        &config.images[idx].app_address,
    )?;

    let mut fw = configure_header(fw, config, idx)?;
    let crc = crc32(&fw.data[4..fw.image_length()]);
    fw.write_u32(0, crc);

    Ok(fw)
}

pub fn load_btl(config: &Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].btl_path, config_dir)?;
    let fw_config = &config.images[idx];
    Firmware::load_from_file(
        &path,
        &fw_config.hex_file_format,
        &fw_config.device_config,
        &fw_config.btl_address,
    )
}

#[derive(Clone, Serialize, Deserialize)]
struct FwInfo {
    fw_id: u8,
    minor: u16,
    build: u32,
    crc: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Info {
    product_id: u16,
    project_name: String,
    major_version: u16,
    images: Vec<FwInfo>,
    files: Vec<String>,
}

pub fn info(config: &Config, config_dir: &Path, output_dir: &Path) -> Result<PathBuf, Error> {
    let mut config = config.clone();
    config.transform_to_byte_addrs();

    let output_dir = canonicalize(output_dir)?;

    let mut files = Vec::new();

    let mut fws = Vec::new();
    for idx in 0..config.images.len() {
        // will modify config
        let fw = load_app(&mut config, idx, config_dir)?;

        let fw_cfg = &config.images[idx];
        let fw_info = FwInfo {
            fw_id: fw_cfg.fw_id,
            minor: fw_cfg.version.minor,
            build: fw_cfg.version.build,
            crc: fw.read_u32(0),
        };
        fws.push(fw_info);

        // add generated files, for possible archival
        let fw_file = output_dir.join(get_fw_hex_filename(fw_cfg));
        files.push(fw_file.to_str().unwrap().to_string());
        let btl_path = Config::normalize_path(&fw_cfg.btl_path, config_dir)?;
        let app_path = Config::normalize_path(&fw_cfg.app_path, config_dir)?;
        files.push(btl_path.to_str().unwrap().to_string());
        files.push(app_path.to_str().unwrap().to_string());
    }

    let script_file = output_dir.join(generate_script_filename(&config));
    files.push(script_file.to_str().unwrap().to_string());

    let info = Info {
        product_id: config.product_id,
        project_name: config.product_name,
        major_version: config.major_version,
        images: fws,
        files,
    };

    let data = serde_json::to_string_pretty(&info).unwrap();

    let path = output_dir.join("info.json");
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)?;
    Ok(path)
}
