use crate::config::{Config, FwConfig, EXT_CMD_CODE};
use std::path::Path;
use crate::Error;
use crate::firmware::Firmware;
use crate::crc::crc32;
use crate::xcmd::ExtCmdProtocol;
use crate::protocol::generate_script;
use crate::script::Script;
use std::fs::File;
use std::io::Write;


pub fn merge_firmware(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    let app = load_app(config, fw_config)?;
    let btl = load_btl(config, fw_config)?;
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

pub fn create_script(config: &Config, output_dir: &Path) -> Result<(), Error> {
    let protocol = ExtCmdProtocol::new(EXT_CMD_CODE);
    let fws: Result<Vec<_>, _> = config.images.iter()
        .map(|x| load_app(config, x))
        .collect();
    let fws = fws?;
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

pub fn merge_all(config: &Config) -> Result<Vec<Firmware>, Error> {
    let mut ret = Vec::new();
    for fw_config in &config.images {
        let fw = merge_firmware(config, fw_config)?;
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

pub fn load_app(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    todo!()
}

pub fn load_btl(config: &Config, fw_config: &FwConfig) -> Result<Firmware, Error> {
    todo!()
}


