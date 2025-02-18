use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crate::modules::insert::delback;
use crate::modules::mode::MODE;
use crate::modules::show::{Display, MoveDirection};
use crate::modules::undo::Undo;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::KeyCode;

use super::history::Operation;

pub struct Normal {}

impl Normal {
    pub fn proc_normal(
        code: KeyCode,
        display: &mut Display,
        buf: &mut FileBuffer,
        undo: &mut Undo,
    ) -> MODE {
        match code {
            KeyCode::Char(c) => match c {
                ':' => MODE::Command,
                'i' => {
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'I' => {
                    display.move_cursor_nextpos(MoveDirection::Head, buf);
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'a' => {
                    display.move_cursor_nextpos(MoveDirection::Right, buf);
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'A' => {
                    display.move_cursor_nextpos(MoveDirection::Tail, buf);
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'v' => MODE::Visual,
                'j' => {
                    display.move_cursor_nextpos(MoveDirection::Down, &buf);
                    MODE::Normal
                }
                'k' => {
                    display.move_cursor_nextpos(MoveDirection::Up, &buf);
                    MODE::Normal
                }
                'h' => {
                    display.move_cursor_nextpos(MoveDirection::Left, buf);
                    MODE::Normal
                }
                'l' => {
                    display.move_cursor_nextpos(MoveDirection::Right, buf);
                    MODE::Normal
                }
                'x' => {
                    let col = display.get_cursor_coordinate_in_file().col;
                    let row = display.get_cursor_coordinate_in_file().row;
                    let (result, delchar) = delback(
                        display.get_cursor_coordinate_in_file().col,
                        display.get_cursor_coordinate_in_file().row,
                        buf.get_contents(),
                    );
                    buf.update_contents(result);
                    undo.add_do_history(Operation::DELETE, delchar, [col as u32, row as u32]);
                    display.update_all(buf.get_contents()).unwrap();
                    MODE::Normal
                }
                'u' => {
                    let pos = undo.undo(buf);
                    display.update_all(buf.get_contents()).unwrap();
                    display.move_to_point(buf,Point {
                        col: pos[0] as u16,
                        row: pos[1] as u16,
                    });
                    MODE::Normal
                }
                '/' => MODE::Search,
                'n' => {
                    match buf.get_next_searchresult() {
                        Some(point) => display.move_to_point(buf, point),
                        None => (),
                    };
                    MODE::Normal
                }

                _ => MODE::Normal,
            },
            _ => MODE::Normal,
        }
    }
}
