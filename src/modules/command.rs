use super::control_server::{Operation, OperationCode, ProcessID};
use super::coordinate::Point;
use crate::modules::mode::MODE;
use crossterm::event::KeyCode;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

use super::file::FileBuffer;

pub struct Command {
    inputs: Vec<char>,
}

impl Command {
    pub fn new() -> Command {
        Command { inputs: vec![] }
    }
    pub fn proc_command(&mut self, code: char) -> OperationCode {
        match code {
            c if c as u32 == 0x03 => self.exec_command(),
            c if c as u32 == 0x08 => {
                let _ = self.inputs.pop();
                OperationCode::DoNothing
            }
            c if c as u32 == 0x1b => OperationCode::ChangeMode(MODE::Normal),
            c => {
                self.inputs.push(c);
                OperationCode::DoNothing
            }
        }
    }
    pub fn get_command_buf(&self) -> String {
        let ret: String = self.inputs.iter().collect();
        ret
    }
    pub fn exec_command(&mut self) -> OperationCode {
        let inputs: String = self.inputs.iter().collect();
        let ret = match inputs {
            x if x.eq("wq") => OperationCode::SaveAndQuit,
            x if x.eq("w") => OperationCode::Save,
            x if x.eq("q") => OperationCode::Quit,
            x if x.parse::<i32>().is_ok() => OperationCode::Jump(Point {
                column: 0,
                row: x.parse::<usize>().unwrap(),
            }),
            x => {
                let arr: Vec<&str> = x.split(' ').into_iter().collect();
                let ret = if arr.len() > 0 {
                    match arr[0] {
                        "e" => OperationCode::OpenFile(arr[1].to_string()),
                        _ => OperationCode::DoNothing,
                    }
                } else {
                    OperationCode::DoNothing
                };
                ret
            }
        };
        self.inputs.clear();
        ret
    }
}

pub async fn thread_main(
    sender: Arc<RwLock<Sender<Operation>>>,
    receiver: Receiver<Operation>,
) -> Result<(), String> {
    let mut proceedure = Command::new();
    loop {
        let result = receiver.recv().unwrap_or(Operation {
            sender: ProcessID::SystemControl,
            code: OperationCode::DoNothing,
        });
        let result = match result.code {
            OperationCode::Command(command) => proceedure.proc_command(command),
            OperationCode::Quit => break,
            _ => OperationCode::DoNothing,
        };
        if result != OperationCode::DoNothing {
            let lock = sender.read();
            let _ = lock.unwrap().send(Operation {
                sender: ProcessID::Command,
                code: result,
            });
            break;
        }
    }
    sender
        .read()
        .unwrap()
        .send(Operation {
            sender: ProcessID::Command,
            code: OperationCode::End,
        })
        .unwrap();
    Ok(())
}
