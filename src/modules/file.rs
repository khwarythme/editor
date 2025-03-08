use crate::modules::coordinate::Point;
use std::collections::VecDeque;
use std::fs::{canonicalize, File};
use std::io::prelude::*;
use std::io::BufWriter;

use std::path::Path;
#[derive(Debug)]
pub struct FileBuffer {
    contents: VecDeque<VecDeque<char>>,
    is_read_only: bool,
    path: String,
    search_result: VecDeque<Point>,
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
            Ok(_) => {
                let mut chars: VecDeque<VecDeque<char>> = VecDeque::new();
                let buf_string = String::from_utf8(buf).unwrap_or(String::from(""));
                for line in buf_string.lines() {
                    let line_char: VecDeque<char> = line.chars().into_iter().collect();
                    chars.push_back(line_char);
                }

                Ok(FileBuffer {
                    contents: chars,
                    is_read_only: false,
                    path: canonicalize(path)
                        .unwrap()
                        .to_str()
                        .unwrap_or("")
                        .to_string(),
                    search_result: VecDeque::new(),
                    search_result_index: 0,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn change_file(&mut self, new_buf: FileBuffer) {
        self.contents = new_buf.get_contents();
        self.is_read_only = new_buf.get_read_only();
        self.path = new_buf.get_path();
        self.search_result = new_buf.search_result;
        self.search_result_index = new_buf.search_result_index;
    }
    pub fn get_contents(&self) -> VecDeque<VecDeque<char>> {
        self.contents.clone()
    }
    pub fn update_contents(&mut self, new_contents: VecDeque<VecDeque<char>>) {
        self.contents = new_contents.clone();
    }
    pub fn get_path(&self) -> String {
        self.path.clone()
    }
    pub fn save_file(&mut self) -> Result<(), String> {
        let file = match File::create(Path::new(self.path.as_str())) {
            Ok(some) => some,
            Err(e) => return Err(e.to_string()),
        };

        let mut writer = BufWriter::new(file);
        let mut s: String = String::new();
        for line in self.contents.clone() {
            let mut tmpstr: String = line.into_iter().collect();
            tmpstr.push('\n');
            s.push_str(&tmpstr);
        }

        let _result = match writer.write_all(s.as_bytes()) {
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
    pub fn get_col_length(&self, row: usize) -> usize {
        let r: VecDeque<char> = self
            .contents
            .clone()
            .into_iter()
            .nth(row)
            .unwrap_or(VecDeque::new());
        r.len()
    }
    pub fn get_row_length(&self) -> u16 {
        self.contents.len() as u16
    }
    pub fn search_result_register(&mut self, result: VecDeque<Point>) {
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
                column: self.search_result[self.search_result_index as usize].column,
                row: self.search_result[self.search_result_index as usize].row,
            })
        } else {
            None
        }
    }
}
#[cfg(test)]
mod file_test {
    use super::FileBuffer;
    use std::collections::VecDeque;

    #[test]
    fn test_read_write_contents() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        let target: VecDeque<VecDeque<char>> = VecDeque::from([VecDeque::from([
            'n', 'e', 'w', ' ', 's', 't', 'r', 'i', 'n', 'g',
        ])]);
        buf.update_contents(target);
        let expected: VecDeque<VecDeque<char>> = VecDeque::from([VecDeque::from([
            'n', 'e', 'w', ' ', 's', 't', 'r', 'i', 'n', 'g',
        ])]);
        assert_eq!(buf.get_contents(), expected);
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
        let src = VecDeque::from([VecDeque::from([
            'n', 'e', 'w', ' ', 's', 't', 'r', 'i', 'n', 'g', '2',
        ])]);
        buf.update_contents(src);
        let expected = VecDeque::from([VecDeque::from([
            'n', 'e', 'w', ' ', 's', 't', 'r', 'i', 'n', 'g', '2',
        ])]);
        assert_eq!(buf.get_contents(), expected);
        assert_eq!(buf.save_file(), Ok(()));
        let buf2 = FileBuffer::new(&p).unwrap();
        assert_eq!(buf2.get_contents(), expected);
    }
    #[test]
    fn test_get_length() {
        let p = std::path::Path::new("test.txt");
        let mut buf = FileBuffer::new(&p).unwrap();
        assert_eq!(std::fs::exists(p).unwrap(), true);
        let src = VecDeque::from([
            VecDeque::from(['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']),
            VecDeque::from(['2', '2', '3', '4', '5', '6', '7', '8', '9', '0']),
            VecDeque::from([]),
            VecDeque::from(['3', '2', '3', '4', '5', '6', '7', '8', '9', '0']),
        ]);
        buf.update_contents(src);
        assert_eq!(buf.get_row_length(), 4);
        assert_eq!(buf.get_col_length(0), 10);
        assert_eq!(buf.get_col_length(1), 10);
        assert_eq!(buf.get_col_length(2), 0);
        assert_eq!(buf.get_col_length(3), 10);
    }
}
