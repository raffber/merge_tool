use crate::Error;
use serde::{Deserialize, Serialize};
use std::fs::{canonicalize, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use regex::Regex;

pub const EXT_CMD_CODE: u8 = 0x11;

mod default {
    pub fn fw_id() -> u8 {
        0
    }
    pub fn major_version() -> u8 {
        0xFF
    }
    pub fn minor_version() -> u8 {
        0xFF
    }
    pub fn build_version() -> u8 {
        0xFF
    }
    pub fn header_offset() -> u64 {
        4
    }
    pub fn include_in_script() -> bool {
        false
    }
    pub fn btl_version() -> u8 {
        1
    }
    pub fn empty_string() -> String {
        "".to_string()
    }
    pub fn use_backdoor() -> bool {
        false
    }
}

fn skip_if_ff(value: &u8) -> bool {
    *value == 0xFF
}

fn skip_if_zero(value: &u8) -> bool {
    *value == 0
}

fn skip_if_one(value: &u8) -> bool {
    *value == 1
}

fn skip_if_false(value: &bool) -> bool {
    !*value
}

fn skip_if_version(value: &ImageVersion) -> bool {
    value.build == 0xFF && value.minor == 0xFF
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Timings {
    pub data_send: u32,
    pub data_send_done: u32,
    pub leave_btl: u32,
    pub erase_time: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AddressRange {
    pub begin: u64,
    pub end: u64,
}

impl AddressRange {
    pub fn new(begin: u64, end: u64) -> Self {
        Self { begin, end }
    }
    pub fn len(&self) -> u64 {
        self.end - self.begin + 1
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum HexFileFormat {
    IntelHex,
    SRecord,
}

impl Default for HexFileFormat {
    fn default() -> Self {
        HexFileFormat::IntelHex
    }
}

impl HexFileFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            HexFileFormat::IntelHex => "hex",
            HexFileFormat::SRecord => "s37",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Endianness {
    Big,
    Little,
}

impl Default for Endianness {
    fn default() -> Self {
        Endianness::Little
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct DeviceConfig {
    pub word_addressing: bool,
    pub endianness: Endianness,
    pub page_size: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageVersion {
    #[serde(default = "default::minor_version", skip_serializing_if = "skip_if_ff")]
    pub minor: u8,
    #[serde(default = "default::build_version", skip_serializing_if = "skip_if_ff")]
    pub build: u8,
}

impl Default for ImageVersion {
    fn default() -> Self {
        Self {
            minor: 0xFF,
            build: 0xFF,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FwConfig {
    #[serde(default = "default::fw_id", skip_serializing_if = "skip_if_zero")]
    pub fw_id: u8,
    pub btl_path: String,
    pub app_path: String,
    #[serde(default = "Default::default", skip_serializing_if = "skip_if_version")]
    pub version: ImageVersion,
    pub app_address: AddressRange,
    pub btl_address: AddressRange,
    #[serde(default = "default::include_in_script")]
    pub include_in_script: bool,
    pub header_offset: u64,
    pub hex_file_format: HexFileFormat,
    pub device_config: DeviceConfig,
    pub timings: Timings,
}

impl Default for FwConfig {
    fn default() -> Self {
        FwConfig {
            fw_id: default::fw_id(),
            btl_path: "".to_string(),
            app_path: "".to_string(),
            version: Default::default(),
            app_address: Default::default(),
            btl_address: Default::default(),
            include_in_script: true,
            header_offset: default::header_offset(),
            hex_file_format: Default::default(),
            device_config: Default::default(),
            timings: Default::default(),
        }
    }
}

impl FwConfig {
    pub fn designator(&self) -> String {
        format!("F{}", self.fw_id)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub product_id: u16,
    pub product_name: String,
    #[serde(default = "default::major_version", skip_serializing_if = "skip_if_ff")]
    pub major_version: u8,
    #[serde(default = "default::btl_version", skip_serializing_if = "skip_if_one")]
    pub btl_version: u8,
    #[serde(
        default = "default::use_backdoor",
        skip_serializing_if = "skip_if_false"
    )]
    pub use_backdoor: bool,
    pub images: Vec<FwConfig>,
    pub time_state_transition: u32,
    #[serde(
        default = "default::empty_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub repo_path: String,
}

impl Config {
    pub fn validate_product_name(name: &str) -> Result<(), Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^\w+[\w-]*\w+$").unwrap();
        }
        if !RE.is_match(name) {
            Err(Error::InvalidProductName)
        } else {
            Ok(())
        }
    }

    pub fn get_config_dir(config_path: &Path) -> Result<PathBuf, Error> {
        let config_path = canonicalize(config_path).map_err(Error::Io)?;
        Ok(config_path
            .parent()
            .map(|x| x.to_path_buf())
            .unwrap_or(PathBuf::from("/")))
    }

    pub fn normalize_path(path: &str, config_dir: &Path) -> Result<PathBuf, Error> {
        let mut path = PathBuf::from(path);
        if path.is_relative() {
            path = canonicalize(config_dir.join(&path)).map_err(Error::Io)?;
        }
        Ok(path)
    }

    pub fn load_from_file(path: &Path) -> Result<Config, Error> {
        let mut data = String::new();
        File::open(path)
            .map_err(Error::Io)?
            .read_to_string(&mut data)
            .map_err(Error::Io)?;
        Self::load_from_string(&data)
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create(path).map_err(Error::Io)?;
        file.write_all(data.as_bytes()).map_err(Error::Io)
    }

    pub fn get_repo_path(&self, config_dir: &Path) -> Result<PathBuf, Error> {
        let repo_path = self.repo_path.trim();
        if repo_path.is_empty() {
            let mut path = config_dir;
            let git_path = path.join(".git");
            if git_path.exists() {
                return Ok(git_path);
            }
            while let Some(parent) = path.parent() {
                let git_path = parent.join(".git");
                if git_path.exists() {
                    return Ok(git_path);
                }
                path = parent;
            }
            Err(Error::CannotFindGitRepo)
        } else {
            Ok(repo_path.into())
        }
    }

    pub fn load_from_string(data: &str) -> Result<Config, Error> {
        let config: Config = serde_json::from_str(data).map_err(Error::CannotParseConfig)?;
        Self::validate_product_name(&config.product_name)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            product_id: 0,
            product_name: "".to_string(),
            major_version: 0xFF,
            btl_version: 1,
            use_backdoor: false,
            images: vec![],
            time_state_transition: 0,
            repo_path: "".to_string(),
        }
    }
}
