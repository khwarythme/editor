use super::edit::Undo;
use crate::modules::history::Operation;
use crate::modules::mode::MODE;
use crate::modules::show::Display;
use crate::modules::show::MoveDirection;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::KeyCode;

use super::coordinate::Point;
use super::edit::{del, insert};
use super::file::FileBuffer;

/// insert a charactor on a point.
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
                Point {
                    col: display.get_cursor_coordinate_in_file().col,
                    row: display.get_cursor_coordinate_in_file().row,
                },
                buf.get_contents(),
                &vec!['\n'],
            ));
            undo.add_do_history(
                Operation::ADD,
                vec!['\n' as char],
                Point {
                    col: display.get_cursor_coordinate_in_file().col,
                    row: display.get_cursor_coordinate_in_file().row,
                },
            );
            display.move_cursor_nextpos(MoveDirection::Down, &buf);
            display.move_cursor_nextpos(MoveDirection::Head, &buf);
            display.update_all(buf.get_contents()).unwrap();
            MODE::Insert
        }
        KeyCode::Char(c) => {
            buf.update_contents(insert(
                Point {
                    col: display.get_cursor_coordinate_in_file().col,
                    row: display.get_cursor_coordinate_in_file().row,
                },
                buf.get_contents(),
                &vec![c],
            ));
            undo.add_do_history(
                Operation::ADD,
                vec![c as char],
                Point {
                    col: display.get_cursor_coordinate_in_file().col,
                    row: display.get_cursor_coordinate_in_file().row,
                },
            );
            display.move_cursor_nextpos(MoveDirection::Right, &buf);
            display.update_all(buf.get_contents()).unwrap();
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
            let (result, delchar) = del(
                display.get_cursor_coordinate_in_file(),
                buf.get_contents(),
                1,
            );
            buf.update_contents(result);
            undo.add_do_history(
                Operation::DELETE,
                delchar,
                display.get_cursor_coordinate_in_file(),
            );
            display.update_all(buf.get_contents()).unwrap();
            MODE::Insert
        }
        _ => MODE::Insert,
    }
}
