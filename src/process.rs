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
use gix::commit::describe::SelectRef;
use semver::Version;
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
    parts.push(format!("{}", config.product_name.clone()));
    for fw_config in &config.images {
        let version = fw_config.version.clone().unwrap_or(Version::new(0, 0, 0));
        parts.push(format!("_{}", version));
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
        generate_script(&protocol, &fws, config)?
    } else {
        let protocol = BlockingDdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, &fws, config)?
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
    version: Version,
    crc: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Info {
    product_id: u16,
    project_name: String,
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
        let version = fw_cfg.version.clone().unwrap_or(Version::new(0, 0, 0));

        let repo = gix::discover(Path::new(".")).unwrap();
        let mut head = repo.head().unwrap();
        let head_commit = head.peel_to_commit_in_place().unwrap();
        let describe = head_commit.describe().names(SelectRef::AllTags);
        let resolution = describe.try_resolve().unwrap().unwrap();
        let tag_name = resolution.outcome.name.unwrap();
        println!("HEAD: {}", tag_name);
        let ref_name = format!("tags/{}", tag_name);
        let mut tag_ref = repo.find_reference(&ref_name).unwrap();
        let tag_id = tag_ref.peel_to_id_in_place().unwrap();
        println!("id: {}", tag_id);

        // let head_id = head.id().unwrap();

        // let graph = repo.commit_graph().unwrap();

        // git_describe(&head_id, &mut graph, DescribeOptions::default()).unwrap();

        let fw_info = FwInfo {
            fw_id: fw_cfg.node_id,
            version,
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
        images: fws,
        files,
    };

    let data = serde_json::to_string_pretty(&info).unwrap();

    let path = output_dir.join("info.json");
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)?;
    Ok(path)
}
