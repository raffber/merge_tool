use std::path::Path;
use crate::{Error, swap_bytearray};
use crate::config::{DeviceConfig, AddressRange};
use std::iter::repeat;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use hex;
use std::cmp::min;

struct Line {
    address: u64,
    data: Vec<u8>,
    kind: u8,
}


pub fn load(path: &Path, config: &DeviceConfig, range: &AddressRange) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(Error::Io)?;
    let lines: Result<Vec<_>, _> = BufReader::new(file)
        .lines()
        .map(|x| x.map_err(Error::Io).and_then(parse_line))
        .collect();
    let lines = lines?;
    let address_multiplier = if config.word_addressing { 2 } else { 1 };
    let mut extend_line_address = 0_u64;
    let mut ret: Vec<_>= repeat(0xFF_u8).take(range.len() as usize).collect();
    for line in &lines {
        if line.kind == 0x04 {
            if line.address == 0 {
                extend_line_address = (line.data[0] as u64) << 8;
                extend_line_address += line.data[1] as u64;
            } else {
                return Err(Error::InvalidHexFile);
            }
        } else if line.kind == 0x00 {
            let addr = ((extend_line_address << 16) | line.address) * address_multiplier;
            if addr < range.begin || addr > range.end {
                continue;
            }
            for k in 0 .. line.data.len() {
                let idx = k + (addr as usize) - (range.begin as usize);
                ret[idx] = line.data[k];
            }
        } else if line.address == 0x01 {
            break;
        }
    }
    if config.word_addressing {
        swap_bytearray(&mut ret);
    }
    Ok(ret)
}

fn parse_line(line: String) -> Result<Line, Error> {
    if line.as_bytes()[0] != b':' {
        return Err(Error::InvalidHexFile)
    }
    let data = hex::decode(&line[1..]).map_err(|x| Error::InvalidHexFile)?;
    let count = data[0] as usize;
    if count + 5 != data.len() {
        return Err(Error::InvalidHexFile);
    }
    let a1 = data[1] as u64;
    let a2 = data[2] as u64;
    let addr = a2 + (a1 << 8);
    let kind = data[3];
    let data: Vec<_> = data[4..4+count].to_vec();
    Ok(Line {
        address: addr,
        data,
        kind
    })
}

fn checksum(data: &[u8]) -> u8 {
    let result : i32 = data.iter().map(|x| *x as i32).sum();
    ((-1*result) & 0xFF_i32) as u8
}

const WRITE_DATA_PER_LINE: usize = 16;

pub fn serialize(config: &DeviceConfig, range: &AddressRange, data: &Vec<u8>) -> String {
    let mut data = data.clone();
    if config.word_addressing {
        swap_bytearray(&mut data);
    }
    let mut lines = Vec::new();
    for k in 0 .. data.len() {
        let endidx = min(k + WRITE_DATA_PER_LINE, data.len());
        let len = endidx - k;
        let mut address = (k as u64) + range.begin;
        if config.word_addressing {
            address >>= 1;
        }
        let mut out = Vec::new();
        out.extend(&[len as u8, (address >> 8) as u8, (address & 0xFF) as u8, 0xFF_u8]);
        let data_slice = &data[k..endidx];
        if data_slice.iter().all(|x| *x == 0xFF) {
            continue
        }
        out.extend(data_slice);
        out.push(checksum(&out));
        lines.push(out);
    }
    lines.push(vec![0x00, 0x00, 0x00, 0x01, 0xFF]);

    let lines: Vec<_> = lines.iter().map(|x| format!(":{}", hex::encode(x))).collect();
    lines.join("\n")
}

pub fn save(path: &Path, config: &DeviceConfig, range: &AddressRange, data: &Vec<u8>) -> Result<(), Error> {
    let data = serialize(config, range, data);
    let mut file = File::create(path).map_err(Error::Io)?;
    file.write_all(data.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(&[1,2,3]), 250);
        assert_eq!(checksum(&[254, 254]), 4);
    }
}
