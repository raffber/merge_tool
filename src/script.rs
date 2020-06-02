use crate::command::Command;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::iter::once;

pub trait TimeModel {
    fn compute_write_time(&self, num_write: usize) -> f64;
    fn compute_read_time(&self, num_write: usize, num_read: usize) -> f64;

    fn compute(&self, cmds: &Vec<Command>) -> Vec<f64> {
        let mut ret = Vec::new();
        let mut now = 0.0_f64;
        let mut current_timeout = 0.0;
        for cmd in cmds {
            match cmd {
                Command::Query(write, read) => {
                    now += self.compute_read_time(write.len(), read.len());
                    now += current_timeout;
                    ret.push(now);
                }
                Command::Write(write) => {
                    now += self.compute_write_time(write.len());
                    now += current_timeout;
                    ret.push(now);
                }
                Command::Log(_) => {
                    ret.push(now);
                }
                Command::SetError(_) => {
                    ret.push(now);
                }
                Command::Header(_) => {
                    ret.push(now);
                }
                Command::SetTimeOut(x) => {
                    current_timeout = *x as f64 / 1000.0;
                    ret.push(now);
                }
                Command::Progress(_) => {
                    ret.push(now);
                }
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
        Self {
            read_byte_time,
            write_byte_time,
        }
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

pub struct Script {
    commands: Vec<Command>,
    time_model: Box<dyn TimeModel>,
}

impl Script {
    pub fn new_with_model<T: 'static + TimeModel>(cmds: Vec<Command>, model: T) -> Self {
        let mut ret = Self {
            commands: cmds,
            time_model: Box::new(model),
        };
        ret.compute_progres();
        ret
    }

    pub fn new(cmds: Vec<Command>) -> Self {
        let mut ret = Self {
            commands: cmds,
            time_model: Box::new(SimpleTimeModel::new(0.0, 0.0)),
        };
        ret.compute_progres();
        ret
    }

    fn compute_progres(&mut self) {
        let mut new_cmds = Vec::new();
        let time = self.time_model.compute(&self.commands);
        let total: f64 = time[time.len() - 1];
        let mut last_progress = 0.0;
        for (cmd, now) in self.commands.iter().zip(time) {
            let cur_progress = now / total;
            new_cmds.push(cmd.clone());
            if cur_progress - last_progress > 4.0 / 256.0 {
                last_progress = cur_progress;
                let progress = (cur_progress * 256.0).round() as u8;
                new_cmds.push(Command::Progress(progress));
            }
        }
        new_cmds.push(Command::Progress(255));
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use git2::FileMode::Commit;

    fn to_progress(ms_count: u32) -> u8 {
        ((ms_count as f64) / 101.0 * 256.0).round() as u8
    }

    #[test]
    fn check_progress() {
        let cmds = vec![
            Command::SetTimeOut(1),
            Command::Write(vec![]),
            Command::SetTimeOut(10), // no P should be inserted up to here
            Command::Write(vec![]),
            Command::Write(vec![]),
            Command::Write(vec![]),
            Command::Write(vec![]),
            Command::Write(vec![]),
            Command::Write(vec![]),
            Command::SetTimeOut(20),
            Command::Write(vec![]),
            Command::Write(vec![]),
        ];
        let mut script = Script::new(cmds);
        let mut iter = script.commands.iter();
        assert_matches!(iter.next(), Some(Command::SetTimeOut(_)));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::SetTimeOut(_)));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(11));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(21));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(31));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(41));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(51));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(61));
        assert_matches!(iter.next(), Some(Command::SetTimeOut(_)));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(81));
        assert_matches!(iter.next(), Some(Command::Write(_)));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == to_progress(101));
        assert_matches!(iter.next(), Some(Command::Progress(x)) if *x == 255);
        assert_matches!(iter.next(), None);
    }

    #[test]
    fn check_serialize() {
        let cmds = vec![
            Command::Header(vec![("foo".to_string(), "bar".to_string())]),
            Command::Write(vec![0xab, 0xcd, 0xef]),
            Command::Query(vec![0xab, 0xcd, 0xef], vec![0x12, 0x34]),
        ];
        let script = Script::new(cmds);
        let result = script.serialize();
        let mut splits = result.split("\n");
        assert_eq!(splits.next(), Some(":01666F6F3D626172"));
        assert_eq!(splits.next(), Some(":02ABCDEF"));
        assert_eq!(splits.next(), Some(":030302ABCDEF1234"));
        assert_eq!(splits.next(), Some(":22FF"));
        // now comes the SHA-256
        if let Some(x) = splits.next() {
            let x = x.as_bytes();
            assert_eq!(x[0], b':');
            assert_eq!(x[1], b'3');
            assert_eq!(x[2], b'0');
        } else {
            panic!()
        }
        assert_eq!(splits.next(), None);
    }
}
