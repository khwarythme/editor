use std::io::{prelude, stdin};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};

use crossterm::event::{self,Event,KeyEvent,KeyCode,KeyEventKind,KeyEventState,KeyModifiers};

use super::control_server::{Operation,OperationCode,ProcessID};


pub async fn thread_main(tx: Arc<Mutex<Sender<Operation>>>,rx: Receiver<Operation>)-> Result<(), String>{
    let key_event = match event::read(){
        Ok(event) => {
            match event{
                Event::Key(c) => c,
                _ => KeyEvent::new(KeyCode::Null, KeyModifiers::empty())
            }
        }
        Err(e) => return Err(e.to_string()),
    };
    let tx_lock = tx.lock().unwrap();
    tx_lock.send(Operation{
        sender:ProcessID::Input,
        code:OperationCode::Input(key_event.code),

    }).unwrap();
    Ok(())
}

