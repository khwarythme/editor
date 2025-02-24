use crate::modules::coordinate::Point;
use crate::modules::file::FileBuffer;
use crate::modules::history::*;

pub struct Undo {
    history: History,
    undo_history: History,
}
pub struct Yank {
    yank: Vec<char>,
}
impl Yank {
    pub fn new() -> Self {
        Self { yank: vec![] }
    }
    pub fn yankchars(&mut self, chars: Vec<char>) {
        self.yank = chars;
    }
    pub fn past(&self, row: usize, buf: String) -> String {
        insert(Point { col: 0, row }, buf, &self.yank)
    }
}

impl Undo {
    pub fn new() -> Undo {
        Undo {
            history: History::new(),
            undo_history: History::new(),
        }
    }
    pub fn add_do_history(&mut self, op: Operation, target: Vec<char>, pos: Point) {
        if target.len() > 0 {
            self.history.add(op, target, pos);
        }
    }
    pub fn undo(&mut self, buf: &mut FileBuffer) -> Point {
        let record = self.history.undo();
        match record.get_operation() {
            Operation::HEAD => (),
            Operation::ADD => {
                let (result, delchar) = del(
                    record.get_pos(),
                    buf.get_contents(),
                    record.get_target().len(),
                );

                buf.update_contents(result);
                self.undo_history.add(
                    Operation::DELETE,
                    record.get_target().clone(),
                    record.get_pos(),
                );
            }
            Operation::DELETE => {
                buf.update_contents(insert(
                    Point {
                        col: record.get_pos().col,
                        row: record.get_pos().row,
                    },
                    buf.get_contents(),
                    &record.get_target(),
                ));
                self.undo_history.add(
                    Operation::ADD,
                    record.get_target().clone(),
                    record.get_pos(),
                );
            }
            _ => (),
        };
        record.get_pos()
    }
    pub fn redo(&mut self, buf: &mut FileBuffer) -> Point {
        let record = self.undo_history.undo();
        match record.get_operation() {
            Operation::HEAD => (),
            Operation::ADD => {
                let (result, delchar) = del(
                    record.get_pos(),
                    buf.get_contents(),
                    record.get_target().len(),
                );

                buf.update_contents(result);
                self.history.add(
                    Operation::DELETE,
                    record.get_target().clone(),
                    record.get_pos(),
                );
            }
            Operation::DELETE => {
                buf.update_contents(insert(
                    Point {
                        col: record.get_pos().col,
                        row: record.get_pos().row,
                    },
                    buf.get_contents(),
                    &record.get_target(),
                ));
                self.history.add(
                    Operation::ADD,
                    record.get_target().clone(),
                    record.get_pos(),
                );
            }
            _ => (),
        };
        record.get_pos()
    }
}

/// insert a charactor on a point.
pub fn insert(start: Point, base_string: String, charactor: &Vec<char>) -> String {
    // create copy string
    let mut tmp = String::from(base_string);
    let mut count = 0;
    let mut result = String::new();
    let mut first = true;
    let insert_target: String = charactor.into_iter().collect();

    if tmp.len() < 1 {
        return insert_target;
    }
    let lines_list: Vec<&str> = tmp.lines().collect();
    if lines_list.len() < start.row + 1 {
        tmp.push_str(&insert_target);
        return tmp;
    }
    // move to target row
    for content in lines_list {
        if first {
            first = false;
        } else {
            result.push_str("\n");
        }
        let mut tmpstring = format!("{}", content);
        if count == start.row {
            if tmpstring.is_char_boundary(start.col) {
                tmpstring.insert_str(start.col, &insert_target);
            } else {
                println!("cannot insert txt");
            }
        }
        result.push_str(&tmpstring);
        count += 1;
    }
    String::from(result)
}
/// delete 1 or some charactor(s) from base_string
/// point_in_file is first point to delete charactor(s).
/// length is delete length
/// it returns result string and deleted charactor(s) set as vector
pub fn del(start: Point, base_string: String, length: usize) -> (String, Vec<char>) {
    let mut ret_string = String::new();
    let mut ret_chars = vec![];
    let mut tmp_point = start;
    let mut is_require_cr = false;
    // Q: lines() is not contains '\n' ?
    let mut row_count = 0;
    for line in base_string.lines() {
        let mut col_count = 0;
        let mut tmp_line = String::new();
        if is_require_cr {
            tmp_line.push('\n');
        } else {
            is_require_cr = true;
        }
        // looking for target row
        if tmp_point.row == row_count {
            //loocing for target col
            for charactor in line.to_string().clone().chars() {
                if tmp_point.col <= col_count {
                    if (col_count as usize - tmp_point.col as usize) < length {
                        ret_chars.push(charactor);
                    } else {
                        tmp_line.push(charactor);
                    }
                } else {
                    tmp_line.push(charactor);
                }
                col_count += 1;
            }
            ret_string.push_str(&tmp_line);
            if col_count as usize > line.len() {
                tmp_point = Point {
                    col: 0,
                    row: tmp_point.row + 1,
                };
            }
        } else {
            ret_string.push_str(&tmp_line);
            ret_string.push_str(line);
        }
        row_count += 1;
    }

    (ret_string, ret_chars)
}
