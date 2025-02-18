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
            KeyCode::Esc => MODE::Normal,
            _ => MODE::Command,
        }
    }
    pub fn exec_command(&mut self) -> MODE {
        let mut flg: u16 = 0x00;

        for cmd in &(self.inputs) {
            match cmd {
                'q' => flg |= 0x01,
                'w' => flg |= 0x02,
                _ => flg |= !0x03,
            }
        }
        self.inputs.clear();
        match flg {
            0x01 => MODE::Quit,
            0x02 => MODE::Save,
            0x03 => MODE::SaveAndQuit,
            _ => MODE::Normal,
        }
    }
}
