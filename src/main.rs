mod modules;
use modules::command;
use modules::coordinate::Point;
use modules::edit::Undo;
use modules::edit::Yank;
use modules::file::FileBuffer;
use modules::insert::proc_insert;
use modules::lsp::client;
use modules::mode::{State, MODE};
use modules::normal::Normal;
use modules::search::Search;
use modules::show::*;

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::size;

use std::env;
use std::ffi::OsStr;
use std::path::Path;

fn main() {
    let args = env::args();
    let arg: Vec<String> = args.collect();
    let path = Path::new(&arg[1]);
    let mut buf = FileBuffer::new(path).expect("cannot open file");
    let (col, row) = size().unwrap();
    let mut display = Display::new(Point {
        column: col as usize,
        row: row as usize,
    });
    display.init_window();
    display.set_cursor_type(SetCursorStyle::SteadyBlock);

    let client_name = detect_file_type(path);
    if client_name != "" {
        let mut client = client::new(client_name);
        let _ = client.run(&buf);
    }

    handle(&mut display, &mut buf);
    display.close_terminal("".to_string());
}
fn detect_file_type(path: &Path) -> String {
    match path
        .extension()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap_or("")
    {
        //"rs" => "rust-analyzer".to_string(),
        //"c" => "clangd".to_string(),
        _ => "".to_string(),
    }
}

fn handle(display: &mut Display, buf: &mut FileBuffer) {
    let mut state = State::new();
    let mut column_prev: u16 = 0;
    let mut row_prev: u16 = 0;
    let is_required_update = true;
    let mut command: command::Command = command::Command::new();
    display.update_all(buf.get_contents()).unwrap();
    let mut undo = Undo::new();
    let mut yank = Yank::new();
    let mut sch = Search::new();

    loop {
        let rowsize = buf.get_contents().len();
        let percent = if rowsize > 0 {
            ((display.get_cursor_coordinate_in_file().row + 1) * 100) / rowsize
        } else {
            100
        };
        let cmd_buf = command.get_command_buf();
        display.update_info_line([
            buf.get_path()
                .clone()
                .split('/')
                .into_iter()
                .next_back()
                .unwrap_or(""),
            "",
            "",
            &cmd_buf,
            "",
            "",
            "",
            &format!("{}%", percent),
            &format!(
                "{}:{}",
                display.get_cursor_coordinate_in_file().row + 1,
                display.get_cursor_coordinate_in_file().column + 1
            ),
            "",
        ]);
        let (size_column, size_row) = size().unwrap();
        if is_required_update || column_prev != size_column || row_prev != size_row {
            display.update_wsize(Point {
                column: size_column as usize,
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

        let mut new_mode = match mode {
            MODE::Normal => Normal::proc_normal(code, display, buf, &mut undo, &mut yank),
            MODE::Insert => proc_insert(code, display, buf, &mut undo),
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
        match new_mode {
            MODE::Save => {
                let _ = buf.save_file();
                new_mode = MODE::Normal;
            }
            MODE::SaveAndQuit => {
                let _ = buf.save_file();
                break;
            }
            MODE::Quit => break,
            MODE::Jump(mut x) => {
                if x > buf.get_contents().len() as i32 {
                    x = buf.get_contents().len() as i32;
                } else if x < 1 {
                    x = 1;
                }
                display.move_to_point(
                    buf,
                    Point {
                        row: (x - 1) as usize,
                        column: display.get_cursor_coordinate_in_file().column,
                    },
                );
                let _ = display.update_all(buf.get_contents());

                new_mode = MODE::Normal;
            }
            _ => {}
        };
        let _ = display.update_all(buf.get_contents());
        state.change_mode(new_mode);
    }
}
