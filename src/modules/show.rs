use crossterm::terminal::{window_size, WindowSize};
use std::io::{prelude::*, Stdout};
use std::io::{stdout, BufWriter};

pub struct Display {
    buffer: BufWriter<Stdout>,
}

impl Display {
    pub fn update(&mut self, point: [u16; 2], content: &String) -> Result<(), String> {
        let mut is_ignore = false;
        let mut col_index = 0;
        let mut row_index = 0;
        let crlf: [u8; 2] = [0x0d, 0x0a];
        let tmp_content = String::from(content);
        let wsize = window_size().expect("");
        if wsize.rows <= 0 || wsize.columns <= 0 {
            return Err("cannot read window infomation".to_string());
        }
        for chara in tmp_content.split('\n') {
            if row_index > wsize.rows - 2 + point[1] {
                break;
            }
            if point[1] > row_index {
                row_index += 1;
                continue;
            } else {
                row_index += 1;
                self.buffer.write(chara.as_bytes()).unwrap();
                self.buffer.write("\r\n".as_bytes()).unwrap();
            }
        }
        self.buffer.flush().unwrap();
        Ok(())
    }
    pub fn new() -> Display {
        Display {
            buffer: BufWriter::new(stdout()),
        }
    }
}
