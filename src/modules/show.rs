use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crossterm::cursor::MoveTo;
use crossterm::cursor::{self, SetCursorStyle};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::terminal::{Clear, ClearType};
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
}

impl Display {
    pub fn update(&mut self, content: String) -> Result<(), String> {
        let mut row_index = 0;
        let tmp_content = String::from(content);
        execute!(self.out, Clear(ClearType::All))
            .unwrap_or_else(|e| self.close_terminal(e.to_string()));
        execute!(self.out, MoveTo(0, 0))
            .unwrap_or_else(|_| self.close_terminal("[E101] failed to move cursor".to_string()));
        for chara in tmp_content.split('\n') {
            if row_index > self.wsize.row - 2 + self.point_in_file.row {
                break;
            }
            if self.point_in_file.row > row_index {
                row_index += 1;
                continue;
            } else {
                row_index += 1;
                self.buffer.write(chara.as_bytes()).unwrap();
                self.buffer.write("\r\n".as_bytes()).unwrap();
            }
        }
        if row_index - self.point_in_file.row < self.wsize.row - 2 {
            while row_index - self.point_in_file.row < self.wsize.row - 2 {
                self.buffer.write("~\r\n".as_bytes()).unwrap();
                row_index += 1;
            }
        }
        self.buffer.flush().unwrap();
        self.move_cursor_to_point(self.point);
        Ok(())
    }
    pub fn update_wsize(&mut self, size: Point) {
        self.wsize = size;
    }
    pub fn new(size: Point) -> Display {
        Display {
            buffer: BufWriter::new(stdout()),
            point: Point { col: 0, row: 0 },
            point_in_file: Point { col: 0, row: 0 },
            wsize: size,
            pos_tmp: Point { col: 0, row: 0 },
            out: stdout(),
        }
    }
    pub fn move_cursor_to_point(&mut self, point: Point) {
        execute!(self.out, MoveTo(point.col, point.row)).unwrap();
    }
    pub fn move_cursor_nextpos(&mut self, direction: MoveDirection, buf: &FileBuffer) {
        match direction {
            MoveDirection::Down => {
                if buf.get_row_length() <= self.point.row + self.point_in_file.row + 1 {
                } else if self.wsize.row > self.point.row + 2 {
                    self.point.row = self.point.row + 1;
                    if self.point.col > buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = buf.get_col_length(self.point.row + self.point_in_file.row)
                    } else if self.pos_tmp.col
                        < buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = self.pos_tmp.col;
                    } else {
                        self.point.col =
                            buf.get_col_length(self.point.row + self.point_in_file.row);
                    }
                } else {
                    self.point_in_file.row += 1;
                    self.update(buf.get_contents()).unwrap();
                    if self.pos_tmp.col
                        < buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = self.pos_tmp.col;
                    } else {
                        self.point.col =
                            buf.get_col_length(self.point.row + self.point_in_file.row);
                    }
                }
            }
            MoveDirection::Up => {
                if self.point.row > 0 {
                    self.point.row = self.point.row - 1;
                    if self.point.col > buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = buf.get_col_length(self.point.row + self.point_in_file.row)
                    } else if self.pos_tmp.col
                        < buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = self.pos_tmp.col;
                    } else {
                        self.point.col =
                            buf.get_col_length(self.point.row + self.point_in_file.row);
                    }
                } else {
                    if self.point_in_file.row > 0 {
                        self.point_in_file.row -= 1;
                        self.update(buf.get_contents()).unwrap();
                    }
                    if self.pos_tmp.col
                        < buf.get_col_length(self.point.row + self.point_in_file.row)
                    {
                        self.point.col = self.pos_tmp.col;
                    } else {
                        self.point.col =
                            buf.get_col_length(self.point.row + self.point_in_file.row);
                    }
                }
            }
            MoveDirection::Left => {
                if self.point.col > 0 {
                    self.point.col = self.point.col - 1;
                    self.pos_tmp.col = self.point.col;
                }
            }
            MoveDirection::Right => {
                if buf.get_col_length(self.point.row + self.point_in_file.row) <= self.point.col {
                } else {
                    self.point.col = self.point.col + 1;
                    self.pos_tmp.col = self.point.col;
                }
            }
        }
        self.move_cursor_to_point(self.point);
    }
    pub fn get_cursor_coordinate_in_file(&self) -> Point {
        Point {
            col: self.point.col + self.point_in_file.col,
            row: self.point.row + self.point_in_file.row,
        }
    }
    pub fn get_cursor_coordinate(&self) -> Point {
        Point {
            col: self.point.col,
            row: self.point.row,
        }
    }
    pub fn set_cursor_type(&mut self, style: SetCursorStyle) {
        execute!(self.out, style).unwrap();
    }
    pub fn init_window(&mut self) {
        execute!(
            self.out,
            cursor::Show,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(self.point.col, self.point.row)
        )
        .expect("Failed to open alternate screen");
        enable_raw_mode().expect("Failed to open raw mode");
    }
    pub fn close_terminal(&mut self, err: String) {
        print!("{}", err);
        execute!(self.out, cursor::Show, LeaveAlternateScreen,)
            .expect("failed to close alternate screen");
        disable_raw_mode().expect("");
    }
}
