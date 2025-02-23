mod modules;
use modules::command;
use modules::coordinate::Point;
use modules::file::FileBuffer;
use modules::insert::proc_insert;
use modules::mode::{State, MODE};
use modules::normal::Normal;
use modules::search::Search;
use modules::show::*;
use modules::undo::Undo;

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::size;

use std::env;
use std::path::Path;

fn main() {
    let args = env::args();
    let arg: Vec<String> = args.collect();
    let path = Path::new(&arg[1]);
    let mut buf = FileBuffer::new(path).expect("cannot open file");
    let (col, row) = size().unwrap();
    let mut display = Display::new(Point {
        col: col as usize,
        row: row as usize,
    });
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
    display.update_all(buf.get_contents()).unwrap();
    let mut undo = Undo::new();
    let mut sch = Search::new();

    loop {
        let (size_column, size_row) = size().unwrap();
        if is_required_update || column_prev != size_column || row_prev != size_row {
            display.update_wsize(Point {
                col: size_column as usize,
                row: size_row as usize,
            });
            row_prev = size_row;
            column_prev = size_column;
        }

        let input = match event::read().unwrap() {
            Event::Key(event) => event,
            _ => KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
        };
        if input.kind == KeyEventKind::Release {
            continue;
        }
        let code = input.code;
        let mode = state.check_mode();

        let new_mode = match mode {
            MODE::Normal => Normal::proc_normal(code, display, buf, &mut undo),
            MODE::Insert => {
                let ret = proc_insert(code, display, buf, &mut undo);
                display.update_all(buf.get_contents()).unwrap();
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
            MODE::Search => sch.proc_search(code, buf),
            m => m,
        };
        if new_mode == MODE::SaveAndQuit {
            buf.save_file().unwrap();
            break;
        }
        if new_mode == MODE::Quit {
            break;
        }
        state.change_mode(new_mode);
    }
}
