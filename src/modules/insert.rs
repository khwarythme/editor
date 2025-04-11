use std::collections::VecDeque;

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
pub async fn proc_insert(
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
                    column: display.get_cursor_coordinate_in_file().column,
                    row: display.get_cursor_coordinate_in_file().row,
                },
                buf.get_contents(),
                VecDeque::from(['\n']),
            ).await);
            undo.add_do_history(
                Operation::ADD,
                VecDeque::from(['\n']),
                Point {
                    column: display.get_cursor_coordinate_in_file().column,
                    row: display.get_cursor_coordinate_in_file().row,
                },
            );
            display.move_cursor_nextpos(MoveDirection::Down, &buf).await;
            display.move_cursor_nextpos(MoveDirection::Head, &buf).await;
            //display.update_all(buf.get_contents()).await.unwrap();
            MODE::Insert
        }
        KeyCode::Char(c) => {
            buf.update_contents(insert(
                Point {
                    column: display.get_cursor_coordinate_in_file().column,
                    row: display.get_cursor_coordinate_in_file().row,
                },
                buf.get_contents(),
                VecDeque::from([c]),
            ).await);
            undo.add_do_history(
                Operation::ADD,
                VecDeque::from([c]),
                Point {
                    column: display.get_cursor_coordinate_in_file().column,
                    row: display.get_cursor_coordinate_in_file().row,
                },
            );
            display.move_cursor_nextpos(MoveDirection::Right, &buf).await;
            //display.update_all(buf.get_contents()).await.unwrap();
            MODE::Insert
        }
        KeyCode::Backspace => {
            let _tmp_pos = display.get_cursor_coordinate_in_file();
            if display.get_cursor_coordinate_in_file().column <= 0 {
                if display.get_cursor_coordinate().row > 0 {
                    display.move_cursor_nextpos(MoveDirection::Up, &buf).await;
                    display.move_cursor_nextpos(MoveDirection::Tail, &buf).await;
                    let (result, delchar) = del(
                        display.get_cursor_coordinate_in_file(),
                        buf.get_contents(),
                        1,
                    ).await;
                    buf.update_contents(result);
                    undo.add_do_history(
                        Operation::DELETE,
                        delchar,
                        display.get_cursor_coordinate_in_file(),
                    );
                    //display.update_all(buf.get_contents()).await.unwrap();
                } else {
                };
            } else {
                display.move_cursor_nextpos(MoveDirection::Left, &buf).await;
                let (result, delchar) = del(
                    display.get_cursor_coordinate_in_file(),
                    buf.get_contents(),
                    1,
                ).await;
                buf.update_contents(result);
                undo.add_do_history(
                    Operation::DELETE,
                    delchar,
                    display.get_cursor_coordinate_in_file(),
                );
                //display.update_all(buf.get_contents()).await.unwrap();
            };
            MODE::Insert
        }
        _ => MODE::Insert,
    }
}
