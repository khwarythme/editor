use tokio;
use std::sync::{Arc,Mutex, mpsc::*};
use std::collections::VecDeque;
use std::path::Path;
use super::coordinate::Point;
use super::*;

enum SenderID{
    Show,
    FileControl,
    Input,
    Lsp,
    Mode,
}

struct Operation{
    sender: SenderID,
    coordinate: Point,
}

pub async fn server_main(file_path: &Path) -> Result<(), String> {
    let file = file::FileBuffer::new(file_path)?;
    let contents= Arc::new(Mutex::new(file.get_contents()));

    let (tx,rx) = channel::<Operation>();
    let tx = Mutex::new(tx);
    let rx = Mutex::new(rx);
    let show_thread = tokio::task::spawn(
       show::thread_main(Arc::clone(&contents)));
    let edit_thread = tokio::task::spawn(
        edit::thread_main(Arc::clone(&contents)
    ));

    Ok(())
}
