#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MODE {
    Normal,
    Insert,
    Visual,
    Command,
    Quit,
    Save,
    SaveAndQuit,
    Search,
}

#[derive(Debug)]
pub struct State {
    mode: MODE,
    is_read_only: bool,
}

impl State {
    pub fn change_mode(&mut self, new_mode: MODE) {
        self.mode = new_mode;
    }
    pub fn check_mode(&self) -> MODE {
        self.mode
    }
    pub fn set_read_only(&mut self, flg: bool) {
        self.is_read_only = flg;
    }
    pub fn get_read_only(&self) -> bool {
        self.is_read_only
    }
    pub fn new() -> State {
        State {
            mode: MODE::Normal,
            is_read_only: false,
        }
    }
    pub fn mode_manager(&self) {}
}
