use byteorder::{LittleEndian, ByteOrder};

#[derive(Clone)]
pub struct Readback {
    pub data: Vec<u8>,
    pub timeout: u16
}


#[derive(Clone)]
pub enum Command {
    Write { write: Vec<u8>, read: Option<Readback> },
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
            Command::Write { write: write, read: Some(read) } => {
                ret.push(write.len() as u8);
                ret.push(read.data.len() as u8);
                ret.extend(write);
                let mut buf = [0_u8; 2];
                LittleEndian::write_u16(&mut buf, read.timeout);
                ret.extend(&buf);
                ret.extend(&read.data);
            },
            Command::Write { write: write, read: None } => {
                ret.push(write.len() as u8);
                ret.push(0);
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
            Command::Header(_) => 01,
            Command::Write { .. } => 02,
            Command::Log(_) => 03,
            Command::SetError(_) => 04,
            Command::SetTimeOut(_) => 05,
            Command::Progress(_) => 06,
            Command::Checksum(_) => 07,
        }
    }

    pub fn script_line(&self) -> String {
        let data = hex::encode_upper(self.data());
        let identifier = hex::encode_upper(&[self.identifier()]);
        format!(":{}{}", identifier, data)
    }
}
