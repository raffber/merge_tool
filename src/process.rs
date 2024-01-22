use std::fs::{create_dir_all, File};
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
use semver::Version;
use serde::{Deserialize, Serialize};

pub struct GenerateOptions {
    pub config: Config,
    pub output_dir: PathBuf,
    pub config_dir: PathBuf,
    pub repo_dir: PathBuf,
}

pub fn generate(options: GenerateOptions) -> Result<(), Error> {
    create_dir_all(&options.output_dir)?;

    let loaded = load_firmware_images(&options.config, &options.config_dir)?;

    // create script
    let script = create_script(&loaded)?;
    save_script(&script, &loaded, &options.config_dir)?;

    // merge firmware images
    let merged = merge_all(&loaded)?;
    save_merged_firmware_images(&merged, &options.output_dir)?;

    // generate info.json
    let info = generate_info(&loaded, &options.config_dir, &options.output_dir)?;
    save_info(&info, &options.output_dir)?;

    Ok(())
}

pub struct LoadedFirmware {
    pub btl: Firmware,
    pub app: Firmware,
    pub config: FwConfig,
    pub merged_hex_file_name: String,
}

pub struct LoadedFirmwareImages {
    pub images: Vec<LoadedFirmware>,
    pub config: Config,
    pub script_file_name: String,
}

pub fn load_firmware_images(
    config: &Config,
    config_dir: &Path,
) -> Result<LoadedFirmwareImages, Error> {
    let mut config = config.clone();
    let mut ret = Vec::new();
    config.transform_to_byte_addrs();
    for idx in 0..config.images.len() {
        let app = load_app(&mut config, idx, config_dir)?;
        let btl = load_btl(&mut config, idx, config_dir)?;
        let merged_hex_file_name = get_merged_hex_filename(&config.images[idx]);

        let loaded = LoadedFirmware {
            btl,
            app,
            config: config.images[idx].clone(),
            merged_hex_file_name,
        };

        ret.push(loaded);
    }
    Ok(LoadedFirmwareImages {
        images: ret,
        config: config.clone(),
        script_file_name: generate_script_filename(&config),
    })
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
    let fwconfig = &mut config.images[idx];
    if let Some(version) = fwconfig.version.as_ref() {
        header.set_major_version(version.major as u16);
        header.set_minor_version(version.minor as u16);
        header.set_patch_version(version.patch as u32);
    } else {
        let version = Version::new(
            header.major_version() as u64,
            header.minor_version() as u64,
            header.patch_version() as u64,
        );
        fwconfig.version = Some(version);
    }

    let fw_id = config.images[idx].node_id;
    if fw_id != default::node_id() && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(format!(
            "Firmware ID in firmware and config does not match: {} vs. {}",
            fw_id,
            header.fw_id()
        )));
    } else if fw_id == default::node_id() {
        config.images[idx].node_id = header.fw_id();
    } else if header.fw_id() == default::node_id() {
        header.set_fw_id(fw_id);
    }
    header.set_length(image_length as u32);
    Ok(fw)
}

fn get_merged_hex_filename(fw_config: &FwConfig) -> String {
    format!(
        "merged_{}.{}",
        fw_config.designator(),
        fw_config.hex_file_format.file_extension()
    )
}

fn generate_script_filename(config: &Config) -> String {
    let mut parts = Vec::new();
    parts.push(format!("{}", config.product_name.clone()));
    for fw_config in &config.images {
        let version = fw_config.version.clone().unwrap_or(Version::new(0, 0, 0));
        parts.push(format!("_{}", version));
    }
    parts.push(".gctbtl".to_string());
    parts.join("")
}

pub fn create_script(loaded: &LoadedFirmwareImages) -> Result<Script, crate::Error> {
    let cmds = if !loaded.config.blocking {
        let protocol = DdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, loaded, &loaded.config)?
    } else {
        let protocol = BlockingDdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, loaded, &loaded.config)?
    };
    let script = Script::new(cmds);
    Ok(script)
}

pub fn save_script(
    script: &Script,
    loaded: &LoadedFirmwareImages,
    output_dir: &Path,
) -> Result<(), Error> {
    let path = output_dir.join(&loaded.script_file_name);
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(script.serialize().as_bytes())
        .map_err(Error::Io)?;
    Ok(())
}

pub struct MergedFirmwareImages<'a> {
    pub images: Vec<(Firmware, &'a LoadedFirmware)>,
}

pub fn merge_all<'a>(loaded: &'a LoadedFirmwareImages) -> Result<MergedFirmwareImages<'a>, Error> {
    let mut ret = Vec::new();
    for fw in &loaded.images {
        let merged = Firmware::merge(&fw.btl, &fw.app)?;
        ret.push((merged, fw));
    }
    Ok(MergedFirmwareImages { images: ret })
}

pub fn save_merged_firmware_images(
    merged: &MergedFirmwareImages,
    output_dir: &Path,
) -> Result<(), Error> {
    for fw in &merged.images {
        let fpath = output_dir.join(&fw.1.merged_hex_file_name);
        fw.0.write_to_file(&fpath, &fw.1.config.hex_file_format)
            .unwrap();
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Info {
    product_id: u16,
    project_name: String,
    images: Vec<FwInfo>,
    files: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct FwInfo {
    fw_id: u8,
    version: Version,
    crc: u32,
}

pub fn generate_info(
    fws: &LoadedFirmwareImages,
    config_dir: &Path,
    output_dir: &Path,
) -> Result<Info, Error> {
    let mut files = Vec::new();
    let mut fw_infos = Vec::new();

    for fw in &fws.images {
        let fw_info = FwInfo {
            fw_id: fw.config.node_id,
            version: fw.config.version.clone().unwrap(),
            crc: fw.app.read_u32(0),
        };
        fw_infos.push(fw_info);

        // add generated files, for possible archival
        let fw_file = output_dir.join(&fw.merged_hex_file_name);
        files.push(fw_file.to_str().unwrap().to_string());
        let btl_path = Config::normalize_path(&fw.config.btl_path, config_dir)?;
        let app_path = Config::normalize_path(&fw.config.app_path, config_dir)?;
        files.push(btl_path.to_str().unwrap().to_string());
        files.push(app_path.to_str().unwrap().to_string());
    }

    let script_file = output_dir.join(&fws.script_file_name);
    files.push(script_file.to_str().unwrap().to_string());

    let info = Info {
        product_id: fws.config.product_id,
        project_name: fws.config.product_name.clone(),
        images: fw_infos,
        files,
    };

    Ok(info)
}

pub fn save_info(info: &Info, output_dir: &Path) -> Result<(), Error> {
    let data = serde_json::to_string_pretty(&info).unwrap();

    let path = output_dir.join("info.json");
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)?;
    Ok(())
}
