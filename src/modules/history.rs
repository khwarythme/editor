use std::collections::VecDeque;
#[derive(Clone)]
struct HistoryRecord {
    ope: Operation,
    target: Vec<char>,
    position: [u32; 2],
}
#[derive(Clone, Copy)]
pub enum Operation {
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
    fn new(operation: Operation, input: Vec<char>, pos: [u32; 2]) -> HistoryRecord {
        HistoryRecord {
            ope: operation,
            target: input,
            position: pos,
        }
    }
}
impl History {
    fn new() -> History {
        History {
            history: VecDeque::with_capacity(20),
            index: 0,
        }
    }
    fn add(&mut self, ope: Operation, target: Vec<char>, pos: [u32; 2]) {
        if self.history.len() >= 19 {
            self.history.pop_front();
            self.index -= self.index;
        }
        self.history.push_back(HistoryRecord::new(ope, target, pos));
        self.index += self.index + 1;
    }
    fn undo(&mut self) -> HistoryRecord {
        match (self.history.pop_back()) {
            Some(t) => t,
            None => HistoryRecord::new(Operation::HEAD, Vec::new(), [0, 0]),
        }
    }
}
