use crate::command::Command;
use crate::protocol::Protocol;
use crate::crc::crc8;

struct ExtCmdProtocol {
    xcmd_code: u8
}

const CMD_NONE: u8 = 0x00;
const CMD_ADVANCE: u8 = 0x01;
const CMD_RESET: u8 = 0x02;
const CMD_DATA: u8 = 0x03;
const CMD_ENTER: u8 = 0x04;
const CMD_LEAVE: u8 = 0x05;

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

    fn query(&self, tx: Vec<u8>, rx: Vec<u8>) -> Command {
        todo!()
    }
}

impl Protocol for ExtCmdProtocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        // vec![
        //     Command::SetTimeOut(wait_time),
        //     self.write(vec![xcmd_code, fw_id, CMD_ENTER]),
        //     self.write(vec![xcmd_code | 0x80, fw_id, CMD_ENTER]),
        // ];
        todo!()
    }

    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        unimplemented!()
    }

    fn reset(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        unimplemented!()
    }

    fn start(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        unimplemented!()
    }

    fn send_validation_data(&self, fw_id: u8, data: &[u8]) -> Vec<Command> {
        unimplemented!()
    }

    fn check_validated(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        unimplemented!()
    }

    fn enter_receive(&self, fw_id: u8, erase_time: u32, transition_time: u32) -> Vec<Command> {
        unimplemented!()
    }

    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command> {
        unimplemented!()
    }

    fn finalize(&self, fw_id: u8, wait_time: u32) -> Vec<Command> {
        unimplemented!()
    }
}

