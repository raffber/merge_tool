use merge_tool::config::Config;
use futures::Stream;
use futures::channel::mpsc;
use std::thread;
use merge_tool::process;
use std::path::{Path, PathBuf};
use std::fs::create_dir_all;

#[derive(Debug)]
pub enum RunnerMsg {
    Info(String),
    Warn(String),
    Error(String),
    Failure(String),
    Success(String),
}

fn config_dir(config_path: &Path) -> PathBuf {
    todo!()
}

pub fn generate_script(mut config: Config, config_path: &Path) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let config_dir = config_dir(config_path);
    thread::spawn(move || {
        let folderpath = config_dir.join("out");
        if let Err(_) = create_dir_all(&folderpath) {
            tx.unbounded_send(RunnerMsg::Failure("Cannot create output directory!".to_string())).unwrap();
            return;
        }
        let msg = match process::create_script(&mut config, &config_dir, &folderpath) {
            Ok(_) => RunnerMsg::Success("Successfully create script file!".to_string()),
            Err(err) => RunnerMsg::Failure(err.to_string()),
        };
        tx.unbounded_send(msg).unwrap();
    });
    rx
}

pub fn release(config: Config, config_path: &Path) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let config_dir = config_dir(config_path);
    thread::spawn(move || {
        let mut config = config;
        match process::release(&mut config, &config_dir) {
            Ok(_) => {
                tx.unbounded_send(RunnerMsg::Success("Successfully released firmware!".to_string())).unwrap();
            },
            Err(err) => {
                tx.unbounded_send(RunnerMsg::Success(format!("Error releaseing firmware: {}", err))).unwrap();
            },
        }
    });
    rx
}

fn do_merge(config: &mut Config, config_dir: &Path, target_dir: &Path) -> Result<RunnerMsg, RunnerMsg> {
    create_dir_all(&target_dir).map_err(|_| RunnerMsg::Failure("Cannot create output directory!".to_string()))?;
    let fws = process::merge_all(config, config_dir)
        .map_err(|err| RunnerMsg::Failure(format!("Cannot merge firmware images: {}", err)))?;
    process::write_fws(config, &fws, target_dir)
        .map_err(|err| RunnerMsg::Failure(format!("Cannot write firmware images: {}", err)))?;
    Ok(RunnerMsg::Success("Successfully merged firmware files!".to_string()))
}

pub fn merge(config: Config, config_path: &Path) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let config_dir = config_dir(config_path);
    let folderpath = config_dir.join("out");
    thread::spawn(move || {
        let mut config = config;
        let msg = match do_merge(&mut config, &config_dir, &folderpath) {
            Ok(msg) => msg,
            Err(msg) => msg,
        };
        tx.unbounded_send(msg).unwrap();
    });
    rx
}
