use byteorder::{LittleEndian, ByteOrder};

#[derive(Clone)]
pub struct Readback {
    pub data: Vec<u8>,
    pub timeout: u16
}


#[derive(Clone)]
pub enum Command {
    Write(Vec<u8>),
    Query(Vec<u8>, Readback),
    Log(String),
    SetError(String),
    Header(Vec<(String, String)>),
    SetTimeOut(u32),
    Progress(u8),
    Checksum(Vec<u8>)
}

impl Command {
    fn data(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        match self {
            Command::Query(write, read) => {
                ret.push(write.len() as u8);
                ret.push(read.data.len() as u8);
                ret.extend(write);
                let mut buf = [0_u8; 2];
                LittleEndian::write_u16(&mut buf, read.timeout);
                ret.extend(&buf);
                ret.extend(&read.data);
            },
            Command::Write(write) => {
                ret.push(write.len() as u8);
                ret.extend(write);
            },
            Command::Log(x) => {
                ret.extend(x.as_bytes());
            },
            Command::SetError(x) => {
                ret.extend(x.as_bytes());
            },
            Command::Header(kv) => {
                for (k, v) in kv {
                    ret.extend(k.as_bytes());
                    ret.push(b'|');
                    ret.extend(v.as_bytes());
                }
            },
            Command::SetTimeOut(timeout) => {
                let mut buf = [0_u8; 2];
                LittleEndian::write_u32(&mut buf, *timeout);
                ret.extend(&buf);
            },
            Command::Progress(v) => {
                ret.push(*v);
            },
            Command::Checksum(data) => {
                ret.extend(data.iter())
            }
        }
        ret
    }

    fn identifier(&self) -> u8 {
        match self {
            Command::Header(_) => 0x01,
            Command::Write(_) => 0x02,
            Command::Query(_, _) => 0x03,
            Command::SetTimeOut(_) => 0x10,
            Command::Log(_) => 0x20,
            Command::SetError(_) => 0x21,
            Command::Progress(_) => 0x22,
            Command::Checksum(_) => 0x30,
        }
    }

    pub fn script_line(&self) -> String {
        let data = hex::encode_upper(self.data());
        let identifier = hex::encode_upper(&[self.identifier()]);
        format!(":{}{}", identifier, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let read = Readback {
            data: vec![0xD, 0xE],
            timeout: 0x123
        };
        let cmd = Command::Write(vec![0xA, 0xB, 0xC]);
        assert_eq!(cmd.script_line(), ":02030A0B0C");
        let cmd = Command::Query(vec![0xA, 0xB, 0xC], read);
        assert_eq!(cmd.script_line(), ":0303020A0B0C23010D0E");
    }
}