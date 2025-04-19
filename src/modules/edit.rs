use super::control_server::{Operation, OperationCode, ProcessID};
use super::insert;
use super::show::MoveDirection;
use crate::modules::coordinate::Point;
use crate::modules::history::*;
use std::sync::{mpsc::*, Arc, RwLock};

use std::collections::VecDeque;

pub struct Undo {
    history: History,
    undo_history: History,
}
pub enum UndoDirection {
    Undo,
    Redo,
}
pub struct Yank {
    yank: VecDeque<char>,
}
impl Yank {
    pub fn new() -> Self {
        Self {
            yank: VecDeque::new(),
        }
    }
    pub fn yankchars(&mut self, chars: VecDeque<char>) {
        self.yank = chars;
    }
    pub fn yank(&mut self, buf: VecDeque<VecDeque<char>>, start: Point) {
        let mut count = 0;
        for line in buf {
            if count == start.row {
                let c: VecDeque<char> = line.clone();
                self.yankchars(c);
            }
            count += 1;
        }
    }
    pub async fn past(
        &self,
        row: usize,
        buf: VecDeque<VecDeque<char>>,
        undo: &mut Undo,
    ) -> VecDeque<VecDeque<char>> {
        let mut tmp = self.yank.clone();
        tmp.push_back('\n');
        let ret = VecDeque::new();//= insert(Point { column: 0, row }, buf, tmp.clone()).await;
        undo.add_do_history(EditOperation::ADD, tmp, Point { column: 0, row });
        ret
    }
}

impl Undo {
    pub fn new() -> Undo {
        Undo {
            history: History::new(),
            undo_history: History::new(),
        }
    }
    pub fn add_do_history(&mut self, op: EditOperation, target: VecDeque<char>, pos: Point) {
        if target.len() > 0 {
            self.history.add(op, target, pos);
        }
    }
    pub async fn undo(
        &mut self,
        buf: Arc<RwLock<VecDeque<VecDeque<char>>>>,
        direction: UndoDirection,
    ) -> OperationCode {
        let record = match direction {
            UndoDirection::Undo => self.history.undo(),
            UndoDirection::Redo => self.undo_history.undo(),
        };
        let history = match direction {
            UndoDirection::Undo => &mut self.undo_history,
            UndoDirection::Redo => &mut self.history,
        };

        match record.get_operation() {
            EditOperation::ADD => {
                let mut buf = buf.write().unwrap();
                if buf.len() > record.get_pos().row {
                    if buf.get(record.get_pos().row).is_none() {
                        let line = buf.get_mut(record.get_pos().row - 1).unwrap();
                        for _ in 0..record.get_target().len() {
                            line.remove(record.get_pos().column);
                        }
                    }
                }else{
                    buf.remove(record.get_pos().row);
                }
                history.add(
                    EditOperation::DELETE,
                    record.get_target().clone(),
                    record.get_pos(),
                );
                OperationCode::Jump(Point {
                    column: record.get_pos().column,
                    row: record.get_pos().row,
                })
            }
            EditOperation::DELETE => {
                let mut buf = buf.write().unwrap();
                if record.get_target().back() == Some(&'\n') {
                    buf.insert(record.get_pos().row, VecDeque::new());
                }else {
                    let line = buf.get_mut(record.get_pos().row).unwrap();
                    for target in record.get_target() {
                        line.insert(record.get_pos().column, target);
                    }
                }
                
                history.add(
                    EditOperation::ADD,
                    record.get_target().clone(),
                    record.get_pos(),
                );
                OperationCode::Jump(Point {
                    column: record.get_pos().column,
                    row: record.get_pos().row,
                })
            }
            _ => OperationCode::DoNothing,
        }
    }
}

pub async fn thread_main(
    contents: Arc<RwLock<VecDeque<VecDeque<char>>>>,
    tx: Arc<RwLock<Sender<Operation>>>,
    rx: Receiver<Operation>,
) -> Result<(), String> {
    let mut undo = Undo::new();
    loop {
        let operation = match rx.recv() {
            Ok(operation) => operation,
            Err(_) => continue,
        };
        if operation.code == OperationCode::Quit {
            break;
        }

        let code = insert::proc_insert(operation.code, Arc::clone(&contents), &mut undo).await;
        let tx = tx.write().unwrap();
        tx.send(Operation {
            sender: ProcessID::Edit,
            code,
        });
    }

    Ok(())
}
