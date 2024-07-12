use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::app_package::{self, AppPackage};
use crate::config::{Config, FwConfig, HexFileFormat, DDP_CMD_CODE};
use crate::crc::crc32;
use crate::ddp::DdpProtocol;
use crate::firmware::Firmware;
use crate::git_description::{retrieve_description, GitDescription};
use crate::header::Header;
use crate::protocol::generate_script;
use crate::script::Script;
use crate::Error;

use crate::blocking_ddp::BlockingDdpProtocol;
use chrono::Utc;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

pub struct GenerateOptions {
    pub config: Config,
    pub output_dir: PathBuf,
    pub config_dir: PathBuf,
    pub repo_dir: Option<PathBuf>,
}

pub fn generate(options: GenerateOptions) -> Result<(), Error> {
    create_dir_all(&options.output_dir)?;

    let loaded = load_firmware_images(
        &options.config,
        &options.config_dir,
        options.repo_dir.as_ref().map(|x| x.as_path()),
    )?;

    // create script
    let script = create_script(&loaded)?;
    save_script(&script, &loaded, &options.config_dir)?;

    // merge firmware images
    let merged = merge_all(&loaded)?;
    save_merged_firmware_images(&merged, &options.output_dir)?;

    // generate info.json
    let info = generate_info(&loaded, &options.output_dir)?;
    save_info(&info, &options.output_dir)?;

    // generate app package
    let package = AppPackage::from_loaded_firmware_images(loaded.config.product_id, &loaded);
    save_app_package(&package, &options.output_dir, &loaded.app_package_file_name)?;

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
    pub app_package_file_name: String,
}

pub fn load_firmware_images(
    config: &Config,
    config_dir: &Path,
    git_repo: Option<&Path>,
) -> Result<LoadedFirmwareImages, Error> {
    let mut config = config.clone();

    if config.build_time.timestamp() == 0 {
        config.build_time = chrono::Utc::now();
    }

    let mut git_description = None;
    if let Some(repo) = git_repo {
        git_description = Some(retrieve_description(repo)?);
    }

    let mut ret = Vec::new();
    config.transform_to_byte_addrs();
    for idx in 0..config.images.len() {
        let app = load_app(&mut config, idx, config_dir)?;
        let btl = load_btl(&mut config, idx, config_dir)?;

        if let Some(desc) = git_description.as_ref() {
            add_pre_release_info(
                &mut config.images[idx].version.as_mut().unwrap(),
                &config.build_time,
                desc,
            );
        }

        let fw_config = &config.images[idx];
        let merged_hex_file_name = format!(
            "merged_f{}.{}",
            fw_config.node_id,
            fw_config.hex_file_format.file_extension(),
        );

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
        script_file_name: format!("{}.gctbtl", config.product_name),
        app_package_file_name: format!("app_pkg.{}", app_package::BINARY_FILE_EXTENSION),
    })
}

pub fn load_app(config: &mut Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].app_path, config_dir)?;
    config.images[idx].app_path = path.to_str().unwrap().to_string();
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

pub fn load_btl(config: &mut Config, idx: usize, config_dir: &Path) -> Result<Firmware, Error> {
    let path = Config::normalize_path(&config.images[idx].btl_path, config_dir)?;
    config.images[idx].btl_path = path.to_str().unwrap().to_string();
    let fw_config = &config.images[idx];
    Firmware::load_from_file(
        &path,
        &fw_config.hex_file_format,
        &fw_config.device_config,
        &fw_config.btl_address,
    )
}

fn configure_header(mut fw: Firmware, config: &mut Config, idx: usize) -> Result<Firmware, Error> {
    let default_config = Config::default();
    let default_fw_config = FwConfig::default();

    let image_length = fw.image_length();
    let mut header = Header::new(&mut fw, config.images[idx].header_offset)?;
    if config.product_id != default_config.product_id && config.product_id != header.product_id() {
        return Err(Error::InvalidConfig(format!(
            "Product ID in firmware and config does not match: {} vs. {}",
            config.product_id,
            header.product_id()
        )));
    } else if config.product_id == default_config.product_id {
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
    if fw_id != default_fw_config.node_id && fw_id != header.fw_id() {
        return Err(Error::InvalidConfig(format!(
            "Firmware ID in firmware and config does not match: {} vs. {}",
            fw_id,
            header.fw_id()
        )));
    } else if fw_id == default_fw_config.node_id {
        config.images[idx].node_id = header.fw_id();
    } else if header.fw_id() == default_fw_config.node_id {
        header.set_fw_id(fw_id);
    }
    header.set_length(image_length as u32);

    header.set_timestamp(config.build_time.timestamp() as u64);

    Ok(fw)
}

pub fn create_script(loaded: &LoadedFirmwareImages) -> Result<Script, crate::Error> {
    let cmds = if !loaded.config.blocking {
        let protocol = DdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, loaded)?
    } else {
        let protocol = BlockingDdpProtocol::new(DDP_CMD_CODE);
        generate_script(&protocol, loaded)?
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
        let merged = Firmware::concatenate(&fw.btl, &fw.app)?;
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
    product_name: String,
    images: Vec<FwInfo>,
    files: Vec<String>,
    script_file: String,
    package_file: String,
    output_dir: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct FwInfo {
    fw_id: u8,
    version: Version,
    crc: u32,
    hex_file_format: HexFileFormat,
    merged_file: String,
    app_file: String,
    btl_file: String,
}

pub fn generate_info(fws: &LoadedFirmwareImages, output_dir: &Path) -> Result<Info, Error> {
    let mut files = Vec::new();
    let mut fw_infos = Vec::new();

    for fw in &fws.images {
        let btl_path = normalize_file_path(&fw.config.btl_path, output_dir);
        let app_path = normalize_file_path(&fw.config.app_path, output_dir);

        let fw_info = FwInfo {
            fw_id: fw.config.node_id,
            version: fw.config.version.clone().unwrap(),
            crc: fw.app.read_u32(0),
            merged_file: fw.merged_hex_file_name.clone(),
            app_file: app_path.clone(),
            btl_file: btl_path.clone(),
            hex_file_format: fw.config.hex_file_format,
        };

        fw_infos.push(fw_info);

        // add generated files, for possible archival
        let merged = fw.merged_hex_file_name.clone();
        files.push(merged);
        files.push(btl_path);
        files.push(app_path);
    }

    files.push(fws.script_file_name.clone());

    let info = Info {
        product_id: fws.config.product_id,
        product_name: fws.config.product_name.clone(),
        images: fw_infos,
        script_file: fws.script_file_name.clone(),
        files,
        output_dir: output_dir.to_str().unwrap().to_string(),
        package_file: fws.app_package_file_name.clone(),
    };

    Ok(info)
}

fn normalize_file_path(path: &str, output_dir: &Path) -> String {
    let path = Path::new(path);
    assert!(path.is_absolute());
    assert!(output_dir.is_absolute());

    if let Some(ret) = pathdiff::diff_paths(path, output_dir) {
        ret.to_str().unwrap().to_string()
    } else {
        path.to_str().unwrap().to_string()
    }
}

pub fn save_info(info: &Info, output_dir: &Path) -> Result<(), Error> {
    let data = serde_json::to_string_pretty(&info).unwrap();

    let path = output_dir.join("info.json");
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)?;
    Ok(())
}

pub fn add_pre_release_info(
    version: &mut Version,
    date_time: &chrono::DateTime<Utc>,
    description: &GitDescription,
) {
    if description.is_pre_release() {
        let date_time = date_time.format("%Y%m%d%H%M%S");
        let pre_release = format!("pre.{}", date_time);
        version.pre = Prerelease::new(&pre_release).unwrap();
        version.build = BuildMetadata::new(&description.sha).unwrap();
    }
}

pub fn bundle(info: &Path, output_dir: &Path, versioned: bool) -> Result<(), crate::Error> {
    let info = info.canonicalize()?;
    let info_data = fs::read_to_string(&info)?;
    let info_dir = info.parent().unwrap_or(Path::new("/"));

    let info: Info =
        serde_json::from_str(&info_data).map_err(|x| crate::Error::InvalidInfoFile(x.into()))?;

    let mut new_info: Info = info.clone();
    new_info.files = Vec::new();

    create_dir_all(output_dir)?;

    for (fw, fw_new) in info.images.iter().zip(new_info.images.iter_mut()) {
        fw_new.app_file = get_app_file_name(&fw, versioned);
        fw_new.btl_file = get_btl_file_name(&fw, versioned);
        fw_new.merged_file = get_merged_file_name(&fw, versioned);
        copy_and_rename(&info_dir.join(&fw.app_file), output_dir, &fw_new.app_file)?;
        copy_and_rename(&info_dir.join(&fw.btl_file), output_dir, &fw_new.btl_file)?;
        copy_and_rename(
            &info_dir.join(&fw.merged_file),
            output_dir,
            &fw_new.merged_file,
        )?;
        new_info.files.push(fw_new.app_file.clone());
        new_info.files.push(fw_new.btl_file.clone());
        new_info.files.push(fw_new.merged_file.clone());
    }

    new_info.script_file = get_script_file_name(&info, versioned);
    copy_and_rename(
        &info_dir.join(&info.script_file),
        output_dir,
        &new_info.script_file,
    )?;
    new_info.files.push(new_info.script_file.clone());

    new_info.package_file = get_app_package_file_name(&info, versioned);
    copy_and_rename(
        &info_dir.join(&info.package_file),
        output_dir,
        &new_info.package_file,
    )?;
    new_info.files.push(new_info.package_file.clone());

    let new_info_data = serde_json::to_string_pretty(&new_info).unwrap();
    let new_info_path = output_dir.join("info.json");
    let mut file = File::create(&new_info_path)?;
    file.write_all(new_info_data.as_bytes())?;

    Ok(())
}

pub fn merge_app_packages(files: &[&Path], output_file: &Path) -> crate::Result<()> {
    let mut packages = Vec::new();
    for fpath in files {
        let package = app_package::AppPackage::load_from_file(fpath)?;
        packages.extend(package.app);
    }

    let merged = app_package::AppPackage::new(packages);

    let data = merged.to_cbor();
    let mut file = File::create(output_file)?;
    file.write_all(&data)?;
    file.flush()?;

    Ok(())
}

pub fn save_app_package(
    info: &AppPackage,
    output_dir: &Path,
    file_name: &str,
) -> Result<(), Error> {
    let data = info.to_cbor();
    let path = output_dir.join(file_name);
    let mut file = File::create(&path).map_err(Error::Io)?;
    file.write_all(&data).map_err(Error::Io)?;
    Ok(())
}

fn copy_and_rename(src: &Path, dest_dir: &Path, new_name: &str) -> Result<(), crate::Error> {
    Ok(fs::copy(src, dest_dir.join(new_name)).map(|_| ())?)
}

fn get_app_file_name(fw: &FwInfo, versioned: bool) -> String {
    if versioned {
        format!(
            "app_f{}_{}.{}",
            fw.fw_id,
            fw.version,
            fw.hex_file_format.file_extension()
        )
    } else {
        format!("app_f{}.{}", fw.fw_id, fw.hex_file_format.file_extension())
    }
}

fn get_btl_file_name(fw: &FwInfo, versioned: bool) -> String {
    if versioned {
        format!(
            "btl_f{}_{}.{}",
            fw.fw_id,
            fw.version,
            fw.hex_file_format.file_extension()
        )
    } else {
        format!("btl_f{}.{}", fw.fw_id, fw.hex_file_format.file_extension())
    }
}

fn get_merged_file_name(fw: &FwInfo, versioned: bool) -> String {
    if versioned {
        format!(
            "merged_f{}_{}.{}",
            fw.fw_id,
            fw.version,
            fw.hex_file_format.file_extension()
        )
    } else {
        format!(
            "merged_f{}.{}",
            fw.fw_id,
            fw.hex_file_format.file_extension()
        )
    }
}

fn get_script_file_name(info: &Info, versioned: bool) -> String {
    if !versioned {
        return format!("{}.gctbtl", info.product_name);
    }
    let mut parts = Vec::new();
    parts.push(format!("{}", info.product_name.clone()));
    for fw_info in &info.images {
        parts.push(format!("_{}", fw_info.version));
    }
    parts.push(".gctbtl".to_string());
    parts.join("")
}

fn get_app_package_file_name(info: &Info, versioned: bool) -> String {
    if !versioned {
        return format!("app_pkg.{}", app_package::BINARY_FILE_EXTENSION);
    }
    let mut parts = vec!["app_pkg".to_string()];
    for fw_info in &info.images {
        parts.push(format!("_{}", fw_info.version));
    }
    parts.push(format!(".{}", app_package::BINARY_FILE_EXTENSION));
    parts.join("")
}
