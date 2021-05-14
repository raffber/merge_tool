use futures::channel::mpsc;
use futures::Stream;
use merge_tool::config::Config;
use merge_tool::process;
use std::fs::create_dir_all;
use std::path::Path;
use std::thread;

#[derive(Debug)]
pub enum RunnerMsg {
    Info(String),
    Warn(String),
    Error(String),
    Failure(String),
    Success(String),
}

pub fn generate_script(mut config: Config, config_path: &Path) -> impl Stream<Item = RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let config_path = config_path.to_path_buf();
    thread::spawn(move || {
        let config_dir = match Config::get_config_dir(&config_path) {
            Ok(dir) => dir,
            Err(_) => {
                tx.unbounded_send(RunnerMsg::Failure(
                    "Unable to retrieve config directory.".to_string(),
                ))
                .unwrap();
                return;
            }
        };
        let output_dir = config_dir.join("out");
        if let Err(_) = create_dir_all(&output_dir) {
            tx.unbounded_send(RunnerMsg::Failure(
                "Cannot create output directory!".to_string(),
            ))
            .unwrap();
            return;
        }
        let msg = match process::create_script(&mut config, &config_dir, &output_dir) {
            Ok(_) => {
                tx.unbounded_send(RunnerMsg::Info(format!(
                    "Files written to `{}`",
                    output_dir.to_str().unwrap()
                )))
                .unwrap();
                RunnerMsg::Success("Successfully create script file!".to_string())
            }
            Err(err) => RunnerMsg::Failure(err.to_string()),
        };
        tx.unbounded_send(msg).unwrap();
    });
    rx
}

fn do_merge(
    config: &mut Config,
    config_dir: &Path,
    target_dir: &Path,
    tx: &mpsc::UnboundedSender<RunnerMsg>,
) -> Result<RunnerMsg, RunnerMsg> {
    create_dir_all(&target_dir)
        .map_err(|_| RunnerMsg::Failure("Cannot create output directory!".to_string()))?;
    let fws = process::merge_all(config, config_dir)
        .map_err(|err| RunnerMsg::Failure(format!("Cannot merge firmware images: {}", err)))?;
    process::write_fws(config, &fws, target_dir)
        .map_err(|err| RunnerMsg::Failure(format!("Cannot write firmware images: {}", err)))?;
    tx.unbounded_send(RunnerMsg::Info(format!(
        "Files written to `{}`",
        target_dir.to_str().unwrap()
    )))
    .unwrap();
    Ok(RunnerMsg::Success(
        "Successfully merged firmware files!".to_string(),
    ))
}

pub fn merge(config: Config, config_path: &Path) -> impl Stream<Item = RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let config_path = config_path.to_path_buf();
    thread::spawn(move || {
        let config_dir = match Config::get_config_dir(&config_path) {
            Ok(dir) => dir,
            Err(_) => {
                tx.unbounded_send(RunnerMsg::Failure(
                    "Unable to retrieve config directory.".to_string(),
                ))
                .unwrap();
                return;
            }
        };
        let target_dir = config_dir.join("out");
        let mut config = config;
        let msg = match do_merge(&mut config, &config_dir, &target_dir, &tx) {
            Ok(msg) => msg,
            Err(msg) => msg,
        };
        tx.unbounded_send(msg).unwrap();
    });
    rx
}
