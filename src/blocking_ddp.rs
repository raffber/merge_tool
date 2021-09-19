use crate::ddp;
use crate::ddp::{
    CMD_DATA, CMD_FINISH, CMD_LEAVE, CMD_NONE, CMD_RESET, CMD_START_TRANSMIT, CMD_VALIDATE, COM_OK,
    STATE_DONE, STATE_IDLE, STATE_RX_DATA, STATE_VALIDATED, STATUS_SUCCESS,
};
use crate::protocol::Protocol;
use crate::script_cmd::Command;
use byteorder::{ByteOrder, LittleEndian};

pub struct BlockingDdpProtocol {
    ddp_code: u8,
}

impl BlockingDdpProtocol {
    pub fn new(ddp_code: u8) -> Self {
        Self { ddp_code }
    }
}

impl Protocol for BlockingDdpProtocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            ddp::write(vec![self.ddp_code, fw_id, CMD_RESET]),
            Command::SetTimeOut(0),
            ddp::query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_IDLE, STATUS_SUCCESS],
            ),
        ]
    }

    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        vec![
            Command::SetTimeOut(wait_time),
            ddp::write(vec![self.ddp_code, fw_id, CMD_LEAVE]),
        ]
    }

    fn validate(&self, fw_id: u8, data: &[u8], _wait_time: u32) -> Vec<Command> {
        let mut tx_data = vec![self.ddp_code | 0x80, fw_id, CMD_VALIDATE];
        tx_data.extend(data);
        vec![ddp::query(
            tx_data,
            vec![COM_OK, fw_id, STATE_VALIDATED, STATUS_SUCCESS],
        )]
    }

    fn start_transmit(&self, fw_id: u8, _erase_time: u32) -> Vec<Command> {
        vec![ddp::query(
            vec![self.ddp_code | 0x80, fw_id, CMD_START_TRANSMIT],
            vec![COM_OK, fw_id, STATE_RX_DATA, STATUS_SUCCESS],
        )]
    }

    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command> {
        if data.iter().all(|x| *x == 0xFF) {
            return None;
        }
        let mut tx = vec![self.ddp_code | 0x80, fw_id, CMD_DATA];
        let mut buf = [0_u8; 4];
        LittleEndian::write_u32(&mut buf, address as u32);
        tx.extend(buf.iter());
        tx.extend(data);
        Some(ddp::query(
            tx,
            vec![COM_OK, fw_id, STATE_RX_DATA, STATUS_SUCCESS],
        ))
    }

    fn finish(&self, fw_id: u8, _send_done: u32, _crc_check: u32) -> Vec<Command> {
        vec![
            ddp::query(
                vec![self.ddp_code | 0x80, fw_id, CMD_NONE],
                vec![COM_OK, fw_id, STATE_RX_DATA, STATUS_SUCCESS],
            ),
            ddp::query(
                vec![self.ddp_code | 0x80, fw_id, CMD_FINISH],
                vec![COM_OK, fw_id, STATE_DONE, STATUS_SUCCESS],
            ),
        ]
    }
}
