use std::path::Path;
use crate::{Error, swap_bytearray};
use crate::config::AddressRange;
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


pub fn load(path: &Path, word_addressing: bool, range: &AddressRange) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(Error::Io)?;
    let lines = BufReader::new(file)
        .lines()
        .map(|x| x.unwrap().trim().to_string())
        .filter(|x| !x.is_empty());
    parse(word_addressing, range, lines)
}

pub fn parse<T: Iterator<Item=String>>(word_addressing: bool, range: &AddressRange, lines: T) -> Result<Vec<u8>, Error> {
    let lines: Result<Vec<_>, _> = lines
        .map(parse_line)
        .collect();
    let lines = lines?;
    let address_multiplier = if word_addressing { 2 } else { 1 };
    let mut extend_line_address = 0_u64;
    let mut ret: Vec<_> = repeat(0xFF_u8).take(range.len() as usize).collect();
    for line in &lines {
        match line.kind {
            0x04 => {
                if line.address == 0 {
                    extend_line_address = (line.data[0] as u64) << 8;
                    extend_line_address += line.data[1] as u64;
                } else {
                    return Err(Error::InvalidHexFile);
                }
            },
            0x00 => {
                let addr = ((extend_line_address << 16) | line.address) * address_multiplier;
                if addr < range.begin || addr > range.end {
                    continue;
                }
                for k in 0 .. line.data.len() {
                    let idx = k + (addr as usize) - (range.begin as usize);
                    ret[idx] = line.data[k];
                }
            },
            0x01 => break,
            _ => {}
        }
    }
    if word_addressing {
        swap_bytearray(&mut ret);
    }
    Ok(ret)
}

fn parse_line(line: String) -> Result<Line, Error> {
    if line.len() == 0 {
        return Err(Error::InvalidHexFile)
    }
    if line.as_bytes()[0] != b':' {
        return Err(Error::InvalidHexFile)
    }
    let data = hex::decode(&line[1..]).map_err(|_| Error::InvalidHexFile)?;
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

pub fn serialize(word_addressing: bool, range: &AddressRange, data: &Vec<u8>) -> String {
    let mut data = data.clone();
    if word_addressing {
        swap_bytearray(&mut data);
    }
    let mut lines = Vec::new();
    for k in (0 .. data.len()).step_by(WRITE_DATA_PER_LINE) {
        let endidx = min(k + WRITE_DATA_PER_LINE, data.len());
        let len = endidx - k;
        let mut address = (k as u64) + range.begin;
        if word_addressing {
            address >>= 1;
        }
        let mut out = Vec::new();
        out.extend(&[len as u8, (address >> 8) as u8, (address & 0xFF) as u8, 0_u8]);
        let data_slice = &data[k..endidx];
        if data_slice.iter().all(|x| *x == 0xFF) {
            continue
        }
        out.extend(data_slice);
        out.push(checksum(&out));
        lines.push(out);
    }
    lines.push(vec![0x00, 0x00, 0x00, 0x01, 0xFF]);

    let lines: Vec<_> = lines.iter().map(|x| format!(":{}", hex::encode_upper(x))).collect();
    lines.join("\n")
}

pub fn save(path: &Path, word_addressing: bool, range: &AddressRange, data: &Vec<u8>) -> Result<(), Error> {
    let data = serialize(word_addressing, range, data);
    let mut file = File::create(path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(&[1,2,3]), 250);
        assert_eq!(checksum(&[254, 254]), 4);
    }

    #[test]
    fn test_serialize() {
        let range = AddressRange::new(0xAB00, 0xABFF);
        let data: Vec<_> = (1u8..21).collect();
        let serialized = serialize(false, &range, &data);
        let mut iter = serialized.split("\n");
        assert_eq!(iter.next(), Some(":10AB00000102030405060708090A0B0C0D0E0F10BD"));
        assert_eq!(iter.next(), Some(":04AB100011121314F7"));
        assert_eq!(iter.next(), Some(":00000001FF"))
    }

    #[test]
    fn test_parse() {
        let range = AddressRange::new(0xAB00, 0xAB13);
        let file = r#"
        :10AB00000102030405060708090A0B0C0D0E0F10BD
        :04AB100011121314F7
        :00000001FF
        "#;
        let lines = file
            .split("\n")
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string());
        let parsed = parse(false, &range, lines).unwrap();
        let data: Vec<_> = (1u8..21).collect();
        assert_eq!(&parsed, &data);
   }
}
