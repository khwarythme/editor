use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use super::edit::Undo;
use crate::modules::history::EditOperation;

use crate::modules::show::MoveDirection;
use super::control_server::OperationCode;

use super::coordinate::Point;

/// insert a charactor on a point.
pub async fn proc_insert(
    code: OperationCode,
    buf: Arc<RwLock<VecDeque<VecDeque<char>>>>,
    undo: &mut Undo,
) -> OperationCode {
    match code {
        OperationCode::InsertChar(c, size, position) if c == '\n' => {
            undo.add_do_history(
                EditOperation::ADD,
                VecDeque::from(['\n']),
                Point {
                    column: position.column,
                    row: position.row,
                },
            );
            if position.row>= buf.read().unwrap().len() {
                buf.write().unwrap().push_back(VecDeque::new());
            }else {
                let mut buf = buf.write().unwrap();
                let line = buf.get_mut(position.row).unwrap();
                let mut new_line = VecDeque::new();
                for _ in position.column..line.len() {
                    new_line.push_back(line.pop_back().unwrap());
                }
                buf.insert(position.row + 1, new_line);
            }
           OperationCode::Move(MoveDirection::NewLine)
        }
        OperationCode::InsertChar(c,size,position) => {
           buf.write().unwrap().get_mut(position.row).unwrap().insert(
                position.column,
                c,
            );
            undo.add_do_history(
                EditOperation::ADD,
                VecDeque::from([c]),
                Point {
                    column: position.column,
                    row: position.row,
                },
            );
            OperationCode::Move(MoveDirection::Right)
        }
        OperationCode::BackSpace(size, position) => {
            
            if position.column <= 0 {
                if position.row > 0 {
                    let mut buf = buf.write().unwrap();
                    
                    let mut tmp_line =     buf.remove(position.row).unwrap_or(VecDeque::new());
                    buf.get_mut(position.row - 1).unwrap().append(&mut tmp_line);
                    let delchar = VecDeque::from(['\n']);
                    undo.add_do_history(
                        EditOperation::DELETE,
                        delchar,
                        Point {
                            column: position.column,
                            row: position.row,
                        },
                    );
                    OperationCode::Move(MoveDirection::Up)
                } else {
                    OperationCode::DoNothing
                };
            } else {
               let mut buf = buf.write().unwrap();
                let line = buf.get_mut(position.row).unwrap();
                let delchar = line.remove(position.column - 1).unwrap_or('\0');
                undo.add_do_history(
                    EditOperation::DELETE,
                    VecDeque::from([delchar]),
                    Point {
                        column: position.column,
                        row: position.row,
                    },
                );
                //display.update_all(buf.get_contents()).await.unwrap();
            };
            OperationCode::Move(MoveDirection::Left)
        }
        _ => {
            OperationCode::DoNothing
        },
    }
}
