/// insert a charactor on a point.
pub fn insert(col: u16, row: u16, base_string: &String, charactor: char) -> String {
    // create copy string
    let tmp = String::from(base_string);
    let mut count = 0;
    let mut after = String::new();
    let mut first = true;
    // move to target row
    for content in tmp.split('\n') {
        if first {
            first = false;
        } else {
            after.push_str("\n");
        }
        let mut tmpstring = format!("{}", content);
        if count == row {
            tmpstring.insert(col as usize, charactor);
        }
        after.push_str(&tmpstring);
        count += 1;
    }
    String::from(after)
}
pub fn delback(col: u16, row: u16, base_string: &String) -> String {
    let tmp = String::from(base_string);
    let mut count = 0;
    let mut after = String::new();
    let mut cr = true;
    // move to target row
    for content in tmp.split('\n') {
        if cr {
            cr = false;
        } else {
            after.push_str("\n");
        }
        let mut tmpstring = format!("{}", content);
        if count == row {
            if col < tmpstring.len() as u16 {
                tmpstring.remove((col) as usize);
            } else {
                cr = true;
            }
        }
        after.push_str(&tmpstring);
        count += 1;
    }
    String::from(after)
}
