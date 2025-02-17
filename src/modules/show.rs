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
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{stdout, Stdout};
use std::thread::ScopedJoinHandle;

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
    fn update_line(&mut self, content: String, row: u16) {
        let tmp_cursor_pos = row - self.point_in_file.row;
        queue!(self.out, MoveTo(0, tmp_cursor_pos));
        let tmp_content = String::from(content);
        let printstring = tmp_content
            .split('\n')
            .into_iter()
            .nth(row as usize)
            .unwrap();
        let printstring = format!("{}\n", printstring);

        queue!(self.out, Print(printstring));
        queue!(self.out, MoveTo(self.point.col, self.point.row));
    }
    pub fn update_all(&mut self, content: String) -> Result<(), String> {
        let mut row_index = 0;
        let tmp_content = String::from(content);
        queue!(self.out, Clear(ClearType::All))
            .unwrap_or_else(|e| self.close_terminal(e.to_string()));
        queue!(self.out, MoveTo(0, 0))
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
        self.out.flush().unwrap();
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
        queue!(self.out, MoveTo(point.col, point.row)).unwrap();
        self.out.flush();
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
                    queue!(self.out, ScrollUp(1)).unwrap();
                    self.point_in_file.row += 1;
                    self.update_line(buf.get_contents(), self.get_cursor_coordinate_in_file().row);
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
                        queue!(self.out, ScrollDown(1)).unwrap();
                        self.point_in_file.row -= 1;
                        self.update_line(
                            buf.get_contents(),
                            self.get_cursor_coordinate_in_file().row,
                        );
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
            MoveDirection::Head => self.point.col = 0,
            MoveDirection::Tail => {
                self.point.col = buf.get_col_length(self.get_cursor_coordinate_in_file().row)
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
        queue!(self.out, style).unwrap();
    }
    pub fn init_window(&mut self) {
        queue!(
            self.out,
            cursor::Show,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(self.point.col, self.point.row)
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
            col: 0,
            row: self.wsize.row,
        });
        println!("{}", msg);
        self.move_cursor_to_point(cursor_pos);
    }
}
