use byteorder::{ByteOrder, LittleEndian};
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub enum Command {
    Write(Vec<u8>),
    Query(Vec<u8>, Vec<u8>),
    Log(String),
    SetErrorMessage(String),
    Header(Vec<(String, String)>),
    SetTimeOut(u32),
    Progress(u8),
    Checksum(Vec<u8>),
}

pub const IDN_HEADER: u8 = 0x01;
pub const IDN_WRITE: u8 = 0x02;
pub const IDN_QUERY: u8 = 0x03;
pub const IDN_SET_TIMEOUT: u8 = 0x10;
pub const IDN_LOG: u8 = 0x20;
pub const IDN_SET_ERROR_MESSAGE: u8 = 0x21;
pub const IDN_PROGRESS: u8 = 0x22;
pub const IDN_CHECKSUM: u8 = 0x30;

impl Command {
    fn data(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        match self {
            Command::Query(write, read) => {
                let mut buf = [0_u8; 2];
                LittleEndian::write_u16(&mut buf, write.len() as u16);
                ret.extend(&buf);
                LittleEndian::write_u16(&mut buf, read.len() as u16);
                ret.extend(&buf);
                ret.extend(write);
                ret.extend(read);
            }
            Command::Write(write) => {
                ret.extend(write);
            }
            Command::Log(x) => {
                ret.extend(x.as_bytes());
            }
            Command::SetErrorMessage(x) => {
                ret.extend(x.as_bytes());
            }
            Command::Header(kv) => {
                let mut first = true;
                for (k, v) in kv {
                    if !first {
                        ret.push(b'|');
                    }
                    first = false;
                    ret.extend(k.as_bytes());
                    ret.push(b'=');
                    ret.extend(v.as_bytes());
                }
            }
            Command::SetTimeOut(timeout) => {
                let mut buf = [0_u8; 4];
                LittleEndian::write_u32(&mut buf, *timeout);
                ret.extend(&buf);
            }
            Command::Progress(v) => {
                ret.push(*v);
            }
            Command::Checksum(data) => ret.extend(data.iter()),
        }
        ret
    }

    fn identifier(&self) -> u8 {
        match self {
            Command::Header(_) => IDN_HEADER,
            Command::Write(_) => IDN_WRITE,
            Command::Query(_, _) => IDN_QUERY,
            Command::SetTimeOut(_) => IDN_SET_TIMEOUT,
            Command::Log(_) => IDN_LOG,
            Command::SetErrorMessage(_) => IDN_SET_ERROR_MESSAGE,
            Command::Progress(_) => IDN_PROGRESS,
            Command::Checksum(_) => IDN_CHECKSUM,
        }
    }

    pub fn script_line(&self) -> String {
        let data = hex::encode_upper(self.data());
        let identifier = hex::encode_upper(&[self.identifier()]);
        format!(":{}{}", identifier, data)
    }

    pub fn parse_line(line: &str) -> Result<Command, ParseError> {
        if line.len() < 5 || line.len() % 2 != 1 {
            return Err(ParseError::InvalidLength);
        }
        match line.chars().nth(0) {
            Some(':') => {}
            _ => return Err(ParseError::DelimiterMissing)
        }
        let line = &line[1..];
        let parsed: Result<Vec<u8>, ParseIntError> = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16))
            .collect();
        let parsed = parsed.map_err(|_| ParseError::InvalidHexCharacter)?;
        let cmd = parsed[0];
        let data = &parsed[1..];
        let ret = match cmd {
            IDN_HEADER => {
                let values =
                    String::from_utf8(data.to_vec()).map_err(|_| ParseError::InvalidEncoding)?;
                let mut header_data = Vec::new();
                for kv in values.split("|") {
                    let mut kv: Vec<_> = kv.split("=").map(|x| Some(x.to_string())).collect();
                    if kv.len() != 2 {
                        return Err(ParseError::InvalidHeaderFormat);
                    }
                    let key = kv[0].take().unwrap();
                    let value = kv[1].take().unwrap();
                    header_data.push((key, value));
                }
                Command::Header(header_data)
            }
            IDN_WRITE => Command::Write(data.to_vec()),
            IDN_QUERY => {
                if data.len() < 4 {
                    return Err(ParseError::InvalidLength);
                }
                let write_len = LittleEndian::read_u16(&data[0..2]) as usize;
                let read_len = LittleEndian::read_u16(&data[2..4]) as usize;
                let data = &data[4..];
                if data.len() != write_len + read_len {
                    return Err(ParseError::InvalidLength);
                }
                let write = data[0..write_len].to_vec();
                let read = data[write_len..].to_vec();
                Command::Query(write, read)
            }
            IDN_CHECKSUM => Command::Checksum(data.to_vec()),
            IDN_PROGRESS => {
                if data.len() != 1 {
                    return Err(ParseError::InvalidLength);
                }
                Command::Progress(data[0])
            }
            IDN_SET_ERROR_MESSAGE => match String::from_utf8(data.to_vec()) {
                Ok(x) => Command::SetErrorMessage(x),
                Err(_) => return Err(ParseError::InvalidEncoding),
            },
            IDN_LOG => match String::from_utf8(data.to_vec()) {
                Ok(x) => Command::Log(x),
                Err(_) => return Err(ParseError::InvalidEncoding),
            },
            IDN_SET_TIMEOUT => {
                if data.len() != 4 {
                    return Err(ParseError::InvalidLength);
                }
                let timeout = LittleEndian::read_u32(&data);
                Command::SetTimeOut(timeout)
            }
            _ => return Err(ParseError::InvalidCommand),
        };

        Ok(ret)
    }
}

#[derive(Debug)]
pub enum ParseError {
    DelimiterMissing,
    InvalidLength,
    InvalidHexCharacter,
    InvalidCommand,
    InvalidEncoding,
    InvalidHeaderFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let cmd = Command::Write(vec![0xA, 0xB, 0xC]);
        assert_eq!(cmd.script_line(), ":020A0B0C");
        let cmd = Command::Query(vec![0xA, 0xB, 0xC], vec![0xD, 0xE]);
        assert_eq!(cmd.script_line(), ":03030002000A0B0C0D0E");
        let cmd = Command::SetTimeOut(0xDEADBEEF);
        assert_eq!(cmd.script_line(), ":10EFBEADDE");
        let cmd = Command::Log("foobar".to_string());
        assert_eq!(cmd.script_line(), ":20666F6F626172");
        let cmd = Command::Header(vec![
            ("foo".to_string(), "bar".to_string()),
            ("more".to_string(), "stuff".to_string()),
        ]);
        assert_eq!(cmd.script_line(), ":01666F6F3D6261727C6D6F72653D7374756666");
        let cmd = Command::SetErrorMessage("foobar".to_string());
        assert_eq!(cmd.script_line(), ":21666F6F626172");
        let cmd = Command::Progress(0xAB);
        assert_eq!(cmd.script_line(), ":22AB");
    }

    #[test]
    fn round_trip_parse() {
        let cmd = Command::Write(vec![0xAB, 0xCD, 0xEF]);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Write(x) = parsed {
            assert_eq!(&x, &[0xAB, 0xCD, 0xEF])
        } else {
            panic!()
        }

        let cmd = Command::Query(vec![0xA, 0xB, 0xC], vec![0xD, 0xE]);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Query(a, b) = parsed {
            assert_eq!(&a, &[0xA, 0xB, 0xC]);
            assert_eq!(&b, &[0xD, 0xE]);
        } else {
            panic!()
        }


        let cmd = Command::Checksum(vec![0xA, 0xB, 0xC]);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Checksum(a) = parsed {
            assert_eq!(&a, &[0xA, 0xB, 0xC]);
        } else {
            panic!()
        }


        let cmd = Command::SetTimeOut(0xDEADBEEF);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::SetTimeOut(a) = parsed {
            assert_eq!(a, 0xDEADBEEF);
        } else {
            panic!()
        }


        let cmd = Command::SetErrorMessage("foobar".to_string());
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::SetErrorMessage(x) = parsed {
            assert_eq!(&x, "foobar");
        } else {
            panic!()
        }

        let cmd = Command::Progress(123);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Progress(x) = parsed {
            assert_eq!(x, 123);
        } else {
            panic!()
        }

        let cmd = Command::Log("foobar".to_string());
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Log(x) = parsed {
            assert_eq!(&x, "foobar");
        } else {
            panic!()
        }


        let cmd = Command::Header(vec![
            ("foo".to_string(), "bar".to_string()),
            ("bar".to_string(), "baz".to_string()),
            ("hello".to_string(), "world".to_string()),
        ]);
        let parsed = Command::parse_line(&cmd.script_line()).unwrap();
        if let Command::Header(x) = parsed {
            assert_eq!(x.len(), 3);
            assert_eq!(&x[0].0, "foo");
            assert_eq!(&x[0].1, "bar");
            assert_eq!(&x[1].0, "bar");
            assert_eq!(&x[1].1, "baz");
            assert_eq!(&x[2].0, "hello");
            assert_eq!(&x[2].1, "world");
        } else {
            panic!()
        }
    }
}
