use crate::command::Command;
use crate::config::Config;
use crate::firmware::Firmware;

const DATA_LEN_PER_PACKAGE: usize = 16;

pub trait Protocol {
    fn enter(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn leave(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn reset(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn start(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn send_validation_data(&self, fw_id: u8, data: &[u8]) -> Vec<Command>;
    fn check_validated(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
    fn enter_receive(&self, fw_id: u8, erase_time: u32, transition_time: u32) -> Vec<Command>;
    fn send_data(&self, fw_id: u8, address: u64, data: &[u8]) -> Option<Command>;
    fn finalize(&self, fw_id: u8, wait_time: u32) -> Vec<Command>;
}

fn make_header(config: &Config) -> Command {
    let mut header: Vec<_> = vec![
        ("product", config.product_name.clone()),
        ("product_id", config.product_id.to_string()),
        ("version", config.btl_version.to_string()),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
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
) -> Vec<Command> {
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
        ret.push(Command::SetError("Could not enter bootlader!".to_string()));
        ret.push(Command::Log("done".to_string()));
        ret.extend(protocol.start(id, config.time_state_transition));

        let mut validation_data = [0_u8; 4];
        if config.use_backdoor {
            validation_data[0] = 0xFF;
            validation_data[1] = 0xFF;
        } else {
            validation_data[0] = config.product_id as u8 & 0xFF;
            validation_data[1] = ((config.product_id >> 8) & 0xFF) as u8;
        }
        validation_data[2] = config.major_version;
        validation_data[3] = config.btl_version;
        ret.push(Command::Log("Validating firmware...".to_string()));
        ret.extend(protocol.send_validation_data(id, &validation_data));
        ret.extend(protocol.check_validated(id, config.time_state_transition));
        ret.push(Command::SetError("failed".to_string()));
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Start data transmission...".to_string()));
        ret.extend(protocol.enter_receive(
            id,
            fw_config.timings.erase_time,
            config.time_state_transition,
        ));
        ret.push(Command::SetError("failed".to_string()));
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::SetTimeOut(fw_config.timings.data_send));
        ret.push(Command::Log(format!(
            "Programming {}...",
            fw_config.designator()
        )));
        assert_eq!(fw.data.len() % DATA_LEN_PER_PACKAGE, 0);
        for k in (0..fw.data.len()).step_by(DATA_LEN_PER_PACKAGE) {
            let cmd = protocol.send_data(id, k as u64, &fw.data[k..k + DATA_LEN_PER_PACKAGE]);
            if let Some(cmd) = cmd {
                ret.push(cmd);
            }
        }
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Checking CRC...".to_string()));
        ret.extend(protocol.finalize(id, fw_config.timings.data_send_done));
        ret.push(Command::SetError("failed".to_string()));
        ret.push(Command::Log("done".to_string()));

        ret.push(Command::Log("Starting application...".to_string()));
        ret.extend(protocol.leave(id, fw_config.timings.leave_btl));
        ret.push(Command::SetError("failed".to_string()));
        ret.push(Command::Log("done".to_string()));
    }

    ret.push(Command::Log("Bootload successful!".to_string()));
    ret
}
