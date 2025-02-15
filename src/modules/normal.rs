use crate::modules::file::FileBuffer;
use crate::modules::insert::delback;
use crate::modules::mode::MODE;
use crate::modules::show::{Display, MoveDirection};
use crossterm::cursor::SetCursorStyle;
use crossterm::event::KeyCode;

pub struct Normal {}

impl Normal {
    pub fn proc_normal(code: KeyCode, display: &mut Display, buf: &mut FileBuffer) -> MODE {
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
                    buf.update_contents(delback(
                        display.get_cursor_coordinate_in_file().col,
                        display.get_cursor_coordinate_in_file().row,
                        buf.get_contents(),
                    ));
                    display.update(buf.get_contents()).unwrap();
                    MODE::Normal
                }

                _ => MODE::Normal,
            },
            _ => MODE::Normal,
        }
    }
}
