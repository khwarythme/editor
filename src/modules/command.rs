use crate::modules::mode::MODE;
use crossterm::event::KeyCode;
use std::path::Path;

use super::file::FileBuffer;

pub struct Command {
    inputs: Vec<char>,
}

impl Command {
    pub fn new() -> Command {
        Command { inputs: vec![] }
    }
    pub fn proc_command(&mut self, code: KeyCode, buf: &mut FileBuffer) -> MODE {
        match code {
            KeyCode::Char(c) => {
                self.inputs.push(c);
                MODE::Command
            }
            KeyCode::Enter => self.exec_command(buf),
            KeyCode::Backspace => {
                let _ = self.inputs.pop();
                MODE::Command
            }
            KeyCode::Esc => MODE::Normal,
            _ => MODE::Command,
        }
    }
    pub fn get_command_buf(&self) -> String {
        let ret: String = self.inputs.iter().collect();
        ret
    }
    pub fn exec_command(&mut self, buf: &mut FileBuffer) -> MODE {
        let inputs: String = self.inputs.iter().collect();
        let ret = match inputs {
            x if x.eq("wq") => MODE::SaveAndQuit,
            x if x.eq("w") => MODE::Save,
            x if x.eq("q") => MODE::Quit,
            x if x.parse::<i32>().is_ok() => MODE::Jump(x.parse::<i32>().unwrap()),
            x => {
                let arr: Vec<&str> = x.split(' ').into_iter().collect();
                if arr.len() > 0 {
                    match arr[0] {
                        "e" => {
                            let path = Path::new(arr[1]);
                            match FileBuffer::new(path) {
                                Ok(s) => buf.change_file(s),
                                Err(_) => {}
                            };
                        }
                        &_ => {}
                    }
                }
                MODE::Normal
            }
        };
        self.inputs.clear();
        ret
    }
}
