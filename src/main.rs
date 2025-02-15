mod modules;
use modules::file::FileBuffer;
use modules::insert::{delback, insert};
use modules::mode::{State, MODE};
use modules::show::Display;

use crossterm::cursor;
use crossterm::cursor::{MoveTo, SetCursorStyle};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::size;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::{Clear, ClearType};
use crossterm::terminal::{ScrollDown, ScrollUp};

use std::env;
use std::path::Path;

use std::io::{stdout, Stdout};
#[derive(Clone, Copy)]
struct Point {
    col: u16,
    row: u16,
}
fn main() {
    let mut out = stdout();
    let args = env::args();
    let arg: Vec<String> = args.collect();
    let path = Path::new(&arg[1]);
    let mut buf = FileBuffer::new(path).expect("cannot open file");
    let mut display = init_window().unwrap();
    execute!(out, SetCursorStyle::SteadyBlock).unwrap();
    handle(&mut display, &mut buf, &mut out);

    close_terminal("".to_string(), &mut out);
}
fn init_window() -> Result<Display, String> {
    match enable_raw_mode() {
        Ok(_) => Ok(Display::new()),
        Err(e) => Err(e.to_string()),
    }
}

fn close_terminal(err: String, out: &mut Stdout) {
    print!("{}", err);
    execute!(out, cursor::Show, LeaveAlternateScreen,).expect("failed to close alternate screen");
    disable_raw_mode().expect("");
}
fn handle(display: &mut Display, buf: &mut FileBuffer, out: &mut Stdout) {
    let mut point: Point = Point { col: 0, row: 0 };
    let mut point_in_file = Point { col: 0, row: 0 };
    let mut state = State::new();
    let mut pos_tmp: u16 = 0;
    let mut column_prev: u16 = 0;
    let mut row_prev: u16 = 0;
    let mut is_required_update = true;

    loop {
        let (size_column, size_row) = size().unwrap();
        if is_required_update || column_prev != size_column || row_prev != size_row {
            row_prev = size_row;
            column_prev = size_row;
            execute!(
                out,
                cursor::Show,
                EnterAlternateScreen,
                Clear(ClearType::All),
                MoveTo(point.col, point.row)
            )
            .expect("Failed to open alternate screen");
            execute!(out, Clear(ClearType::All))
                .unwrap_or_else(|e| close_terminal(e.to_string(), out));
            execute!(out, MoveTo(0, 0)).unwrap_or_else(|_| {
                close_terminal("[E101] failed to move cursor".to_string(), out)
            });
            display
                .update(
                    [point_in_file.col, point_in_file.row],
                    buf.get_contents(),
                    size_row,
                    size_column,
                )
                .unwrap_or_else(|x| close_terminal(x, out));
            is_required_update = false;
        }
        execute!(out, MoveTo(point.col, point.row))
            .unwrap_or_else(|_| close_terminal("[E101] failed to move cursor".to_string(), out));

        let input = match event::read().unwrap() {
            Event::Key(event) => event,
            _ => KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
        };
        let code = input.code;
        let mode = state.check_mode();

        match mode {
            MODE::NORMAL => {
                let new_mode = match code {
                    KeyCode::Char(c) => match c {
                        ':' => MODE::COMMAND,
                        'i' => {
                            let ret = if state.get_read_only() {
                                mode
                            } else {
                                execute!(out, SetCursorStyle::BlinkingBar)
                                    .unwrap_or_else(|x| close_terminal(x.to_string(), out));
                                MODE::INSERT
                            };
                            ret
                        }
                        'v' => MODE::VISUAL,
                        'j' => {
                            if buf.get_row_length() <= point.row + point_in_file.row + 1 {
                            } else if size_row > point.row + 2 {
                                point.row = point.row + 1;
                                if point.col > buf.get_col_length(point.row + point_in_file.row) {
                                    point.col = buf.get_col_length(point.row + point_in_file.row)
                                } else if pos_tmp
                                    < buf.get_col_length(point.row + point_in_file.row)
                                {
                                    point.col = pos_tmp;
                                } else {
                                    point.col = buf.get_col_length(point.row + point_in_file.row);
                                }
                            } else {
                                {
                                    point_in_file.row += 1;
                                    execute!(out, ScrollUp(1))
                                        .unwrap_or_else(|x| close_terminal(x.to_string(), out));
                                    is_required_update = true;
                                }
                                if pos_tmp < buf.get_col_length(point.row + point_in_file.row) {
                                    point.col = pos_tmp;
                                } else {
                                    point.col = buf.get_col_length(point.row + point_in_file.row);
                                }
                            }
                            MODE::NORMAL
                        }
                        'k' => {
                            if point.row > 0 {
                                point.row = point.row - 1;
                                if point.col > buf.get_col_length(point.row + point_in_file.row) {
                                    point.col = buf.get_col_length(point.row + point_in_file.row)
                                } else if pos_tmp
                                    < buf.get_col_length(point.row + point_in_file.row)
                                {
                                    point.col = pos_tmp;
                                } else {
                                    point.col = buf.get_col_length(point.row + point_in_file.row);
                                }
                            } else {
                                if point_in_file.row > 0 {
                                    point_in_file.row -= 1;
                                    execute!(out, ScrollDown(1))
                                        .unwrap_or_else(|e| close_terminal(e.to_string(), out));
                                    is_required_update = true;
                                }
                                if pos_tmp < buf.get_col_length(point.row + point_in_file.row) {
                                    point.col = pos_tmp;
                                } else {
                                    point.col = buf.get_col_length(point.row + point_in_file.row);
                                }
                            }
                            MODE::NORMAL
                        }
                        'h' => {
                            if point.col > 0 {
                                point.col = point.col - 1;
                                pos_tmp = point.col;
                            }
                            MODE::NORMAL
                        }
                        'l' => {
                            if buf.get_col_length(point.row + point_in_file.row) <= point.col {
                            } else {
                                point.col = point.col + 1;
                                pos_tmp = point.col;
                            }
                            MODE::NORMAL
                        }
                        'x' => {
                            buf.update_contents(delback(
                                point.col,
                                point.row + point_in_file.row,
                                buf.get_contents(),
                            ));
                            MODE::NORMAL
                        }

                        _ => MODE::NORMAL,
                    },
                    _ => MODE::NORMAL,
                };
                state.change_mode(new_mode);
            }
            MODE::INSERT => {
                match code {
                    KeyCode::Esc => {
                        execute!(out, SetCursorStyle::SteadyBlock)
                            .unwrap_or_else(|x| close_terminal(x.to_string(), out));
                        state.change_mode(MODE::NORMAL);
                    }
                    KeyCode::Enter => {
                        buf.update_contents(insert(
                            point.col,
                            point.row + point_in_file.row,
                            buf.get_contents(),
                            '\n',
                        ));
                        point.col = 0;
                        point.row = point.row + 1;
                        is_required_update = true;
                    }
                    KeyCode::Char(c) => {
                        buf.update_contents(insert(
                            point.col,
                            point.row + point_in_file.row,
                            buf.get_contents(),
                            c,
                        ));
                        point.col = point.col + 1;
                        is_required_update = true;
                    }
                    KeyCode::Backspace => {
                        if point.col <= 0 {
                            if point.row > 0 {
                                point.row = point.row - 1;
                                point.col = buf.get_col_length(point.row + point_in_file.row);
                                buf.update_contents(delback(
                                    point.col,
                                    point.row + point_in_file.row,
                                    buf.get_contents(),
                                ));
                            }
                        } else {
                            point.col = point.col - 1;
                            buf.update_contents(delback(
                                point.col,
                                point.row + point_in_file.row,
                                buf.get_contents(),
                            ));
                        }
                        is_required_update = true;
                    }
                    _ => (),
                };
            }
            MODE::COMMAND => match code {
                KeyCode::Char(c) => match c {
                    'q' => break,
                    'w' => {
                        buf.save_file().unwrap_or_else(|x| close_terminal(x, out));
                        state.change_mode(MODE::NORMAL);
                    }
                    _ => state.change_mode(MODE::NORMAL),
                },
                KeyCode::Esc => state.change_mode(MODE::NORMAL),
                _ => (),
            },
            MODE::VISUAL => {
                state.change_mode(MODE::NORMAL);
            }
        }
    }
}
