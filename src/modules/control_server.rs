use std::path::Path;
use std::sync::{mpsc::*, Arc, RwLock};

use tokio;

use crossterm::event::KeyCode;

use super::coordinate::Point;
use super::mode::MODE;
use super::show::MoveDirection;
use super::*;

use super::inputs;

pub enum ProcessID {
    SystemControl,
    Show,
    FileControl,
    Input,
    Edit,
    Command,
    Lsp,
    ProcessMax,
}
#[derive(Eq, PartialEq)]
pub enum OperationCode {
    Start,
    Initialized,
    Update,
    Undo,
    Redo,
    ChangeMode(MODE),
    Input(KeyCode),
    InsertChar(char, usize, Point),
    InputChar(char, Point),
    Delete(usize, Point),
    BackSpace(usize, Point),
    Move(MoveDirection),
    Jump(Point),
    Command(char),
    Function(u8),
    OpenFile(String),
    UpdatePosition(Point),
    Enter,
    Save,
    Quit,
    SaveAndQuit,
    Click,
    Scroll,
    End,
    Terminated,
    DoNothing,
}

pub struct Operation {
    pub sender: ProcessID,
    pub code: OperationCode,
}
pub async fn server_main(file_path: &Path) -> Result<(), String> {
    let file = vec![file::FileBuffer::new(file_path)?];
    let contents = Arc::new(RwLock::new(file[0].get_contents()));

    let (show_tx, show_rx) = channel::<Operation>();
    let (input_tx, input_rx) = channel::<Operation>();
    let (system_control_tx, system_cotnrol_rx) = channel::<Operation>();
    let (file_control_tx, file_controx_rx) = channel::<Operation>();
    let (lsp_tx, lsp_rx) = channel::<Operation>();
    let (command_tx, command_rx) = channel::<Operation>();
    let (edit_tx, edit_rx) = channel::<Operation>();
    let senders = [
        system_control_tx,
        show_tx,
        file_control_tx,
        input_tx,
        edit_tx,
        command_tx,
        lsp_tx,
    ];

    let (tx, rx) = channel::<Operation>();
    let tx = Arc::new(RwLock::new(tx));

    let show_thread = tokio::task::spawn(show::thread_main(
        Arc::clone(&contents),
        Arc::clone(&tx),
        show_rx,
    ));

    let input_thread = tokio::task::spawn(inputs::thread_main(Arc::clone(&tx), input_rx));
    let edit_thread = tokio::task::spawn(edit::thread_main(
        Arc::clone(&contents),
        Arc::clone(&tx),
        edit_rx,
    ));
    let command_thread = tokio::task::spawn(command::thread_main(Arc::clone(&tx), command_rx));
    let request_handler_thread = tokio::task::spawn(request_handler(senders, rx));
    show_thread.await;
    edit_thread
        .await
        .unwrap_or(Err("thread error".to_string()))?;
    input_thread.await;
    command_thread.await;
    request_handler_thread.await;
    Ok(())
}

async fn request_handler(
    senders: [Sender<Operation>; ProcessID::ProcessMax as usize],
    receiver: Receiver<Operation>,
) -> Result<(), String> {
    let mut current_mode = MODE::Normal;
    let mut position = Point { column: 0, row: 0 };
    loop {
        let rcv_msg = match receiver.recv() {
            Ok(msg) => msg,
            Err(_) => Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::DoNothing,
            },
        };
        let event = analyze_event(&current_mode, rcv_msg.code, position);

        let _result = match event {
            OperationCode::ChangeMode(m) => {
                current_mode = m;
                Ok(())
            }
            OperationCode::InputChar(c, p) => senders[ProcessID::Edit as usize].send(Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::InputChar(c, p),
            }),
            OperationCode::Undo => senders[ProcessID::Edit as usize].send(Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::Undo,
            }),
            OperationCode::Redo => senders[ProcessID::Edit as usize].send(Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::Redo,
            }),
            OperationCode::Delete(size, point) => {
                senders[ProcessID::Edit as usize].send(Operation {
                    sender: ProcessID::SystemControl,
                    code: OperationCode::Delete(size, point),
                })
            }
            OperationCode::BackSpace(size, point) => {
                senders[ProcessID::Edit as usize].send(Operation {
                    sender: ProcessID::SystemControl,
                    code: OperationCode::BackSpace(size, point),
                })
            }
            OperationCode::Move(direction) => senders[ProcessID::Show as usize].send(Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::Move(direction),
            }),
            OperationCode::Jump(point) => senders[ProcessID::Show as usize].send(Operation {
                sender: ProcessID::SystemControl,
                code: OperationCode::Jump(point),
            }),
            OperationCode::Command(command) => {
                senders[ProcessID::Command as usize].send(Operation {
                    sender: ProcessID::SystemControl,
                    code: OperationCode::Command(command),
                })
            }
            OperationCode::UpdatePosition(point) => {
                position = point;
                Ok(())
            }
            OperationCode::Quit => {
                println!("quit start");
                for sender in senders.clone() {
                    let _ = sender.send(Operation {
                        sender: ProcessID::SystemControl,
                        code: OperationCode::Quit,
                    });
                }
                Ok(())
            }
            OperationCode::End => return Ok(()),
            _ => Ok(()),
        };
    }
}

fn analyze_event(mode: &MODE, code: OperationCode, position: Point) -> OperationCode {
    match code {
        OperationCode::Input(c) => analyze_key_event(mode, c, position),
        OperationCode::UpdatePosition(p) => OperationCode::UpdatePosition(p),
        OperationCode::ChangeMode(m) => OperationCode::ChangeMode(m),
        OperationCode::Jump(p) => OperationCode::Jump(p),
        _ => code,
    }
 }
fn analyze_key_event(mode: &MODE, code: KeyCode, position: Point) -> OperationCode {
    match mode {
        MODE::Normal => analyze_key_event_normal(code),
        MODE::Insert => analyze_key_event_insert(code, position),
        MODE::Command => analyze_key_event_command(code),
        _ => analyze_key_event_normal(code),
    }
}

fn analyze_key_event_insert(code: KeyCode, position: Point) -> OperationCode {
    match code {
        KeyCode::Char(c) => OperationCode::InputChar(c, position),
        KeyCode::Backspace => OperationCode::BackSpace(1, position),
        KeyCode::Delete => OperationCode::Delete(1, position),
        KeyCode::Tab => OperationCode::InputChar('\t', position),
        KeyCode::Up => OperationCode::Move(MoveDirection::Up),
        KeyCode::Down => OperationCode::Move(MoveDirection::Down),
        KeyCode::Left => OperationCode::Move(MoveDirection::Left),
        KeyCode::Right => OperationCode::Move(MoveDirection::Right),
        KeyCode::Home => OperationCode::Move(MoveDirection::Head),
        KeyCode::End => OperationCode::Move(MoveDirection::Tail),
        KeyCode::Esc => OperationCode::ChangeMode(MODE::Normal),
        KeyCode::Enter => OperationCode::InputChar('\n', position),
        KeyCode::F(c) => OperationCode::Function(c),
        KeyCode::Modifier(_) => OperationCode::DoNothing,
        KeyCode::Media(_) => OperationCode::DoNothing,
        KeyCode::KeypadBegin => OperationCode::DoNothing,
        KeyCode::Menu => OperationCode::DoNothing,
        KeyCode::NumLock => OperationCode::DoNothing,
        KeyCode::ScrollLock => OperationCode::DoNothing,
        KeyCode::CapsLock => OperationCode::DoNothing,
        KeyCode::Null => OperationCode::DoNothing,
        KeyCode::PageDown => OperationCode::DoNothing,
        KeyCode::PageUp => OperationCode::DoNothing,
        KeyCode::Pause => OperationCode::DoNothing,
        KeyCode::PrintScreen => OperationCode::DoNothing,
        KeyCode::BackTab => OperationCode::DoNothing,
        KeyCode::Insert => OperationCode::DoNothing,
    }
}

fn analyze_key_event_normal(code: KeyCode) -> OperationCode {
    match code {
        KeyCode::Char(c) if c == 'j' => OperationCode::Move(MoveDirection::Down),
        KeyCode::Char(c) if c == 'h' => OperationCode::Move(MoveDirection::Left),
        KeyCode::Char(c) if c == 'k' => OperationCode::Move(MoveDirection::Up),
        KeyCode::Char(c) if c == 'l' => OperationCode::Move(MoveDirection::Right),
        KeyCode::Char(c) if c == 'i' => OperationCode::ChangeMode(MODE::Insert),
        KeyCode::Char(c) if c == 'u' => OperationCode::Undo,
        KeyCode::Char(c) if c == 'r' => OperationCode::Redo,
        KeyCode::Char(c) if c == ':' => OperationCode::ChangeMode(MODE::Command),
        KeyCode::Char(_) => OperationCode::DoNothing,
        KeyCode::Backspace => OperationCode::Move(MoveDirection::Left),
        KeyCode::Delete => OperationCode::DoNothing,
        KeyCode::Tab => OperationCode::DoNothing,
        KeyCode::Up => OperationCode::Move(MoveDirection::Up),
        KeyCode::Down => OperationCode::Move(MoveDirection::Down),
        KeyCode::Left => OperationCode::Move(MoveDirection::Left),
        KeyCode::Right => OperationCode::Move(MoveDirection::Right),
        KeyCode::Home => OperationCode::Move(MoveDirection::Head),
        KeyCode::End => OperationCode::Move(MoveDirection::Tail),
        KeyCode::Esc => OperationCode::ChangeMode(MODE::Normal),
        KeyCode::Enter => OperationCode::Enter,
        KeyCode::F(c) => OperationCode::Function(c),
        KeyCode::Modifier(_) => OperationCode::DoNothing,
        KeyCode::Media(_) => OperationCode::DoNothing,
        KeyCode::KeypadBegin => OperationCode::DoNothing,
        KeyCode::Menu => OperationCode::DoNothing,
        KeyCode::NumLock => OperationCode::DoNothing,
        KeyCode::ScrollLock => OperationCode::DoNothing,
        KeyCode::CapsLock => OperationCode::DoNothing,
        KeyCode::Null => OperationCode::DoNothing,
        KeyCode::PageDown => OperationCode::DoNothing,
        KeyCode::PageUp => OperationCode::DoNothing,
        KeyCode::Pause => OperationCode::DoNothing,
        KeyCode::PrintScreen => OperationCode::DoNothing,
        KeyCode::BackTab => OperationCode::DoNothing,
        KeyCode::Insert => OperationCode::DoNothing,
    }
}

fn analyze_key_event_command(code: KeyCode) -> OperationCode {
    match code {
        KeyCode::Char(c) => OperationCode::Command(c),
        KeyCode::Backspace => OperationCode::Command(0x08 as char),
        KeyCode::Delete => OperationCode::DoNothing,
        KeyCode::Tab => OperationCode::DoNothing,
        KeyCode::Up => OperationCode::DoNothing,
        KeyCode::Down => OperationCode::DoNothing,
        KeyCode::Left => OperationCode::DoNothing,
        KeyCode::Right => OperationCode::DoNothing,
        KeyCode::Home => OperationCode::DoNothing,
        KeyCode::End => OperationCode::DoNothing,
        KeyCode::Esc => OperationCode::ChangeMode(MODE::Normal),
        KeyCode::Enter => OperationCode::Command(0x03 as char),
        KeyCode::F(_) => OperationCode::DoNothing,
        KeyCode::Modifier(_) => OperationCode::DoNothing,
        KeyCode::Media(_) => OperationCode::DoNothing,
        KeyCode::KeypadBegin => OperationCode::DoNothing,
        KeyCode::Menu => OperationCode::DoNothing,
        KeyCode::NumLock => OperationCode::DoNothing,
        KeyCode::ScrollLock => OperationCode::DoNothing,
        KeyCode::CapsLock => OperationCode::DoNothing,
        KeyCode::Null => OperationCode::DoNothing,
        KeyCode::PageDown => OperationCode::DoNothing,
        KeyCode::PageUp => OperationCode::DoNothing,
        KeyCode::Pause => OperationCode::DoNothing,
        KeyCode::PrintScreen => OperationCode::DoNothing,
        KeyCode::BackTab => OperationCode::DoNothing,
        KeyCode::Insert => OperationCode::DoNothing,
    }
}
