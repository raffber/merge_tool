use byteorder::{ByteOrder, LittleEndian};

use crate::script_cmd::Command;
use crate::crc::crc8;
use crate::protocol::Protocol;

const CMD_NONE: u8 = 0x00;
const CMD_RESET: u8 = 0x01;
const CMD_VALIDATE: u8 = 0x02;
const CMD_START_TRANSMIT: u8 = 0x03;
const CMD_DATA: u8 = 0x04;
const CMD_FINISH: u8 = 0x05;
const CMD_LEAVE: u8 = 0x06;

const COM_OK: u8 = 0x00;

const STATE_NOT_IN_BTL: u8 = 0x00;
const STATE_IDLE: u8 = 0x01;
const STATE_VALIDATED: u8 = 0x02;
const STATE_ERASING: u8 = 0x03;
const STATE_RX_DATA: u8 = 0x04;
const STATE_CHECKING_CRC: u8 = 0x05;
const STATE_DONE: u8 = 0x06;
const STATE_ERR: u8 = 0x07;

const STATUS_SUCCESS: u8 = 0x00;

pub struct DdpProtocol {
    ddp_code: u8,
}

impl DdpProtocol {
    pub fn new(ddp_code: u8) -> Self {
        Self { ddp_code }
    }

    pub fn write(&self, mut data: Vec<u8>) -> Command {
        data.push(crc8(&data));
        Command::Write(data)
    }

    pub fn query(&self, mut tx: Vec<u8>, mut rx: Vec<u8>) -> Command {
        tx.push(crc8(&tx));
        rx.push(crc8(&rx));
        Command::Query(tx, rx)
    }
}

impl Protocol for DdpProtocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.ddp_code, fw_id, CMD_RESET]),
            Command::SetTimeOut(0),
            self.query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_IDLE, STATUS_SUCCESS],
            ),
        ]
    }

    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.ddp_code, fw_id, CMD_LEAVE]),
        ]
    }

    fn validate(&self, fw_id: u8, data: &[u8], wait_time: u32) -> Vec<Command> {
        let mut tx_data = vec![self.ddp_code, fw_id, CMD_VALIDATE];
        tx_data.extend(data);
        vec![
            Command::SetTimeOut(wait_time),
            self.write(tx_data),
            Command::SetTimeOut(0),
            self.query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_VALIDATED, STATUS_SUCCESS],
            ),
        ]
    }

    fn start_transmit(&self, fw_id: u8, erase_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(erase_time),
            self.write(vec![self.ddp_code, fw_id, CMD_START_TRANSMIT]),
            Command::SetTimeOut(0),
            self.query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_RX_DATA, STATUS_SUCCESS],
            ),
        ]
    }

    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command> {
        if data.iter().all(|x| *x == 0xFF) {
            return None;
        }
        let mut tx = vec![self.ddp_code, fw_id, CMD_DATA];
        let mut buf = [0_u8; 4];
        LittleEndian::write_u32(&mut buf, address as u32);
        tx.extend(buf.iter());
        tx.extend(data);
        Some(self.write(tx))
    }

    fn finish(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.ddp_code, fw_id, CMD_FINISH]),
            Command::SetTimeOut(0),
            self.query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_DONE, STATUS_SUCCESS],
            ),
        ]
    }
}
