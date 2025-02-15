mod modules;
use modules::command;
use modules::coordinate::Point;
use modules::file::FileBuffer;
use modules::insert::proc_insert;
use modules::mode::{State, MODE};
use modules::normal::Normal;
use modules::show::*;

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::size;

use std::env;
use std::path::Path;

fn main() {
    let args = env::args();
    let arg: Vec<String> = args.collect();
    let path = Path::new(&arg[1]);
    let mut buf = FileBuffer::new(path).expect("cannot open file");
    let (col, row) = size().unwrap();
    let mut display = Display::new(Point { col: col, row: row });
    display.init_window();
    display.set_cursor_type(SetCursorStyle::SteadyBlock);
    handle(&mut display, &mut buf);
    display.close_terminal("".to_string());
}

fn handle(display: &mut Display, buf: &mut FileBuffer) {
    let mut state = State::new();
    let mut column_prev: u16 = 0;
    let mut row_prev: u16 = 0;
    let is_required_update = true;
    let mut command: command::Command = command::Command::new();
    display.update(buf.get_contents()).unwrap();

    loop {
        let (size_column, size_row) = size().unwrap();
        if is_required_update || column_prev != size_column || row_prev != size_row {
            display.update_wsize(Point {
                col: size_column,
                row: size_row,
            });
            row_prev = size_row;
            column_prev = size_column;
        }

        let input = match event::read().unwrap() {
            Event::Key(event) => event,
            _ => KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
        };
        let code = input.code;
        let mode = state.check_mode();

        let mode = match mode {
            MODE::Normal => Normal::proc_normal(code, display, buf),
            MODE::Insert => {
                let ret = proc_insert(code, display, buf);
                display.update(buf.get_contents()).unwrap();
                ret
            }
            MODE::Command => command.proc_command(code, buf),
            MODE::Visual => MODE::Normal,
            MODE::Save => {
                buf.save_file().unwrap();
                MODE::Normal
            }
            MODE::SaveAndQuit => {
                buf.save_file().unwrap();
                MODE::Quit
            }
            m => m,
        };
        if mode == MODE::Quit {
            break;
        }
        state.change_mode(mode);
    }
}
