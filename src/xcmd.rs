use crate::command::Command;
use crate::protocol::Protocol;
use crate::crc::crc8;
use byteorder::{LittleEndian, ByteOrder};

struct ExtCmdProtocol {
    xcmd_code: u8
}

const CMD_NONE: u8 = 0x00;
const CMD_ADVANCE: u8 = 0x01;
const CMD_RESET: u8 = 0x02;
const CMD_DATA: u8 = 0x03;
const CMD_ENTER: u8 = 0x04;
const CMD_LEAVE: u8 = 0x05;

const COM_OK: u8 = 0x00;

const STATE_IDLE: u8 = 0x00;
const STATE_RX_VALIDATION: u8 = 0x01;
const STATE_VALIDATED: u8 = 0x02;
const STATE_RX_DATA: u8 = 0x03;
const STATE_FINISHED: u8 = 0x04;
const STATE_IN_APP: u8 = 0x05;
const STATE_ERR: u8 = 0x06;

const STATUS_SUCCESS: u8 = 0x06;

impl ExtCmdProtocol {
    fn new(xcmd_code: u8) -> Self {
        Self {
            xcmd_code
        }
    }

    fn write(&self, mut data: Vec<u8>) -> Command {
        data.push(crc8(&data));
        Command::Write(data)
    }

    fn query(&self, mut tx: Vec<u8>, mut rx: Vec<u8>) -> Command {
        tx.push(crc8(&tx));
        rx.push(crc8(&rx));
        Command::Query(tx, rx)
    }

    fn advance_state(&self, fw_id: u8, wait_time: u32, expected_state: u8) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.xcmd_code, fw_id, CMD_ADVANCE]),
            self.query(vec![self.xcmd_code | 0x80, fw_id],
                       vec![COM_OK, fw_id, expected_state, STATUS_SUCCESS]),
        ]

    }
}

impl Protocol for ExtCmdProtocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.xcmd_code, fw_id, CMD_ENTER]),
            self.query(vec![self.xcmd_code | 0x80, fw_id, CMD_ENTER],
                       vec![COM_OK, fw_id, STATE_IDLE, STATUS_SUCCESS]),
        ]
    }

    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.xcmd_code, fw_id, CMD_RESET])
        ]
    }

    fn reset(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            self.write(vec![self.xcmd_code, fw_id, CMD_RESET])
        ]
    }

    fn start(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        self.advance_state(fw_id, wait_time, STATE_RX_VALIDATION)
    }

    fn send_validation_data(&self, fw_id: u8, data: &[u8]) -> Vec<Command> {
        let mut tx_data = vec![self.xcmd_code, fw_id, CMD_DATA];
        tx_data.extend(data);
        vec![ Command::Write(tx_data) ]
    }

    fn check_validated(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        self.advance_state(fw_id, wait_time, STATE_VALIDATED)
    }

    fn enter_receive(&self, fw_id: u8, erase_time: u32, transition_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(erase_time),
            Command::Write(vec![self.xcmd_code, fw_id, CMD_ADVANCE]),
            Command::Query(vec![self.xcmd_code | 0x80, fw_id], vec![COM_OK, fw_id, STATE_RX_DATA, STATUS_SUCCESS]),
        ]
    }

    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command> {
        if data.iter().all(|x| *x == 0xFF) {
            return None;
        }
        let mut tx = vec![self.xcmd_code, fw_id, CMD_DATA];
        let mut buf = [0_u8; 4];
        LittleEndian::write_u32(&mut buf, address as u32);
        tx.extend(buf.iter());
        tx.extend(data);
        Some(Command::Write(tx))
    }

    fn finalize(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        self.advance_state(fw_id, wait_time, STATE_FINISHED)
    }
}

