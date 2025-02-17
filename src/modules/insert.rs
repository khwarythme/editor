use crate::modules::history::Operation;
use crate::modules::mode::MODE;
use crate::modules::show::Display;
use crate::modules::show::MoveDirection;
use crate::modules::undo::Undo;
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
pub fn delback(col: u16, row: u16, base_string: String) -> (String, Vec<char>) {
    let tmp = String::from(base_string);
    let mut count = 0;
    let mut after = String::new();
    let mut cr = true;
    let mut del_char: char = 0x00 as char;
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
                let mut colcount = 0;
                for charactor in tmpstring.chars() {
                    if colcount == col {
                        del_char = charactor;
                    }
                    colcount += 1;
                }
                tmpstring.remove((col) as usize);
            } else {
                del_char = '\n' as char;
                cr = true;
            }
        }
        after.push_str(&tmpstring);
        count += 1;
    }
    (String::from(after), vec![del_char])
}
pub fn proc_insert(
    code: KeyCode,
    display: &mut Display,
    buf: &mut FileBuffer,
    undo: &mut Undo,
) -> MODE {
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
            undo.add_do_history(
                Operation::ADD,
                vec!['\n' as char],
                [
                    display.get_cursor_coordinate_in_file().col as u32,
                    display.get_cursor_coordinate_in_file().row as u32,
                ],
            );
            display.move_cursor_nextpos(MoveDirection::Down, &buf);
            display.move_cursor_nextpos(MoveDirection::Head, &buf);
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
            undo.add_do_history(
                Operation::ADD,
                vec![c as char],
                [
                    display.get_cursor_coordinate_in_file().col as u32,
                    display.get_cursor_coordinate_in_file().row as u32,
                ],
            );
            display.move_cursor_nextpos(MoveDirection::Right, &buf);
            display.update(buf.get_contents()).unwrap();
            MODE::Insert
        }
        KeyCode::Backspace => {
            let tmp_pos = display.get_cursor_coordinate_in_file();
            if display.get_cursor_coordinate_in_file().col <= 0 {
                if display.get_cursor_coordinate().row > 0 {
                    display.move_cursor_nextpos(MoveDirection::Up, &buf);
                    display.move_cursor_nextpos(MoveDirection::Tail, &buf);
                } else {
                };
            } else {
                display.move_cursor_nextpos(MoveDirection::Left, &buf);
            };
            let (result, delchar) = delback(
                display.get_cursor_coordinate_in_file().col,
                display.get_cursor_coordinate_in_file().row,
                buf.get_contents(),
            );
            buf.update_contents(result);
            undo.add_do_history(
                Operation::DELETE,
                delchar,
                [tmp_pos.col as u32, tmp_pos.row as u32],
            );
            display.update(buf.get_contents()).unwrap();
            MODE::Insert
        }
        _ => MODE::Insert,
    }
}
