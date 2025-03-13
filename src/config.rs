use crate::Error;
use chrono::{DateTime, Utc};
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs::{self, canonicalize, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const DDP_CMD_CODE: u8 = 0x10;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default::product_id")]
    pub product_id: u16,
    pub product_name: String,
    #[serde(default = "default::btl_version")]
    pub btl_version: u8,
    #[serde(default = "default::use_backdoor")]
    pub use_backdoor: bool,
    #[serde(default = "default::blocking")]
    pub blocking: bool,
    pub images: Vec<FwConfig>,
    #[serde(default = "default::zero_u32")]
    pub time_state_transition: u32,

    #[serde(skip)]
    byte_addresses: bool,

    #[serde(default = "default::default_time")]
    pub build_time: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FwConfig {
    #[serde(default = "default::node_id")]
    pub node_id: u8,
    #[serde(default = "Default::default", skip_serializing_if = "Option::is_none")]
    pub version: Option<Version>,
    pub btl_path: String,
    pub app_path: String,
    pub app_address: AddressRange,
    pub btl_address: AddressRange,
    #[serde(default = "default::write_data_size")]
    pub write_data_size: usize,
    #[serde(default = "default::include_in_script")]
    pub include_in_script: bool,
    #[serde(default = "default::header_offset")]
    pub header_offset: u64,
    pub hex_file_format: HexFileFormat,
    pub device_config: DeviceConfig,
    #[serde(default = "Default::default")]
    pub timings: Timings,
}

impl Default for FwConfig {
    fn default() -> Self {
        FwConfig {
            node_id: 0,
            version: None,
            btl_path: Default::default(),
            app_path: Default::default(),
            app_address: Default::default(),
            btl_address: Default::default(),
            write_data_size: default::write_data_size(),
            include_in_script: false,
            header_offset: default::header_offset(),
            hex_file_format: HexFileFormat::default(),
            device_config: DeviceConfig::default(),
            timings: Timings::default(),
        }
    }
}

impl FwConfig {
    pub fn designator(&self) -> String {
        format!("f{}", self.node_id)
    }
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
        let config_path = fs::canonicalize(config_path)?;
        let path = config_path
            .parent()
            .map(|x| x.to_path_buf())
            .unwrap_or(PathBuf::from("/"));

        Ok(fs::canonicalize(path)?)
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
        let mut to_serialize = self.clone();
        to_serialize.transform_to_word_addrs();
        let data = serde_json::to_string_pretty(&to_serialize).unwrap();
        let mut file = File::create(path).map_err(Error::Io)?;
        file.write_all(data.as_bytes()).map_err(Error::Io)
    }

    pub fn load_from_string(data: &str) -> Result<Config, Error> {
        let config: Config = serde_json::from_str(data).map_err(Error::CannotParseConfig)?;
        Self::validate_product_name(&config.product_name)?;
        Ok(config)
    }

    pub fn transform_to_byte_addrs(&mut self) {
        if self.byte_addresses {
            return;
        }
        for fwconfig in &mut self.images {
            if fwconfig.device_config.word_addressing {
                fwconfig.app_address.begin *= 2;
                fwconfig.app_address.end = 2 * fwconfig.app_address.end;
                fwconfig.btl_address.begin *= 2;
                fwconfig.btl_address.end = 2 * fwconfig.btl_address.end;
                fwconfig.header_offset *= 2;
                fwconfig.device_config.page_size *= 2;
            }
        }
        self.byte_addresses = true;
    }

    pub fn transform_to_word_addrs(&mut self) {
        if !self.byte_addresses {
            return;
        }
        for fwconfig in &mut self.images {
            if fwconfig.device_config.word_addressing {
                fwconfig.app_address.begin /= 2;
                fwconfig.app_address.end /= 2;
                fwconfig.btl_address.begin /= 2;
                fwconfig.btl_address.end /= 2;
                fwconfig.header_offset /= 2;
                fwconfig.device_config.page_size /= 2;
            }
        }
        self.byte_addresses = true;
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            product_id: 0,
            product_name: "".to_string(),
            btl_version: 1,
            use_backdoor: false,
            blocking: false,
            images: vec![],
            time_state_transition: 0,
            byte_addresses: false,
            build_time: default::default_time(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Timings {
    pub data_send: u32,
    pub crc_check: u32,
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
        self.end - self.begin
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceConfig {
    #[serde(default = "default::get_false")]
    pub word_addressing: bool,
    #[serde(default = "default::endianness")]
    pub endianness: Endianness,
    pub page_size: u64,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig {
            word_addressing: false,
            endianness: default::endianness(),
            page_size: 64,
        }
    }
}

impl DeviceConfig {
    pub fn byte_address_multiplier(&self) -> u64 {
        if self.word_addressing {
            2
        } else {
            1
        }
    }
}

pub mod default {
    use chrono::{DateTime, Utc};

    use super::Endianness;

    pub fn node_id() -> u8 {
        0
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
    pub fn product_id() -> u16 {
        0
    }
    pub fn blocking() -> bool {
        true
    }

    pub fn write_data_size() -> usize {
        16
    }

    pub fn zero_u32() -> u32 {
        0
    }

    pub fn get_false() -> bool {
        false
    }

    pub fn endianness() -> Endianness {
        Endianness::Little
    }

    pub fn default_time() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(0, 0).unwrap()
    }
}
