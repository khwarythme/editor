use tokio;
use std::sync::{Arc,Mutex, mpsc::*};
use std::path::Path;
use std::time::Duration;

use crossterm::event::KeyCode;

use super::mode::MODE;
use super::coordinate::Point;
use super::*;
use super::show::MoveDirection;
use super::command::Command;

use super::inputs;

pub enum ProcessID{
    SystemControl,
    Show,
    FileControl,
    Input,
    Lsp,
    Mode,
    ProcessMax,
}
pub enum OperationCode{
    Start,
    Initialized,
    Update,
    ChangeMode(MODE),
    Input(KeyCode),
    InsertChar(char,usize, Point),
    Delete(usize,usize, Point),
    Move(MoveDirection),
    Jump(Point),
    Command(Command),
    End,
    Terminated,
    DoNothing,
}

pub struct Operation{
    pub sender: ProcessID,
    pub code: OperationCode,
}

pub async fn server_main(file_path: &Path) -> Result<(), String> {
    let file = vec![file::FileBuffer::new(file_path)?];
    let contents= Arc::new(Mutex::new(file[0].get_contents()));

    let (show_tx,show_rx) = channel::<Operation>();
    let (input_tx,input_rx) = channel::<Operation>();
    let (tx,rx) = channel::<Operation>();
    let tx = Arc::new(Mutex::new(tx));

    let show_thread = tokio::task::spawn(
    show::thread_main(Arc::clone(&contents),Arc::clone(&tx),show_rx    ));

    let input_thread = tokio::task::spawn(
    inputs::thread_main(Arc::clone(&tx), input_rx)
    );
    let edit_thread = tokio::task::spawn(
     edit::thread_main(Arc::clone(&contents)
    ));
    show_thread.await.unwrap_or(Err("tread error".to_string()))?;
    edit_thread.await.unwrap_or(Err("thread error".to_string()))?;
    input_thread.await.unwrap_or(Err("thread error".to_string()))?;
    Ok(())
}


async fn request_handler(senders: [Sender<Operation>; ProcessID::ProcessMax as usize],receiver: Receiver<Operation>) -> !{
    loop{
        let rcv_msg = match receiver.recv_timeout(Duration::from_millis(400)){
            Ok(msg) =>msg,
            Err(_) => Operation{
                sender: ProcessID::SystemControl,
                code : OperationCode::DoNothing,
            }
        };

    }
}


fn analyze_event(msg: Operation) -> (ProcessID, OperationCode){
    match msg.code {
        OperationCode::DoNothing => (ProcessID::ProcessMax, OperationCode::DoNothing),


    }
}
