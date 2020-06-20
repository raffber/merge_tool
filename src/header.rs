use crate::firmware::Firmware;

const PRODUCT_ID_OFFSET: usize = 0;
const FW_ID_OFFSET: usize = 2;
const MAJOR_VERSION_OFFSET: usize = 4;
const MINOR_VERSION_OFFSET: usize = 6;
const BUILD_VERSION_OFFSET: usize = 8;
const LENGTH_OFFSET: usize = 12;

pub struct Header<'a> {
    fw: &'a mut Firmware,
    offset: usize,
}

impl<'a> Header<'a> {
    pub fn new(fw: &'a mut Firmware, offset: u64) -> Self {
        Self {
            fw,
            offset: offset as usize,
        }
    }

    pub fn product_id(&self) -> u16 {
        self.fw.read_u16(self.offset + PRODUCT_ID_OFFSET)
    }

    pub fn set_product_id(&mut self, value: u16) {
        self.fw.write_u16(self.offset + PRODUCT_ID_OFFSET, value);
    }

    pub fn major_version(&self) -> u8 {
        self.fw.data[self.offset + MAJOR_VERSION_OFFSET]
    }

    pub fn set_major_version(&mut self, value: u8) {
        self.fw.data[self.offset + MAJOR_VERSION_OFFSET] = value;
    }

    pub fn minor_version(&self) -> u8 {
        self.fw.data[self.offset + MINOR_VERSION_OFFSET]
    }

    pub fn set_minor_version(&mut self, value: u8) {
        self.fw.data[self.offset + MINOR_VERSION_OFFSET] = value;
    }

    pub fn build_version(&self) -> u8 {
        self.fw.data[self.offset + BUILD_VERSION_OFFSET]
    }

    pub fn set_build_version(&mut self, value: u8) {
        self.fw.data[self.offset + BUILD_VERSION_OFFSET] = value;
    }

    pub fn fw_id(&self) -> u8 {
        self.fw.data[self.offset + FW_ID_OFFSET]
    }

    pub fn set_fw_id(&mut self, value: u8) {
        self.fw.data[self.offset + FW_ID_OFFSET] = value;
    }

    pub fn length(&self) -> u32 {
        self.fw.read_u32(self.offset + LENGTH_OFFSET)
    }

    pub fn set_length(&mut self, value: u32) {
        self.fw.write_u32(self.offset + LENGTH_OFFSET, value);
    }
}
