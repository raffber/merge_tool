use serde::{Serialize, Deserialize};

mod default {
    pub fn fw_id() -> u8 { 0 }
    pub fn header_offset() -> u64 { 0 }
    pub fn include_in_script() -> bool { false }
}


#[derive(Clone, Serialize, Deserialize)]
struct Timings {
    pub data_send: u32,
    pub data_send_done: u32,
    pub leave_btl: u32,
    pub erase_time: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct AddressRange {
    pub begin: u64,
    pub end: u64,
}

#[derive(Clone, Serialize, Deserialize)]
enum HexFileFormat {
    IntelHex,
    SRecord
}

#[derive(Clone, Serialize, Deserialize)]
enum Endianness {
    Big, Little
}

#[derive(Clone, Serialize, Deserialize)]
struct DeviceConfig {
    word_addressing: bool,
    endianness: Endianness
}

#[derive(Clone, Serialize, Deserialize)]
struct FwConfig {
    #[serde(default = "default::fw_id")]
    pub fw_id: u8,
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
    fn designator(&self) -> String {
        format!("F{}", self.fw_id)
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Config {
    pub product_id: u32,
    pub btl_version: u32,
    pub use_backdoor: bool,
    pub images: Vec<FwConfig>,
    pub time_state_transition: u32,
}

