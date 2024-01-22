use crate::config::{AddressRange, DeviceConfig, Endianness, HexFileFormat};
use crate::{intel_hex, srecord, Error};
use std::iter::repeat;
use std::path::Path;

#[derive(Clone)]
pub struct Firmware {
    pub range: AddressRange,
    pub config: DeviceConfig,
    pub data: Vec<u8>,
}

impl Firmware {
    pub fn new(range: AddressRange, config: DeviceConfig, data: Vec<u8>) -> Result<Self, Error> {
        if range.begin % config.page_size != 0 {
            return Err(Error::AddressRangeNotAlignedToPage);
        }
        if range.end % config.page_size != 0 {
            return Err(Error::AddressRangeNotAlignedToPage);
        }
        if data.len() != range.len() as usize {
            return Err(Error::InvalidDataLength);
        }
        Ok(Self {
            data,
            range,
            config,
        })
    }

    pub fn load_from_file(
        path: &Path,
        file_format: &HexFileFormat,
        config: &DeviceConfig,
        range: &AddressRange,
    ) -> Result<Firmware, Error> {
        let ret = match file_format {
            HexFileFormat::IntelHex => intel_hex::load(path, config.word_addressing, &range),
            HexFileFormat::SRecord => srecord::load(path, config.word_addressing, &range),
        };
        ret.and_then(|data| Firmware::new(range.clone(), config.clone(), data))
    }

    pub fn write_to_file(&self, path: &Path, file_format: &HexFileFormat) -> Result<(), Error> {
        match file_format {
            HexFileFormat::IntelHex => {
                intel_hex::save(path, self.config.word_addressing, &self.range, &self.data)
            }
            HexFileFormat::SRecord => {
                srecord::save(path, self.config.word_addressing, &self.range, &self.data)
            }
        }
    }

    pub fn page_count(&self) -> u64 {
        self.data.len() as u64 / self.config.page_size
    }

    pub fn read_u16(&self, idx: usize) -> u16 {
        assert!(idx + 1 < self.data.len());
        let a = self.data[idx];
        let b = self.data[idx + 1];
        match self.config.endianness {
            Endianness::Big => b as u16 + ((a as u16) << 8),
            Endianness::Little => a as u16 + ((b as u16) << 8),
        }
    }

    pub fn read_u32(&self, idx: usize) -> u32 {
        let a = self.read_u16(idx) as u32;
        let b = self.read_u16(idx + 2) as u32;
        match self.config.endianness {
            Endianness::Big => b | (a << 16),
            Endianness::Little => a | (b << 16),
        }
    }

    pub fn write_u16(&mut self, idx: usize, data: u16) {
        let lsb = (data & 0xFF) as u8;
        let msb = ((data >> 8) & 0xFF) as u8;
        match self.config.endianness {
            Endianness::Big => {
                self.data[idx] = msb;
                self.data[idx + 1] = lsb;
            }
            Endianness::Little => {
                self.data[idx] = lsb;
                self.data[idx + 1] = msb;
            }
        }
    }

    pub fn write_u32(&mut self, idx: usize, data: u32) {
        match self.config.endianness {
            Endianness::Big => {
                self.write_u16(idx, ((data >> 16) & 0xFFFF) as u16);
                self.write_u16(idx + 2, (data & 0xFFFF) as u16);
            }
            Endianness::Little => {
                self.write_u16(idx, (data & 0xFFFF) as u16);
                self.write_u16(idx + 2, ((data >> 16) & 0xFFFF) as u16);
            }
        }
    }

    pub fn image_length(&self) -> usize {
        let mut k = self.data.len() - 1;
        while self.data[k] == 0xFF {
            k -= 1;
            if k == 0 {
                break;
            }
        }
        let len = k + 1;
        let page_size = self.config.page_size as usize;
        let last_page_size = len % page_size;
        if last_page_size == 0 {
            return len;
        }
        return len + page_size - last_page_size;
    }

    pub fn concatenate(first: &Firmware, second: &Firmware) -> Result<Firmware, Error> {
        if second.range.begin < first.range.begin + (first.data.len() as u64) {
            return Err(Error::InvalidAddress);
        }
        let gap = second.range.begin - (first.range.begin + (first.data.len() as u64));
        let new_range = AddressRange::new(first.range.begin, second.range.end);

        let mut new_code = Vec::new();
        new_code.extend(&first.data);
        new_code.extend(repeat(0xFF).take(gap as usize));
        new_code.extend(second.data.clone());

        let ret = Firmware::new(new_range, first.config.clone(), new_code)?;
        Ok(ret)
    }
}
