use std::collections::VecDeque;
#[derive(Clone)]
pub struct HistoryRecord {
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
    pub fn get_operation(&self) -> Operation {
        self.ope
    }
    pub fn get_pos(&self) -> [u32; 2] {
        self.position
    }
    pub fn get_target(&self) -> &Vec<char> {
        &self.target
    }
}
impl History {
    pub fn new() -> History {
        History {
            history: VecDeque::with_capacity(1000),
            index: 0,
        }
    }
    pub fn add(&mut self, ope: Operation, target: Vec<char>, pos: [u32; 2]) {
        if self.history.len() >= 999{
            self.history.pop_front();
            self.index -= 1;
        }
        self.history.push_back(HistoryRecord::new(ope, target, pos));
        self.index += 1;
    }
    pub fn undo(&mut self) -> HistoryRecord {
        match self.history.pop_back() {
            Some(t) => t,
            None => HistoryRecord::new(Operation::HEAD, Vec::new(), [0, 0]),
        }
    }
}
