use crate::config::AddressRange;
use crate::{load_lines, swap_bytearray, Error};
use hex;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::from_utf8;

struct Line {
    data: Vec<u8>,
    addr: u64,
    kind: String,
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
    let multiplier = if word_addressing { 2 } else { 1 };
    let mut ret = vec![0xFF; range.len() as usize];
    for line in lines {
        match line.kind.as_str() {
            "S0" => continue,
            "S3" => {
                let addr = line.addr * multiplier;
                if addr < range.begin || addr > range.end {
                    continue;
                }
                for k in 0..line.data.len() {
                    let idx = k + (addr as usize) - (range.begin as usize);
                    ret[idx] = line.data[k];
                }
            }
            "S7" => {
                break;
            }
            _ => {
                return Err(Error::InvalidHexFile);
            }
        }
    }
    if word_addressing {
        swap_bytearray(&mut ret);
    }
    Ok(ret)
}

fn parse_line(line: String) -> Result<Line, Error> {
    let line = line.as_bytes();
    if line.len() < 2 {
        return Err(Error::InvalidHexFile);
    }
    let kind = from_utf8(&line[0..2])
        .map_err(|_| Error::InvalidHexFile)?
        .to_string();
    let decoded = hex::decode(&line[2..]).map_err(|_| Error::InvalidHexFile)?;
    let cnt = decoded[0];
    if cnt as usize != decoded.len() - 1 {
        return Err(Error::InvalidHexFile);
    }
    let sum: u32 = decoded.iter().map(|x| *x as u32).sum();
    if sum & 0xFF != 0xFF {
        return Err(Error::InvalidHexFile);
    }
    let addr = &decoded[1..5];
    let addr = ((addr[0] as u64) << 24)
        | ((addr[1] as u64) << 16)
        | ((addr[2] as u64) << 8)
        | (addr[3] as u64);
    let data: Vec<_> = decoded[5..decoded.len() - 1].iter().map(|x| *x).collect();
    Ok(Line { data, addr, kind })
}

const WRITE_DATA_PER_LINE: usize = 16;

pub fn serialize(word_addressing: bool, range: &AddressRange, data: &[u8]) -> String {
    let mut data = data.to_vec();
    if word_addressing {
        swap_bytearray(&mut data);
    }
    let mut lines = Vec::new();
    for k in (0..data.len()).step_by(WRITE_DATA_PER_LINE) {
        let endidx = min(k + WRITE_DATA_PER_LINE, data.len());
        let endidx = min(endidx, (range.end - range.begin) as usize);
        if k > endidx {
            break;
        }
        let len = endidx - k + 5;
        let mut address = (k as u64) + range.begin;
        if word_addressing {
            address >>= 1;
        }
        let mut out = Vec::new();
        out.extend(&[
            len as u8,
            (address >> 24) as u8,
            (address >> 16) as u8,
            (address >> 8) as u8,
            (address & 0xFF) as u8,
        ]);
        out.extend(&data[k..endidx]);
        let sum: u32 = out.iter().map(|x| *x as u32).sum();
        let sum = (sum & 0xFF) as u8;
        out.push(!sum);
        let line = format!("S3{}", hex::encode_upper(out));
        lines.push(line);
    }
    // write S7
    let out = &[0x05, 0x00, 0x00, 0x00, 0x00, !5];
    let line = format!("S7{}", hex::encode_upper(out));
    lines.push(line);

    lines.join("\n")
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
    fn test_serialize() {
        let range = AddressRange::new(0xAB00, 0xABFF);
        let data: Vec<_> = (1u8..21).collect();
        let serialized = serialize(false, &range, &data);
        let mut iter = serialized.split("\n");
        assert_eq!(
            iter.next(),
            Some("S3150000AB000102030405060708090A0B0C0D0E0F10B7")
        );
        assert_eq!(iter.next(), Some("S3090000AB1011121314F1"));
        assert_eq!(iter.next(), Some("S70500000000FA"));
        assert_eq!(iter.next(), None)
    }

    #[test]
    fn test_parse() {
        let range = AddressRange::new(0xAB00, 0xABFF);
        let file = r#"
        S3150000ab000102030405060708090a0b0c0d0e0f10B7
        S3090000ab1011121314f1
        S70500000000fa
        "#;
        let lines = file
            .split("\n")
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string());
        let parsed = parse(false, &range, lines).unwrap();
        let data: Vec<_> = (1u8..21).collect();
        assert_eq!(&parsed[0..20], &data);
    }
}
