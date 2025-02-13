use crossterm::terminal::size;
use std::io::{prelude::*, Stdout};
use std::io::{stdout, BufWriter};

pub struct Display {
    buffer: BufWriter<Stdout>,
}

impl Display {
    pub fn update(
        &mut self,
        point: [u16; 2],
        content: &String,
        wsize_row: u16,
        wsize_column: u16,
    ) -> Result<(), String> {
        let mut row_index = 0;
        let tmp_content = String::from(content);
        for chara in tmp_content.split('\n') {
            if row_index > wsize_row - 2 + point[1] {
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
        if row_index - point[1] < wsize_row - 2 {
            while row_index - point[1] < wsize_row - 2 {
                self.buffer.write("~\r\n".as_bytes()).unwrap();
                row_index += 1;
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
