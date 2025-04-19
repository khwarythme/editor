use std::io::{prelude, stdin};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use super::control_server::{Operation, OperationCode, ProcessID};

pub async fn thread_main(
    tx: Arc<RwLock<Sender<Operation>>>,
    rx: Receiver<Operation>,
) -> Result<(), String> {
    loop {
        let key_event = match event::read().unwrap() {
            event => match event {
                Event::Key(c) => c,
                _ => KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            },
        };
        if key_event.kind == KeyEventKind::Release {
            continue;
        }
        let tx_lock = tx.read().unwrap();
        tx_lock
            .send(Operation {
                sender: ProcessID::Input,
                code: OperationCode::Input(key_event.code),
            })
            .unwrap_or(());
        let result = rx.recv_timeout(Duration::from_nanos(1));
        if result.is_ok() {
            if result.unwrap().code == OperationCode::Quit {
                break;
            }
        }
    }
    let _ = tx.read().unwrap().send(Operation {
        sender: ProcessID::Input,
        code: OperationCode::End,
    });
    Ok(())
}
