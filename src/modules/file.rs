use crate::modules::coordinate::Point;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use std::path::Path;
#[derive(Debug)]
pub struct FileBuffer {
    contents: String,
    is_read_only: bool,
    path: String,
    search_result: Vec<Point>,
    search_result_index: u16,
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
                search_result: vec![],
                search_result_index: 0,
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
    pub fn set_read_only(&mut self, dst: bool) {
        self.is_read_only = dst;
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
    pub fn search_result_register(&mut self, result: Vec<Point>) {
        self.search_result_index = 0;
        self.search_result = result;
    }
    pub fn get_next_searchresult(&mut self) -> Option<Point> {
        if self.search_result.len() > 0 {
            if self.search_result_index + 1 < self.search_result.len() as u16 {
                self.search_result_index += 1;
            } else {
                self.search_result_index = 0;
            }
            Some(Point {
                col: self.search_result[self.search_result_index as usize].col,
                row: self.search_result[self.search_result_index as usize].row,
            })
        } else {
            None
        }
    }
}
#[cfg(test)]
mod FileTest {
    use super::FileBuffer;

    #[test]
    fn test_read_write_contents() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        buf.update_contents(String::from("new string"));
        assert_eq!(buf.get_contents(), String::from("new string"));
    }
    #[test]
    fn test_get_set_readonly() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        buf.set_read_only(true);
        assert!(buf.get_read_only());
        buf.set_read_only(false);
        assert!(!buf.get_read_only());
    }
    #[test]
    fn test_save_file() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        buf.update_contents(String::from("new string2"));
        assert_eq!(buf.get_contents(), String::from("new string2"));
        assert_eq!(buf.save_file(), Ok(()));
        let buf2 = FileBuffer::new(&p).unwrap();
        assert_eq!(buf2.get_contents(), String::from("new string2"));
    }
    #[test]
    fn test_get_length() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        buf.update_contents(String::from("1234567890\n2234567890\n3234567890\n"));
        assert_eq!(buf.get_row_length(), 4);
        assert_eq!(buf.get_col_length(0), 10);
        assert_eq!(buf.get_col_length(1), 10);
        assert_eq!(buf.get_col_length(2), 10);
    }
}
