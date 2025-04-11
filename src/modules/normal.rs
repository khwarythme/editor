use super::edit::Undo;
use super::edit::UndoDirection;
use super::edit::Yank;
use super::edit::{del, insert};
use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crate::modules::mode::MODE;
use crate::modules::show::{Display, MoveDirection};
use crossterm::cursor::SetCursorStyle;
use crossterm::event::KeyCode;

use super::history::Operation;

#[derive(Debug)]
pub struct Normal {}

impl Normal {
    pub async fn proc_normal(
        code: KeyCode,
        display: &mut Display,
        buf: &mut FileBuffer,
        undo: &mut Undo,
        yank: &mut Yank,
    ) -> MODE {
        match code {
            KeyCode::Char(c) => match c {
                ':' => MODE::Command,
                'i' => {
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'I' => {
                    display.move_cursor_nextpos(MoveDirection::Head, buf).await;
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'a' => {
                    display.move_cursor_nextpos(MoveDirection::Right, buf).await;
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'A' => {
                    display.move_cursor_nextpos(MoveDirection::Tail, buf).await;
                    display.set_cursor_type(SetCursorStyle::BlinkingBar);
                    MODE::Insert
                }
                'v' => MODE::Visual,
                'j' => {
                    display.move_cursor_nextpos(MoveDirection::Down, &buf).await;
                    MODE::Normal
                }
                'k' => {
                    display.move_cursor_nextpos(MoveDirection::Up, &buf).await;
                    MODE::Normal
                }
                'h' => {
                    display.move_cursor_nextpos(MoveDirection::Left, &buf).await;
                    MODE::Normal
                }
                'l' => {
                    display.move_cursor_nextpos(MoveDirection::Right, &buf).await;
                    MODE::Normal
                }
                'x' => {
                    let col = display.get_cursor_coordinate_in_file().column;
                    let row = display.get_cursor_coordinate_in_file().row;
                    let (result, delchar) = del(
                        display.get_cursor_coordinate_in_file(),
                        buf.get_contents(),
                        1,
                    ).await;
                    buf.update_contents(result);
                    undo.add_do_history(Operation::DELETE, delchar, Point { column: col, row });
                    //display.update_all(buf.get_contents()).await.unwrap();
                    MODE::Normal
                }
                'u' => {
                    let (ret, pos) = undo.undo(buf.get_contents(), UndoDirection::Undo).await;
                    buf.update_contents(ret);
                    //display.update_all(buf.get_contents()).await.unwrap();
                    display.move_to_point(
                        buf,
                        Point {
                            column: pos.column,
                            row: pos.row,
                        },
                    ).await;

                    MODE::Normal
                }
                'r' => {
                    let (ret, pos) = undo.undo(buf.get_contents(), UndoDirection::Redo).await;
                    buf.update_contents(ret);
                    //display.update_all(buf.get_contents()).await.unwrap();
                    display.move_to_point(
                        buf,
                        Point {
                            column: pos.column,
                            row: pos.row,
                        },
                    ).await;

                    MODE::Normal
                }
                '/' => MODE::Search,
                'n' => {
                    match buf.get_next_searchresult() {
                        Some(point) => display.move_to_point(buf, point).await,
                        None => (),
                    };
                    MODE::Normal
                }
                'y' => {
                    yank.yank(buf.get_contents(), display.get_cursor_coordinate_in_file());
                    MODE::Normal
                }
                'p' => {
                    buf.update_contents(yank.past(
                        display.get_cursor_coordinate_in_file().row,
                        buf.get_contents(),
                        undo,
                    ).await);
                    display.move_cursor_nextpos(MoveDirection::Down, buf).await;
                    //let _ = display.update_all(buf.get_contents());
                    MODE::Normal
                }

                _ => MODE::Normal,
            },
            _ => MODE::Normal,
        }
    }
}
