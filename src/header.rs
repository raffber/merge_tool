use crate::firmware::Firmware;
use crate::Error;

const PRODUCT_ID_OFFSET: usize = 0;
const FW_ID_OFFSET: usize = 2;
const MAJOR_VERSION_OFFSET: usize = 4;
const MINOR_VERSION_OFFSET: usize = 6;
const PATCH_VERSION_OFFSET: usize = 8;
const LENGTH_OFFSET: usize = 12;
const TIMESTAMP_OFFSET: usize = 18;

const HEADER_LENGTH: usize = 32;

pub struct Header<'a> {
    fw: &'a mut Firmware,
    offset: usize,
}

impl<'a> Header<'a> {
    pub fn new(fw: &'a mut Firmware, offset: u64) -> Result<Self, Error> {
        if fw.data.len() < offset as usize + HEADER_LENGTH {
            Err(Error::ImageTooShortForHeader)
        } else {
            Ok(Self {
                fw,
                offset: offset as usize,
            })
        }
    }

    pub fn product_id(&self) -> u16 {
        self.fw.read_u16(self.offset + PRODUCT_ID_OFFSET)
    }

    pub fn set_product_id(&mut self, value: u16) {
        self.fw.write_u16(self.offset + PRODUCT_ID_OFFSET, value);
    }

    pub fn major_version(&self) -> u16 {
        self.fw.read_u16(self.offset + MAJOR_VERSION_OFFSET)
    }

    pub fn set_major_version(&mut self, value: u16) {
        self.fw.write_u16(self.offset + MAJOR_VERSION_OFFSET, value);
    }

    pub fn minor_version(&self) -> u16 {
        self.fw.read_u16(self.offset + MINOR_VERSION_OFFSET)
    }

    pub fn set_minor_version(&mut self, value: u16) {
        self.fw.write_u16(self.offset + MINOR_VERSION_OFFSET, value);
    }

    pub fn patch_version(&self) -> u32 {
        self.fw.read_u32(self.offset + PATCH_VERSION_OFFSET)
    }

    pub fn set_patch_version(&mut self, value: u32) {
        self.fw.write_u32(self.offset + PATCH_VERSION_OFFSET, value);
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

    pub fn set_timestamp(&mut self, value: u64) {
        self.fw
            .write_u32(self.offset + TIMESTAMP_OFFSET, (value & 0xFFFFFFFF) as u32);
        self.fw.write_u16(
            self.offset + TIMESTAMP_OFFSET + 4,
            ((value >> 32) & 0xFFFF) as u16,
        );
    }

    pub fn get_timestamp(&self) -> u64 {
        let low = self.fw.read_u32(self.offset + TIMESTAMP_OFFSET) as u64;
        let high = self.fw.read_u16(self.offset + TIMESTAMP_OFFSET + 4) as u64;
        low | (high << 32)
    }
}

#[cfg(test)]
mod test {
    use crate::config::{AddressRange, DeviceConfig};

    #[test]
    fn test_timestamp() {
        use crate::firmware::Firmware;
        use crate::header::Header;

        let mut fw = Firmware::new(
            AddressRange::new(0, 128),
            DeviceConfig::default(),
            (0..128).collect(),
        )
        .unwrap();

        let mut header = Header::new(&mut fw, 0).unwrap();
        header.set_timestamp(0x000056789ABCDEF0);
        assert_eq!(header.get_timestamp(), 0x000056789ABCDEF0);
    }
}
