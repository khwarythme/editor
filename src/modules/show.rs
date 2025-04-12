use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use super::control_server::Operation;

use crossterm::cursor::MoveTo;
use crossterm::cursor::{self, SetCursorStyle};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::style::*;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::terminal;
use crossterm::terminal::{Clear, ClearType};
use crossterm::terminal::{ScrollDown, ScrollUp};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{stdout, Stdout};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender,Receiver};


const COLUMN_LEFT_LIMIT: u16 = 5;

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
    fn calc_char_disp_length(&self,content:VecDeque<char>, column:u16) -> usize{
        let mut ret:usize = 0;
        for i in 0..column as usize{
            ret += match content[i] {
                c if c == 0x09 as char => 8,
                c if c < 0x7f as char => 1,
                _ => 2
            };
        }
        ret

    }
    pub async fn thread_main(&mut self) -> !{

            println!("hello thread");
            loop{};
    }
    fn update_line(&mut self, content: VecDeque<VecDeque<char>>, row: usize) {
        let tmp_cursor_pos = row - self.point_in_file.row;
        let _ = queue!(self.out, MoveTo(0, tmp_cursor_pos as u16));
        
        let tmp_content = content.clone();
        let printstring = tmp_content.into_iter().nth(row as usize).unwrap();
        let printstring: String = printstring.into_iter().collect();

        let linno = format!("{:4} \r\n", row + 1);
        let _ = queue!(self.out, MoveTo(0,self.point.row as u16), Print(linno));
        let _ = self.out.flush();

        let _ = queue!(self.out, MoveTo(COLUMN_LEFT_LIMIT,self.point.row as u16), Print(printstring));
        let _ = queue!(
            self.out,
            MoveTo(
                self.point.column as u16 + COLUMN_LEFT_LIMIT,
                self.point.row as u16
            )
        );
    }

    pub async fn update_all(&mut self, content: Arc<Mutex<VecDeque<VecDeque<char>>>>) -> Result<(), String> {
        let mut row_index = 0;
        let tmp_content = content.lock().unwrap();
        queue!(self.out, Clear(ClearType::All))
            .unwrap_or_else(|e| self.close_terminal(e.to_string()));
        queue!(self.out, MoveTo(0, 0))
            .unwrap_or_else(|_| self.close_terminal("[E101] failed to move cursor".to_string()));
        if tmp_content.len() < 1 {
            let linno = format!("{:4} \r\n", row_index + 1);
            let _ = queue!(self.out, Print(linno));
            let _ = self.out.flush();
        }

        for chara in tmp_content.iter() {
            row_index += 1;
            if row_index > self.wsize.row - 2 + self.point_in_file.row {
                break;
            }
            if self.point_in_file.row >= row_index {
                continue;
            } else {
                let linno = format!("{:4} ", row_index);
                let _ = queue!(
                    self.out,
                    SetForegroundColor(Color::Grey),
                    Print(linno),
                    ResetColor
                );
                let mut colcount = 0;
                for c in chara {
                    if colcount >= self.wsize.column - COLUMN_LEFT_LIMIT as usize {
                    } else {
                        let _ = queue!(self.out, Print(c));
                    }
                    colcount += 1;
                }
                let _ = queue!(self.out, Print("\r\n"));
            }
        }
        if (row_index - self.point_in_file.row) < self.wsize.row - 2 {
            while row_index - self.point_in_file.row < self.wsize.row - 1 {
                let _ = queue!(self.out, Print("~\r\n"));
                row_index += 1;
            }
        }
        let _ = queue!(
            self.out,
            MoveTo(
                self.point.column as u16 + COLUMN_LEFT_LIMIT,
                self.point.row as u16
            )
        );
        self.out.flush().unwrap();
        Ok(())
    }
    pub async fn update_wsize(&mut self, size: Point) {
        self.wsize = size;
    }
    pub fn new(size: Point) -> Display {
        Self {
            buffer: BufWriter::new(stdout()),
            point: Point { column: 0, row: 0 },
            point_in_file: Point { column: 0, row: 0 },
            wsize: size,
            pos_tmp: Point { column: 0, row: 0 },
            out: stdout(),
        }
    }
    pub async fn move_to_point(&mut self, buf: &mut FileBuffer, point: Point) {
        if point.row <= self.wsize.row / 2 {
            self.point_in_file.row = 0;
        } else {
            self.point_in_file.row = point.row - (self.wsize.row / 2);
        }
        self.point.row = if point.row <= self.wsize.row / 2 {
            point.row % self.wsize.row
        } else {
            self.wsize.row / 2
        };
        self.point.column = if self.point.column > buf.get_col_length(self.point.row) {
            buf.get_col_length(self.point.row)
        } else {
            point.column
        };
        //self.update_all(buf.get_contents()).await.unwrap();
        let _ = queue!(
            self.out,
            MoveTo(
                self.point.column as u16 + COLUMN_LEFT_LIMIT,
                self.point.row as u16
            )
        );
        let _ = self.out.flush();
    }
    pub async fn move_cursor_to_point(&mut self, point: Point) {
        queue!(self.out, MoveTo(point.column as u16, point.row as u16)).unwrap();
    }
    pub async fn move_cursor_nextpos(&mut self, direction: MoveDirection, buf: &FileBuffer) {
        let _ = queue!(self.out,cursor::Hide);
        match direction {
            MoveDirection::Down => {
                if buf.get_row_length() <= self.point.row as u16 + self.point_in_file.row as u16 + 1
                {
                } else if self.wsize.row > self.point.row + 3 {
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
        let column_pos = self.calc_char_disp_length(buf.get_contents()[self.point_in_file.row + self.point.row].clone(),self.point.column as u16);
        let _ = queue!(
            self.out,
            MoveTo(column_pos as u16 + COLUMN_LEFT_LIMIT, self.point.row as u16),
            cursor::Show
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
    pub async fn update_info_line(&mut self, msg: [&str; 10]) {
        let cursor_pos = self.point;
        let sep10 = self.wsize.column / 10;
        let _ = queue!(self.out,cursor::Hide );
        let _ = queue!(self.out, SetBackgroundColor(Color::White));
        let _ = queue!(self.out, SetForegroundColor(Color::Black));

        self.move_cursor_to_point(Point {
            column: 0,
            row: self.wsize.row + 1,
        }).await;
        let mut tmp = 0;
        for _ in 0..self.wsize.column - 1 {
            print!(" ");
        }
        print!("\r");
        self.move_cursor_to_point(Point {
            column: 0,
            row: self.wsize.row + 1,
        }).await;

        for i in msg {
            print!("{}", i);
            self.move_cursor_to_point(Point {
                column: tmp + sep10,
                row: self.wsize.row + 1,
            }).await;
            tmp += sep10;
        }
        let _ = queue!(self.out, ResetColor);

        self.move_cursor_to_point(Point {
            column: cursor_pos.column + COLUMN_LEFT_LIMIT as usize,
            row: cursor_pos.row,
        }).await;
        let _ = queue!(self.out,cursor::Show );
        let _ = self.out.flush();
    }
}

pub async fn thread_main(contents:Arc<Mutex<VecDeque<VecDeque<char>>>>,tx:Arc<Mutex<Sender<Operation>>>,rx:Receiver<Operation>) -> Result<(),String>{
    let (wcol,wrow) = terminal::size().unwrap();
    let mut display = Display::new(Point{column:wcol as usize, row:wrow as usize});
    display.init_window();
    display.update_all(contents).await?;
    display.close_terminal("".to_string());
    Ok(())
}

