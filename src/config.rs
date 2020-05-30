use serde::{Serialize, Deserialize};

mod default {
    pub fn fw_id() -> u8 { 0 }
    pub fn header_offset() -> u64 { 0 }
    pub fn include_in_script() -> bool { false }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Timings {
    pub data_send: u32,
    pub data_send_done: u32,
    pub leave_btl: u32,
    pub erase_time: u32,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub enum HexFileFormat {
    IntelHex,
    SRecord
}

impl HexFileFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            HexFileFormat::IntelHex => "hex",
            HexFileFormat::SRecord => "s37",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Endianness {
    Big, Little
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub word_addressing: bool,
    pub endianness: Endianness,
    pub page_size: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageVersion {
    pub minor: u8,
    pub build: u8
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FwConfig {
    #[serde(default = "default::fw_id")]
    pub fw_id: u8,
    pub btl_path: String,
    pub app_path: String,
    pub version: ImageVersion,
    pub app_address: AddressRange,
    pub btl_address: AddressRange,
    #[serde(default = "default::include_in_script")]
    pub include_in_script: bool,
    #[serde(default = "default::header_offset")]
    pub header_offset: u64,
    pub hex_file_format: HexFileFormat,
    pub device_config: DeviceConfig,
    pub timings: Timings,
}

impl FwConfig {
    pub fn designator(&self) -> String {
        format!("F{}", self.fw_id)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub product_id: u32,
    pub product_name: String,
    pub major_version: u8,
    pub btl_version: u8,
    pub use_backdoor: bool,
    pub images: Vec<FwConfig>,
    pub time_state_transition: u32,
}

