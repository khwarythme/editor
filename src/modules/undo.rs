use crate::modules::file::FileBuffer;
use crate::modules::history::*;
use crate::modules::insert::*;

pub struct Undo {
    history: History,
    undo_history: History,
}

impl Undo {
    pub fn new() -> Undo {
        Undo {
            history: History::new(),
            undo_history: History::new(),
        }
    }
    pub fn add_do_history(&mut self, op: Operation, target: Vec<char>, pos: [u32; 2]) {
        self.history.add(op, target, pos);
    }
    pub fn undo(&mut self, buf: &mut FileBuffer) -> [u32; 2] {
        let record = self.history.undo();
        match record.get_operation() {
            Operation::HEAD => (),
            Operation::ADD => {
                let (result, delchar) = delback(
                    record.get_pos()[0] as u16,
                    record.get_pos()[1] as u16,
                    buf.get_contents(),
                );

                buf.update_contents(result);
                self.undo_history
                    .add(Operation::DELETE, delchar, record.get_pos());
            }
            Operation::DELETE => {
                buf.update_contents(insert(
                    record.get_pos()[0] as u16,
                    record.get_pos()[1] as u16,
                    buf.get_contents(),
                    record.get_target()[0],
                ));
                self.undo_history.add(
                    Operation::DELETE,
                    vec![record.get_target()[0]],
                    record.get_pos(),
                );
            }
            _ => (),
        };
        record.get_pos()
    }
}

