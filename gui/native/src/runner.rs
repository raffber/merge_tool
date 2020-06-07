use merge_tool::config::Config;
use futures::Stream;
use futures::channel::mpsc;
use std::thread;
use merge_tool::{process, Error};
use std::path::Path;

#[derive(Debug)]
pub enum RunnerMsg {
    Info(String),
    Warn(String),
    Error(String),
    Failure(String),
    Success(String),
}

pub fn generate_script(mut _config: Config, config_path: &Path) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    let _filepath = config_path.join("script.gtcbtl");
    thread::spawn(move || {
        // let msg = match process::create_script(&mut config, &filepath) {
        //     Ok(_) => RunnerMsg::Info("Successfully create script file!".to_string()),
        //     Err(err) => RunnerMsg::Error(err.to_string()),
        // };
        let msg = RunnerMsg::Info("Hello, World!".to_string());
        tx.unbounded_send(msg).unwrap();
        tx.unbounded_send(RunnerMsg::Success("Generated script file".to_string())).unwrap();
    });
    rx
}

pub fn release(config: Config) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    rx
}

pub fn merge(config: Config) -> impl Stream<Item=RunnerMsg> {
    let (tx, rx) = mpsc::unbounded();
    rx
}
