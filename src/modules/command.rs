use crate::modules::mode::MODE;
use crossterm::event::KeyCode;

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
            KeyCode::Enter => self.exec_command(),
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
    pub fn exec_command(&mut self) -> MODE {
        let inputs: String = self.inputs.iter().collect();
        let ret = match inputs {
            x if x.eq("wq") => MODE::SaveAndQuit,
            x if x.eq("w") => MODE::Save,
            x if x.eq("q") => MODE::Quit,
            x if x.parse::<i32>().is_ok() => MODE::Jump(x.parse::<i32>().unwrap()),
            _ => MODE::Normal,
        };
        self.inputs.clear();
        ret
    }
}
