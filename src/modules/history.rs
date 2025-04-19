use std::collections::VecDeque;

use super::coordinate::Point;
#[derive(Clone)]
pub struct HistoryRecord {
    ope: EditOperation,
    target: VecDeque<char>,
    position: Point,
}
#[derive(Clone, Copy)]
pub enum EditOperation {
    HEAD,
    ADD,
    DELETE,
    COMMAND,
}
pub struct History {
    history: VecDeque<HistoryRecord>,
    index: u32,
}

impl HistoryRecord {
    fn new(operation: EditOperation, input: VecDeque<char>, pos: Point) -> HistoryRecord {
        HistoryRecord {
            ope: operation,
            target: input,
            position: pos,
        }
    }
    pub fn get_operation(&self) -> EditOperation {
        self.ope
    }
    pub fn get_pos(&self) -> Point {
        self.position
    }
    pub fn get_target(&self) -> VecDeque<char> {
        self.target.clone()
    }
}
impl History {
    pub fn new() -> History {
        History {
            history: VecDeque::with_capacity(1000),
            index: 0,
        }
    }
    pub fn add(&mut self, ope: EditOperation, target: VecDeque<char>, pos: Point) {
        if self.history.len() >= 999 {
            self.history.pop_front();
            self.index -= 1;
        }
        self.history.push_back(HistoryRecord::new(ope, target, pos));
        self.index += 1;
    }
    pub fn undo(&mut self) -> HistoryRecord {
        match self.history.pop_back() {
            Some(t) => t,
            None => HistoryRecord::new(
                EditOperation::HEAD,
                VecDeque::new(),
                Point { column: 0, row: 0 },
            ),
        }
    }
}
