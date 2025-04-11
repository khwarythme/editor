use crate::modules::coordinate::Point;
use crate::modules::history::*;
use std::sync::{Arc, Mutex};

use std::collections::VecDeque;
use tokio;

pub struct Undo {
    history: History,
    undo_history: History,
}
pub enum UndoDirection {
    Undo,
    Redo,
}
pub struct Yank {
    yank: VecDeque<char>,
}
impl Yank {
    pub fn new() -> Self {
        Self {
            yank: VecDeque::new(),
        }
    }
    pub fn yankchars(&mut self, chars: VecDeque<char>) {
        self.yank = chars;
    }
    pub fn yank(&mut self, buf: VecDeque<VecDeque<char>>, start: Point) {
        let mut count = 0;
        for line in buf {
            if count == start.row {
                let c: VecDeque<char> = line.clone();
                self.yankchars(c);
            }
            count += 1;
        }
    }
    pub async fn past(
        &self,
        row: usize,
        buf: VecDeque<VecDeque<char>>,
        undo: &mut Undo,
    ) -> VecDeque<VecDeque<char>> {
        let mut tmp = self.yank.clone();
        tmp.push_back('\n');
        let ret = insert(Point { column: 0, row }, buf, tmp.clone()).await;
        undo.add_do_history(Operation::ADD, tmp, Point { column: 0, row });
        ret
    }
}

impl Undo {
    pub fn new() -> Undo {
        Undo {
            history: History::new(),
            undo_history: History::new(),
        }
    }
    pub fn add_do_history(&mut self, op: Operation, target: VecDeque<char>, pos: Point) {
        if target.len() > 0 {
            self.history.add(op, target, pos);
        }
    }
    pub async fn undo(
        &mut self,
        buf: VecDeque<VecDeque<char>>,
        direction: UndoDirection,
    ) -> (VecDeque<VecDeque<char>>, Point) {
        let record = match direction {
            UndoDirection::Undo => self.history.undo(),
            UndoDirection::Redo => self.undo_history.undo(),
        };
        let history = match direction {
            UndoDirection::Undo => &mut self.undo_history,
            UndoDirection::Redo => &mut self.history,
        };

        match record.get_operation() {
            Operation::ADD => {
                let (result, _delchar) = del(record.get_pos(), buf, record.get_target().len()).await;
                history.add(
                    Operation::DELETE,
                    record.get_target().clone(),
                    record.get_pos(),
                );
                (result, record.get_pos())
            }
            Operation::DELETE => {
                let result = insert(record.get_pos(), buf, record.get_target()).await;
                history.add(
                    Operation::ADD,
                    record.get_target().clone(),
                    record.get_pos(),
                );
                (result, record.get_pos())
            }
            _ => (buf, Point { column: 0, row: 0 }),
        }
    }
}

/// insert a charactor on a point.
pub async fn insert(
    start: Point,
    base_string: VecDeque<VecDeque<char>>,
    charactor: VecDeque<char>,
) -> VecDeque<VecDeque<char>> {
    // create copy string
    let mut result: VecDeque<VecDeque<char>> = base_string.clone();

    if base_string.len() < 1 {
        return VecDeque::from([charactor.clone()]);
    }
    if charactor.back().unwrap_or(&'\0').eq(&'\n') {
        let mut ret = base_string.clone();
        let mut c = charactor.clone();
        c.pop_back();
        let row = if ret.len() < start.row {
            ret.push_back(c);
            return ret;
        } else {
            start.row
        };
        let mut tmp = ret.remove(row).unwrap_or(VecDeque::new());
        let mut tmp_vec = VecDeque::new();
        if tmp.len() + 1 > start.column {
            let tmp2 = tmp.split_off(start.column);
            tmp.append(&mut c);
            tmp_vec.push_back(tmp);
            tmp_vec.push_back(tmp2);
            for t in 0..tmp_vec.len() {
                ret.insert(row + t, tmp_vec.pop_front().unwrap());
            }
        } else {
            tmp.append(&mut c);
            tmp_vec.push_back(VecDeque::new());
            tmp_vec.push_back(tmp);
            for t in 0..tmp_vec.len() {
                ret.insert(row + t, tmp_vec.pop_front().unwrap());
            }
        }
        return ret;
    }
    // move to target row
    let mut col_count = 0;
    let mut tmp_line = VecDeque::from([]);
    let src_line: VecDeque<char> = result
        .clone()
        .into_iter()
        .nth(start.row)
        .unwrap_or(VecDeque::from([]))
        .clone();
    if src_line.len() <= start.column {
        tmp_line = src_line.clone();
        for c in charactor.clone() {
            tmp_line.push_back(c);
        }
    } else {
        for chara in src_line {
            if col_count == start.column {
                for inserted_char in charactor.clone() {
                    tmp_line.push_back(inserted_char);
                }
            }
            tmp_line.push_back(chara);
            col_count += 1;
        }
    }

    result.remove(start.row);
    result.insert(start.row, tmp_line);
    result
}

/// delete 1 or some charactor(s) from base_string
/// point_in_file is first point to delete charactor(s).
/// length is delete length
/// it returns result string and deleted charactor(s) set as vector
pub async fn del(
    start: Point,
    base_string: VecDeque<VecDeque<char>>,
    length: usize,
) -> (VecDeque<VecDeque<char>>, VecDeque<char>) {
    let mut ret_string = VecDeque::new();
    let mut ret_chars = VecDeque::new();
    let tmp_point = start;
    let mut row_count = 0;
    let mut del_cr = false;
    for mut line in base_string {
        // looking for target row
        if tmp_point.row == row_count {
            //loocing for target col
            if tmp_point.column == line.len() {
                ret_chars.push_back('\n');
                del_cr = true;
            } else {
                let limit = if tmp_point.column + length > line.len() {
                    line.len() - tmp_point.column + 1
                } else {
                    length
                };
                for _ in 0..limit {
                    let del_char = line.remove(tmp_point.column).unwrap_or('\0');
                    if del_char == '\0' {
                        ret_chars.push_back('\n');
                        del_cr = true;
                    } else {
                        ret_chars.push_back(del_char);
                    }
                }
            }
            ret_string.push_back(line);
        } else {
            if del_cr {
                let mut tmp_line = ret_string.pop_back().unwrap_or(VecDeque::from(['\0']));
                tmp_line.append(&mut line);
                ret_string.push_back(tmp_line);
                del_cr = false;
            } else {
                ret_string.push_back(line);
            }
        }
        row_count += 1;
    }

    (ret_string, ret_chars)
}

#[cfg(test)]
mod insert_test {
    use super::*;

    #[tokio::test]
    async fn insert_first() {
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected1: VecDeque<VecDeque<char>> = VecDeque::from([
            "adata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 0, row: 0 },
            src.clone(),
            VecDeque::from(['a']),
        ).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected1);
        let expected2: VecDeque<VecDeque<char>> = VecDeque::from([
            "aadata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 0, row: 0 },
            dist.clone(),
            VecDeque::from(['a']),
        ).await;
        assert_ne!(src, dist);
        assert_ne!(dist, expected1);
        assert_eq!(dist, expected2);
        let expected3: VecDeque<VecDeque<char>> = VecDeque::from([
            VecDeque::from([]),
            "aadata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 0, row: 0 },
            dist.clone(),
            VecDeque::from(['\n']),
        ).await;
        assert_ne!(src, dist);
        assert_ne!(dist, expected1);
        assert_ne!(dist, expected2);
        assert_eq!(dist, expected3);
    }
    #[tokio::test]
    async fn insert_midpoint() {
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected4: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            VecDeque::from([]),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 0, row: 1 },
            src.clone(),
            VecDeque::from(['\n']),
        ).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected4);
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected4: VecDeque<VecDeque<char>> = VecDeque::from([
            "data".chars().into_iter().collect(),
            "0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 4, row: 0 },
            src.clone(),
            VecDeque::from(['\n']),
        ).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected4);
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected5: VecDeque<VecDeque<char>> = VecDeque::from([
            "d".chars().into_iter().collect(),
            "ata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 1, row: 0 },
            src.clone(),
            VecDeque::from(['\n']),
        ).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected5);
    }
    #[tokio::test]
    async fn insert_last() {
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected5: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
            VecDeque::from([]),
        ]);
        let dist = insert(
            Point { column: 0, row: 6 },
            src.clone(),
            VecDeque::from(['\n']),
        ).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected5);
    }
}
#[cfg(test)]
mod del_test {
    use super::*;

    #[tokio::test]
    async fn del_test() {
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected1: VecDeque<VecDeque<char>> = VecDeque::from([
            "ata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let (dist, c) = del(Point { column: 0, row: 0 }, src.clone(), 1).await;
        assert_ne!(src, dist);
        assert_eq!(dist, expected1);
        assert_eq!(c, VecDeque::from(['d']));
        let expected2: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "ata1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let (dist, c) = del(Point { column: 0, row: 1 }, src.clone(), 1).await;
        assert_ne!(src, dist);
        assert_ne!(dist, expected1);
        assert_eq!(c, VecDeque::from(['d']));
        assert_eq!(dist, expected2);
        let expected3: VecDeque<VecDeque<char>> = VecDeque::from([
            "data".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let (dist, c) = del(Point { column: 4, row: 0 }, src.clone(), 1).await;
        assert_ne!(src, dist);
        assert_ne!(dist, expected1);
        assert_ne!(dist, expected2);
        assert_eq!(c, VecDeque::from(['0']));
        assert_eq!(dist, expected3);
        let expected4: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let (dist, c) = del(Point { column: 5, row: 0 }, src.clone(), 1).await;
        assert_ne!(src, dist);
        assert_ne!(dist, expected1);
        assert_ne!(dist, expected2);
        assert_ne!(dist, expected3);
        assert_eq!(c, VecDeque::from(['\n']));
        assert_eq!(dist, expected4);
    }
    #[tokio::test]
    async fn del_all() {
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected1: VecDeque<VecDeque<char>> = VecDeque::from([VecDeque::new()]);
        let (dist, c) = del(Point { column: 0, row: 0 }, src.clone(), 6).await;
        assert_eq!(c, VecDeque::from(['d', 'a', 't', 'a', '0', '\n']));

        let (dist, c) = del(Point { column: 0, row: 0 }, dist.clone(), 6).await;
        assert_eq!(c, VecDeque::from(['d', 'a', 't', 'a', '1', '\n']));
        let (dist, c) = del(Point { column: 0, row: 0 }, dist.clone(), 6).await;
        assert_eq!(c, VecDeque::from(['d', 'a', 't', 'a', '2', '\n']));
        let (dist, c) = del(Point { column: 0, row: 0 }, dist.clone(), 6).await;
        assert_eq!(c, VecDeque::from(['d', 'a', 't', 'a', '3', '\n']));
        let (dist, c) = del(Point { column: 0, row: 0 }, dist.clone(), 6).await;
        assert_eq!(c, VecDeque::from(['d', 'a', 't', 'a', '4', '\n']));
        assert_ne!(src, dist);
        assert_eq!(dist, expected1);
    }
}

#[cfg(test)]
mod undo_test {
    use super::*;

    #[tokio::test]
    async fn undo_after_insert() {
        let mut undo = Undo::new();
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected1: VecDeque<VecDeque<char>> = VecDeque::from([
            "adata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let dist = insert(
            Point { column: 0, row: 0 },
            src.clone(),
            VecDeque::from(['a']),
        ).await;
        undo.add_do_history(
            Operation::ADD,
            VecDeque::from(['a']),
            Point { column: 0, row: 0 },
        );
        assert_ne!(src, dist);
        assert_eq!(dist, expected1);
        let (r, p) = undo.undo(dist, UndoDirection::Undo).await;
        assert_eq!(src, r);
        assert_eq!(Point { column: 0, row: 0 }, p);
    }
    #[tokio::test]
    async fn undo_after_delete() {
        let mut undo = Undo::new();
        let src: VecDeque<VecDeque<char>> = VecDeque::from([
            "data0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let expected1: VecDeque<VecDeque<char>> = VecDeque::from([
            "ata0".chars().into_iter().collect(),
            "data1".chars().into_iter().collect(),
            "data2".chars().into_iter().collect(),
            "data3".chars().into_iter().collect(),
            "data4".chars().into_iter().collect(),
        ]);
        let (dist, point) = del(Point { column: 0, row: 0 }, src.clone(), 1).await;
        undo.add_do_history(
            Operation::DELETE,
            VecDeque::from(['d']),
            Point { column: 0, row: 0 },
        );
        assert_ne!(src, dist);
        assert_eq!(dist, expected1);
        let (r, p) = undo.undo(dist, UndoDirection::Undo).await;
        assert_eq!(src, r);
        assert_eq!(Point { column: 0, row: 0 }, p);
    }
}

pub async fn thread_main(contents:Arc<Mutex<VecDeque<VecDeque<char>>>>) ->Result<(),String>{
    println!("hello thread2");        
    Ok(())
}
