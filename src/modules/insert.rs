use crate::modules::mode::MODE;
use crate::modules::show::Display;
use crate::modules::show::MoveDirection;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::KeyCode;

use super::file::FileBuffer;

/// insert a charactor on a point.
pub fn insert(col: u16, row: u16, base_string: String, charactor: char) -> String {
    // create copy string
    let tmp = String::from(base_string);
    let mut count = 0;
    let mut result = String::new();
    let mut first = true;
    // move to target row
    for content in tmp.split('\n') {
        if first {
            first = false;
        } else {
            result.push_str("\n");
        }
        let mut tmpstring = format!("{}", content);
        if count == row {
            tmpstring.insert(col as usize, charactor);
        }
        result.push_str(&tmpstring);
        count += 1;
    }
    String::from(result)
}
pub fn delback(col: u16, row: u16, base_string: String) -> String {
    let tmp = String::from(base_string);
    let mut count = 0;
    let mut after = String::new();
    let mut cr = true;
    // move to target row
    for content in tmp.split('\n') {
        if cr {
            cr = false;
        } else {
            after.push_str("\n");
        }
        let mut tmpstring = format!("{}", content);
        if count == row {
            if col < tmpstring.len() as u16 {
                tmpstring.remove((col) as usize);
            } else {
                cr = true;
            }
        }
        after.push_str(&tmpstring);
        count += 1;
    }
    String::from(after)
}
pub fn proc_insert(code: KeyCode, display: &mut Display, buf: &mut FileBuffer) -> MODE {
    match code {
        KeyCode::Esc => {
            display.set_cursor_type(SetCursorStyle::SteadyBlock);
            MODE::Normal
        }
        KeyCode::Enter => {
            buf.update_contents(insert(
                display.get_cursor_coordinate_in_file().col,
                display.get_cursor_coordinate_in_file().row,
                buf.get_contents(),
                '\n',
            ));
            display.move_cursor_nextpos(MoveDirection::Down, &buf);
            display.update(buf.get_contents()).unwrap();
            MODE::Insert
        }
        KeyCode::Char(c) => {
            buf.update_contents(insert(
                display.get_cursor_coordinate_in_file().col,
                display.get_cursor_coordinate_in_file().row,
                buf.get_contents(),
                c,
            ));
            display.move_cursor_nextpos(MoveDirection::Right, &buf);
            display.update(buf.get_contents()).unwrap();
            MODE::Insert
        }
        KeyCode::Backspace => {
            if display.get_cursor_coordinate_in_file().col <= 0 {
                let ret = if display.get_cursor_coordinate().col > 0 {
                    display.move_cursor_nextpos(MoveDirection::Up, &buf);
                    buf.update_contents(delback(
                        display.get_cursor_coordinate_in_file().col,
                        display.get_cursor_coordinate_in_file().row,
                        buf.get_contents(),
                    ));
                } else {
                };
                ret
            } else {
                display.move_cursor_nextpos(MoveDirection::Left, &buf);
                buf.update_contents(delback(
                    display.get_cursor_coordinate_in_file().col,
                    display.get_cursor_coordinate_in_file().row,
                    buf.get_contents(),
                ));
            };
            display.update(buf.get_contents()).unwrap();
            MODE::Insert
        }
        _ => MODE::Insert,
    }
}
