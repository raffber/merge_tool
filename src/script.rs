use crate::command::Command;
use sha2::{Sha256, Digest};
use std::iter::once;
use itertools::Itertools;

trait TimeModel {
    fn compute_write_time(&self, num_write: usize) -> f64;
    fn compute_read_time(&self, num_write: usize, num_read: usize) -> f64;

    fn compute(&self, cmds: &Vec<Command>) -> Vec<f64> {
        let mut ret = Vec::new();
        let mut now = 0.0_f64;
        let mut current_timeout = 0.0;
        for cmd in cmds {
            match cmd {
                Command::Write { write: write, read: Some(read) } => {
                    now += self.compute_read_time(write.len(), read.data.len());
                    now += current_timeout;
                    ret.push(now);
                },
                Command::Write { write: write, read: None } => {
                    now += self.compute_write_time(write.len());
                    now += current_timeout;
                    ret.push(now);
                },
                Command::Log(_) => {
                    ret.push(now);
                },
                Command::SetError(_) => {
                    ret.push(now);
                },
                Command::Header(_) => {
                    ret.push(now);
                },
                Command::SetTimeOut(x) => {
                    current_timeout = *x as f64 / 1000.0;
                    ret.push(now);
                },
                Command::Progress(_) => {
                    ret.push(now);
                },
                Command::Checksum(_) => {
                    ret.push(now);
                }
            }
        }
        ret
    }
}

struct SimpleTimeModel {
    read_byte_time: f64,
    write_byte_time: f64,
}

impl SimpleTimeModel {
    fn new(read_byte_time: f64, write_byte_time: f64) -> Self {
        Self { read_byte_time, write_byte_time }
    }
}

impl TimeModel for SimpleTimeModel {
    fn compute_write_time(&self, num_write: usize) -> f64 {
        (num_write as f64) * self.write_byte_time
    }

    fn compute_read_time(&self, num_write: usize, num_read: usize) -> f64 {
        self.compute_write_time(num_write) + self.read_byte_time * (num_read as f64)
    }
}

struct Script {
    commands: Vec<Command>,
    time_model: Box<dyn TimeModel>,
}

impl Script {
    fn new_with_model<T: 'static + TimeModel>(cmds: Vec<Command>, model: T) -> Self {
        Self {
            commands: cmds,
            time_model: Box::new(model)
        }
    }

    fn new(cmds: Vec<Command>) -> Self {
        Self {
            commands: cmds,
            time_model: Box::new(SimpleTimeModel::new(0.0, 0.0))
        }
    }

    pub fn postprocess(&mut self) {
        let mut new_cmds = Vec::new();
        let time = self.time_model.compute(&self.commands);
        let total: f64 = time.iter().sum();
        let mut last_progress = 0.0;
        for (cmd, now) in self.commands.iter().zip(time) {
            let cur_progress = now / total;
            new_cmds.push(cmd.clone());
            if cur_progress - last_progress > 4.0 / 256.0 {
                last_progress = cur_progress;
                let progress = (cur_progress * 100.0).round() as u8;
                new_cmds.push(Command::Progress(progress));
            }
        }
        self.commands = new_cmds;
    }

    pub fn serialize(&self) -> String {
        let lines: Vec<String> = self.commands.iter().map(|x| x.script_line()).collect();
        let mut sha = Sha256::new();
        for line in &lines {
            Digest::input(&mut sha, line.as_bytes());
        }
        let cmd = Command::Checksum(sha.result().to_vec());
        let checksum = cmd.script_line();
        lines.iter().chain(once(&checksum)).join("\n")
    }
}
