use crate::script_cmd::Command;
use crate::config::{Config, FwConfig};
use crate::firmware::Firmware;
use crate::Error;

const DATA_LEN_PER_PACKAGE: usize = 16;

pub trait Protocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn validate(&self, fw_id: u8, data: &[u8], wait_time: u32) -> Vec<Command>;
    fn start_transmit(&self, fw_id: u8, erase_time: u32) -> Vec<Command>;
    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command>;
    fn finish(&self, fw_id: u8, send_done: u32, crc_check: u32) -> Vec<Command>;
    fn commit_chunk(&self, fw_id: u8, chunk_commit: u32) -> Vec<Command>;
}

fn make_header(config: &Config) -> Command {
    let mut header: Vec<_> = vec![
        ("product", config.product_name.clone()),
        ("product_id", config.product_id.to_string()),
        ("script_version", 1.to_string()),
    ].iter()
        .map(|(x,y)| (x.to_string(), y.to_string()))
        .collect();
    for fw in &config.images {
        header.push((
            format!("version_f{}", fw.fw_id),
            format!(
                "{}.{}.{}",
                config.major_version, fw.version.minor, fw.version.build
            ),
        ));
    }
    if config.use_backdoor {
        header.push(("backdoor".to_string(), "true".to_string()));
    }
    Command::Header(header)
}

pub fn generate_script<P: Protocol>(
    protocol: &P,
    fws: &[Firmware],
    config: &Config,
) -> Result<Vec<Command>, Error> {
    assert_eq!(fws.len(), config.images.len());
    let mut ret = Vec::new();

    ret.push(make_header(&config));
    for (fw, fw_config) in fws.iter().zip(&config.images) {
        let id = fw_config.fw_id;
        if !fw_config.include_in_script {
            ret.push(Command::Log(format!(
                "Skip bootload of {}!",
                fw_config.designator()
            )));
            continue;
        }
        ret.push(Command::Log(format!(
            "Entering bootloader on {}...",
            fw_config.designator()
        )));
        ret.extend(protocol.enter(id, config.time_state_transition));
        ret.push(Command::SetErrorMessage("Could not enter bootlader!".to_string()));
        ret.push(Command::Log("done".to_string()));

        let mut validation_data = [0_u8; 5];
        if config.use_backdoor {
            validation_data[0] = 0xFF;
            validation_data[1] = 0xFF;
        } else {
            validation_data[0] = config.product_id as u8 & 0xFF;
            validation_data[1] = ((config.product_id >> 8) & 0xFF) as u8;
        }
        validation_data[2] = config.major_version as u8 & 0xFF;
        validation_data[3] = ((config.major_version >> 8) & 0xFF) as u8;
        validation_data[4] = config.btl_version;
        ret.push(Command::Log("Validating firmware...".to_string()));
        ret.extend(protocol.validate(id, &validation_data, config.time_state_transition));
        ret.push(Command::SetErrorMessage("failed".to_string()));
        ret.push(Command::Log("done".to_string()));
        ret.push(Command::Log("Erasing...".to_string()));
        ret.extend(protocol.start_transmit(id, fw_config.timings.erase_time));
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Programming...".to_string()));

        if fw.data.len() % DATA_LEN_PER_PACKAGE != 0 {
            return Err(Error::NotAMultipleOfPackageLength);
        }

        if fw_config.write_chunk_size != 0 {
            if (fw_config.write_chunk_size as usize) % DATA_LEN_PER_PACKAGE != 0 {
                return Err(Error::NotAMultipleOfPackageLength);
            }
            send_junked(protocol, fw_config, fw, &mut ret);
        } else {
            send_all(protocol, fw_config, fw, &mut ret);
        }

        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Checking CRC...".to_string()));
        ret.extend(protocol.finish(id, fw_config.timings.data_send_done, fw_config.timings.crc_check));
        ret.push(Command::SetErrorMessage("failed".to_string()));
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Starting application...".to_string()));
        ret.extend(protocol.leave(id, fw_config.timings.leave_btl));
        ret.push(Command::SetErrorMessage("failed".to_string()));
        ret.push(Command::Log("done".to_string()));
    }

    ret.push(Command::Log("Bootload successful!".to_string()));
    Ok(ret)
}

pub fn send_all<P: Protocol>(protocol: &P, config: &FwConfig, fw: &Firmware, cmds: &mut Vec<Command>) {
    cmds.push(Command::SetTimeOut(config.timings.data_send));
    let id = config.fw_id;
    for k in (0..fw.data.len()).step_by(DATA_LEN_PER_PACKAGE) {
        let cmd = protocol.send_data(id, k as u64, &fw.data[k..k + DATA_LEN_PER_PACKAGE]);
        if let Some(cmd) = cmd {
            cmds.push(cmd);
        }
    }
}

pub fn send_junked<P: Protocol>(protocol: &P, config: &FwConfig, fw: &Firmware, cmds: &mut Vec<Command>) {
    let id = config.fw_id;

    for k_chunk in (0..fw.data.len()).step_by(config.write_chunk_size as usize) {
        cmds.push(Command::SetTimeOut(config.timings.data_send));
        for k_pkg in (0 .. config.write_chunk_size as usize).step_by(DATA_LEN_PER_PACKAGE) {
            let k = k_chunk + k_pkg;
            if k >= fw.data.len() {
                return;
            }
            let cmd = protocol.send_data(id, k as u64, &fw.data[k..k + DATA_LEN_PER_PACKAGE]);
            if let Some(cmd) = cmd {
                cmds.push(cmd);
            }
        }
        cmds.extend(protocol.commit_chunk(id, config.timings.chunk_commit));
    }
}
