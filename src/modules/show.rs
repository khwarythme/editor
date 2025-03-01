use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crossterm::cursor::MoveTo;
use crossterm::cursor::{self, SetCursorStyle};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::terminal::{Clear, ClearType};
use crossterm::terminal::{ScrollDown, ScrollUp};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{stdout, Stdout};

pub struct Display {
    buffer: BufWriter<Stdout>,
    point: Point,
    point_in_file: Point,
    wsize: Point,
    pos_tmp: Point,
    out: Stdout,
}
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
    Head,
    Tail,
}

impl Display {
    fn update_line(&mut self, content: VecDeque<VecDeque<char>>, row: usize) {
        let tmp_cursor_pos = row - self.point_in_file.row;
        let _ = queue!(self.out, MoveTo(0, tmp_cursor_pos as u16));
        let tmp_content = content.clone();
        let printstring = tmp_content.into_iter().nth(row as usize).unwrap();
        let printstring: String = printstring.into_iter().collect();

        let _ = queue!(self.out, Print(printstring));
        let _ = queue!(
            self.out,
            MoveTo(self.point.column as u16, self.point.row as u16)
        );
    }
    pub fn update_all(&mut self, content: VecDeque<VecDeque<char>>) -> Result<(), String> {
        let mut row_index = 0;
        let tmp_content = content.clone();
        queue!(self.out, Clear(ClearType::All))
            .unwrap_or_else(|e| self.close_terminal(e.to_string()));
        queue!(self.out, MoveTo(0, 0))
            .unwrap_or_else(|_| self.close_terminal("[E101] failed to move cursor".to_string()));

        for chara in tmp_content {
            row_index += 1;
            if row_index > self.wsize.row - 2 + self.point_in_file.row {
                break;
            }
            if self.point_in_file.row > row_index {
                continue;
            } else {
                for c in chara {
                    let _ = queue!(self.out, Print(c));
                }
                let _ = queue!(self.out, Print("\r\n"));
            }
        }
        if (row_index - self.point_in_file.row) < self.wsize.row - 2 {
            while row_index - self.point_in_file.row < self.wsize.row - 2 {
                let _ = queue!(self.out, Print("~\r\n"));
                row_index += 1;
            }
        }
        let _ = queue!(
            self.out,
            MoveTo(self.point.column as u16, self.point.row as u16)
        );
        self.out.flush().unwrap();
        Ok(())
    }
    pub fn update_wsize(&mut self, size: Point) {
        self.wsize = size;
    }
    pub fn new(size: Point) -> Display {
        Display {
            buffer: BufWriter::new(stdout()),
            point: Point { column: 0, row: 0 },
            point_in_file: Point { column: 0, row: 0 },
            wsize: size,
            pos_tmp: Point { column: 0, row: 0 },
            out: stdout(),
        }
    }
    pub fn move_to_point(&mut self, buf: &mut FileBuffer, point: Point) {
        if point.row <= self.wsize.row / 2 {
            self.point_in_file.row = 0;
        } else {
            self.point_in_file.row = point.row - self.wsize.row / 2;
        }
        self.point.row = if point.row <= self.wsize.row / 2 {
            point.row % self.wsize.row
        } else {
            self.wsize.row / 2
        };
        self.point.column = point.column;
        self.update_all(buf.get_contents()).unwrap();
        let _ = queue!(
            self.out,
            MoveTo(self.point.column as u16, self.point.row as u16)
        );
        let _ = self.out.flush();
    }
    pub fn move_cursor_to_point(&mut self, point: Point) {
        queue!(self.out, MoveTo(point.column as u16, point.row as u16)).unwrap();
    }
    pub fn move_cursor_nextpos(&mut self, direction: MoveDirection, buf: &FileBuffer) {
        match direction {
            MoveDirection::Down => {
                if buf.get_row_length() <= self.point.row as u16 + self.point_in_file.row as u16 + 1
                {
                } else if self.wsize.row > self.point.row + 2 {
                    self.point.row = self.point.row + 1;
                } else {
                    queue!(self.out, ScrollUp(1)).unwrap();
                    self.point_in_file.row += 1;
                    self.update_line(buf.get_contents(), self.get_cursor_coordinate_in_file().row);
                }
                self.pos_tmp.row = self.point_in_file.row + self.point.row;
                let col_length = buf.get_col_length(self.point.row + self.point_in_file.row);
                if self.pos_tmp.column > col_length {
                    self.point.column = col_length;
                } else {
                    self.point.column = self.pos_tmp.column;
                }
            }
            MoveDirection::Up => {
                if self.point.row > 0 {
                    self.point.row = self.point.row - 1;
                } else {
                    if self.point_in_file.row > 0 {
                        queue!(self.out, ScrollDown(1)).unwrap();
                        self.point_in_file.row -= 1;
                        self.update_line(
                            buf.get_contents(),
                            self.get_cursor_coordinate_in_file().row,
                        );
                    }
                }
                self.pos_tmp.row = self.point_in_file.row + self.point.row;
                let col_length = buf.get_col_length(self.point.row + self.point_in_file.row);
                if self.pos_tmp.column > col_length {
                    self.point.column = col_length;
                } else {
                    self.point.column = self.pos_tmp.column;
                }
            }
            MoveDirection::Left => {
                if self.point.column > 0 {
                    self.point.column = self.point.column - 1;
                    self.pos_tmp.column = self.point.column;
                }
            }
            MoveDirection::Right => {
                if buf.get_col_length(self.point.row + self.point_in_file.row)
                    <= std::cmp::min(self.point.column, self.wsize.column)
                {
                } else {
                    self.point.column += 1;
                    self.pos_tmp.column = self.point.column;
                }
            }
            MoveDirection::Head => self.point.column = 0,
            MoveDirection::Tail => {
                self.point.column = buf.get_col_length(self.get_cursor_coordinate_in_file().row)
            }
        }
        let _ = queue!(
            self.out,
            MoveTo(self.point.column as u16, self.point.row as u16)
        );
        let _ = self.out.flush();
    }
    pub fn get_cursor_coordinate_in_file(&self) -> Point {
        Point {
            column: self.point.column + self.point_in_file.column,
            row: self.point.row + self.point_in_file.row,
        }
    }
    pub fn get_cursor_coordinate(&self) -> Point {
        Point {
            column: self.point.column,
            row: self.point.row,
        }
    }
    pub fn set_cursor_type(&mut self, style: SetCursorStyle) {
        queue!(self.out, style).unwrap();
    }
    pub fn init_window(&mut self) {
        queue!(
            self.out,
            cursor::Show,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(self.point.column as u16, self.point.row as u16)
        )
        .expect("Failed to open alternate screen");
        enable_raw_mode().expect("Failed to open raw mode");
        self.out.flush().unwrap();
    }
    pub fn close_terminal(&mut self, err: String) {
        print!("{}", err);
        queue!(self.out, cursor::Show, LeaveAlternateScreen,)
            .expect("failed to close alternate screen");
        disable_raw_mode().expect("");
        self.out.flush().unwrap();
    }
    pub fn update_info_line(&mut self, msg: &String) {
        let cursor_pos = self.point;
        self.move_cursor_to_point(Point {
            column: 0,
            row: self.wsize.row,
        });
        println!("{}", msg);
        self.move_cursor_to_point(cursor_pos);
    }
}
