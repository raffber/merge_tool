use byteorder::{ByteOrder, LittleEndian};

#[derive(Clone, Debug)]
pub enum Command {
    Write(Vec<u8>),
    Query(Vec<u8>, Vec<u8>),
    Log(String),
    SetError(String),
    Header(Vec<(String, String)>),
    SetTimeOut(u32),
    Progress(u8),
    Checksum(Vec<u8>),
}

impl Command {
    fn data(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        match self {
            Command::Query(write, read) => {
                ret.push(write.len() as u8);
                ret.push(read.len() as u8);
                ret.extend(write);
                ret.extend(read);
            }
            Command::Write(write) => {
                ret.extend(write);
            }
            Command::Log(x) => {
                ret.extend(x.as_bytes());
            }
            Command::SetError(x) => {
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
        let cmd = Command::Write(vec![0xA, 0xB, 0xC]);
        assert_eq!(cmd.script_line(), ":020A0B0C");
        let cmd = Command::Query(vec![0xA, 0xB, 0xC], vec![0xD, 0xE]);
        assert_eq!(cmd.script_line(), ":0303020A0B0C0D0E");
        let cmd = Command::SetTimeOut(0xDEADBEEF);
        assert_eq!(cmd.script_line(), ":10EFBEADDE");
        let cmd = Command::Log("foobar".to_string());
        assert_eq!(cmd.script_line(), ":20666F6F626172");
        let cmd = Command::Header(vec![
            ("foo".to_string(), "bar".to_string()),
            ("more".to_string(), "stuff".to_string()),
        ]);
        assert_eq!(cmd.script_line(), ":01666F6F3D6261727C6D6F72653D7374756666");
        let cmd = Command::SetError("foobar".to_string());
        assert_eq!(cmd.script_line(), ":21666F6F626172");
        let cmd = Command::Progress(0xAB);
        assert_eq!(cmd.script_line(), ":22AB");
    }
}
