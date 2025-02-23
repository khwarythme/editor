use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crate::modules::mode::MODE;
use crossterm::event::KeyCode;
pub struct Search {
    ptn: String,
}
impl Search {
    pub fn new() -> Search {
        Search { ptn: String::new() }
    }
    pub fn proc_search(&mut self, code: KeyCode, buf: &mut FileBuffer) -> MODE {
        match code {
            KeyCode::Char(c) => {
                self.ptn = format!("{}{}", self.ptn, c);
                MODE::Search
            }
            KeyCode::Enter => {
                buf.search_result_register(search_string(buf.get_contents(), &self.ptn));
                self.ptn.clear();
                MODE::Normal
            }
            _ => MODE::Normal,
        }
    }
}
pub fn search_string(buf: String, ptn: &str) -> Vec<Point> {
    let mut ret: Vec<Point> = vec![];
    let matchsize = ptn.len();
    let mut row = 0;
    for line in buf.split('\n') {
        let mut matchindex = 0;
        let mut col = 0;
        for charactor in line.chars() {
            let p = match ptn.chars().nth(matchindex) {
                Some(a) => a,
                None => 0x00 as char,
            };
            if p != charactor {
                if matchindex > 0 {
                    col += matchindex;
                }
                matchindex = 0;
                col += 1;
            } else {
                matchindex += 1;
                if matchindex == matchsize {
                    ret.push(Point { col, row });
                }
            }
        }
        row += 1;
    }

    ret
}
