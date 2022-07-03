use crate::config::AddressRange;
use crate::{load_lines, swap_bytearray, Error};
use hex;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::Path;

struct Line {
    address: u64,
    data: Vec<u8>,
    kind: u8,
}

pub fn load(path: &Path, word_addressing: bool, range: &AddressRange) -> Result<Vec<u8>, Error> {
    let lines = load_lines(path)?;
    parse(word_addressing, range, lines.into_iter())
}

pub fn parse<T: Iterator<Item = String>>(
    word_addressing: bool,
    range: &AddressRange,
    lines: T,
) -> Result<Vec<u8>, Error> {
    let lines: Result<Vec<_>, _> = lines.map(parse_line).collect();
    let lines = lines?;
    let mut extend_line_address = 0_u64;
    let multiplier = if word_addressing { 2 } else { 1 };
    let mut ret = vec![0xFF; range.len() as usize * multiplier];
    for line in &lines {
        match line.kind {
            0x04 => {
                if line.address == 0 {
                    extend_line_address = (line.data[0] as u64) << 8;
                    extend_line_address += line.data[1] as u64;
                    extend_line_address <<= 16;
                } else {
                    return Err(Error::InvalidHexFile);
                }
            }
            0x02 => {
                if line.address == 0 {
                    extend_line_address = (line.data[0] as u64) << 8;
                    extend_line_address += line.data[1] as u64;
                    extend_line_address *= 16;
                } else {
                    return Err(Error::InvalidHexFile);
                }
            }
            0x00 => {
                let addr = extend_line_address + line.address;
                if addr < range.begin || addr >= range.end {
                    continue;
                }
                for k in 0..line.data.len() {
                    let idx = k + (addr as usize) - (range.begin as usize);
                    ret[idx] = line.data[k];
                }
            }
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
        return Err(Error::InvalidHexFile);
    }
    if line.as_bytes()[0] != b':' {
        return Err(Error::InvalidHexFile);
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
    let data: Vec<_> = data[4..4 + count].to_vec();
    Ok(Line {
        address: addr,
        data,
        kind,
    })
}

fn checksum(data: &[u8]) -> u8 {
    let result: i32 = data.iter().map(|x| *x as i32).sum();
    ((-1 * result) & 0xFF_i32) as u8
}

const WRITE_DATA_PER_LINE: usize = 16;

fn get_extended_addr(addr: u64) -> u64 {
    (addr >> 16) & 0xFFFF
}

pub fn serialize(word_addressing: bool, range: &AddressRange, data: &[u8]) -> String {
    let mut data = data.to_vec();
    if word_addressing {
        swap_bytearray(&mut data);
    }
    let mut lines = Vec::new();
    let mut extended_addr = 0;
    for k in (0..data.len()).step_by(WRITE_DATA_PER_LINE) {
        let endidx = min(k + WRITE_DATA_PER_LINE, data.len());
        let endidx = min(endidx, (range.end - range.begin) as usize);
        if k >= endidx {
            break;
        }
        let len = endidx - k;
        let mut address = (k as u64) + range.begin;
        if word_addressing {
            address >>= 1;
        }
        if get_extended_addr(address) != extended_addr {
            extended_addr = get_extended_addr(address);
            lines.push(write_extended_addr(address));
        }
        let mut out = Vec::new();
        out.extend(&[
            len as u8,
            (address >> 8) as u8,
            (address & 0xFF) as u8,
            0_u8,
        ]);
        let data_slice = &data[k..endidx];
        if data_slice.iter().all(|x| *x == 0xFF) {
            continue;
        }
        out.extend(data_slice);
        out.push(checksum(&out));
        lines.push(out);
    }
    lines.push(vec![0x00, 0x00, 0x00, 0x01, 0xFF]);

    let lines: Vec<_> = lines
        .iter()
        .map(|x| format!(":{}", hex::encode_upper(x)))
        .collect();
    lines.join("\n")
}

fn write_extended_addr(addr: u64) -> Vec<u8> {
    let mut out = vec![0x02 as u8, 0x00, 0x00, 0x04];
    out.push(((addr >> 24) & 0xFF) as u8);
    out.push(((addr >> 16) & 0xFF) as u8);
    out.push(checksum(&out));
    out
}

pub fn save(
    path: &Path,
    word_addressing: bool,
    range: &AddressRange,
    data: &Vec<u8>,
) -> Result<(), Error> {
    let data = serialize(word_addressing, range, data);
    let mut file = File::create(path).map_err(Error::Io)?;
    file.write_all(data.as_bytes()).map_err(Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(&[1, 2, 3]), 250);
        assert_eq!(checksum(&[254, 254]), 4);
    }

    #[test]
    fn test_serialize() {
        let range = AddressRange::new(0xAB00, 0xAC00);
        let data: Vec<_> = (1u8..21).collect();
        let serialized = serialize(false, &range, &data);
        let mut iter = serialized.split("\n");
        assert_eq!(
            iter.next(),
            Some(":10AB00000102030405060708090A0B0C0D0E0F10BD")
        );
        assert_eq!(iter.next(), Some(":04AB100011121314F7"));
        assert_eq!(iter.next(), Some(":00000001FF"))
    }

    #[test]
    fn test_parse() {
        let range = AddressRange::new(0xAB00, 0xAB14);
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
