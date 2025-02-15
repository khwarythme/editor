use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use std::path::Path;

pub struct FileBuffer {
    contents: String,
    is_read_only: bool,
    path: String,
}

impl FileBuffer {
    pub fn new(path: &Path) -> Result<Self, String> {
        let mut f = match File::open(path) {
            Ok(some) => some,
            Err(_) => {
                let mut f = match File::create(path) {
                    Ok(t) => t,
                    Err(e) => return Err(e.to_string()),
                };
                let _result = match f.write_all("".as_bytes()) {
                    Ok(_) => "Ok",
                    Err(e) => return Err(e.to_string()),
                };
                let f = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => return Err(e.to_string()),
                };
                f
            }
        };
        let mut buf = Vec::new();
        match f.read_to_end(&mut buf) {
            Ok(_) => Ok(FileBuffer {
                contents: String::from_utf8(buf).unwrap_or(String::from("")),
                is_read_only: false,
                path: String::from(path.to_str().unwrap_or("")),
            }),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn get_contents(&self) -> String {
        String::from(self.contents.as_str())
    }
    pub fn update_contents(&mut self, new_contents: String) {
        self.contents = String::from(new_contents);
    }
    pub fn save_file(&mut self) -> Result<(), String> {
        let file = match File::create(Path::new(self.path.as_str())) {
            Ok(some) => some,
            Err(e) => return Err(e.to_string()),
        };

        let mut writer = BufWriter::new(file);
        let _result = match writer.write_all(self.contents.as_bytes()) {
            Ok(_) => "Ok",
            Err(e) => return Err(e.to_string()),
        };
        let _result = match writer.flush() {
            Ok(_) => "Ok",
            Err(e) => return Err(e.to_string()),
        };
        Ok(())
    }
    pub fn get_read_only(&self) -> bool {
        self.is_read_only
    }
    pub fn get_col_length(&self, row: u16) -> u16 {
        let mut row_count = 0;
        if self.contents.len() < 1 {
            return 0;
        } else {
            for col in self.contents.split('\n') {
                if row_count == row {
                    return col.len() as u16;
                }
                row_count = row_count + 1;
            }
            return 0;
        };
    }
    pub fn get_row_length(&self) -> u16 {
        let v: Vec<&str> = self.contents.split('\n').collect();
        v.len() as u16
    }
}
