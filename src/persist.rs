use std::{
    fs::File,
    io::Write,
    sync::{mpsc, Arc},
    thread::JoinHandle,
};
use zerocopy::AsBytes;

use parking_lot::Mutex;

use crate::protocol::{Atom, LogWriter};

pub struct DequeFileWriter<K, V, D> {
    sender: mpsc::Sender<Atom<K, V, D>>,
    file: Arc<Mutex<File>>,
    writer_handle: JoinHandle<()>,
}
impl<K, V, D> DequeFileWriter<K, V, D>
where
    K: AsBytes + Send + Sync + 'static,
    V: AsBytes + Send + Sync + 'static,
    D: AsBytes + Send + Sync + 'static,
{
    pub fn new(file: Arc<Mutex<File>>) -> Self {
        let (tx, rx) = mpsc::channel::<Atom<K, V, D>>();
        let file_dummy = file.clone();
        let writer_handle = std::thread::spawn(move || loop {
            let mut buffer_length: usize = 0;
            if let Ok(atom) = rx.recv() {
                let mut file = file_dummy.lock();
                file.write(atom.as_bytes()).unwrap();
                buffer_length += atom.len();
                if buffer_length > 256 * 1024 * 1024 {
                    // file memory limit is 256 MiB
                    file.flush().unwrap();
                }
            }
        });
        Self {
            sender: tx,
            file,
            writer_handle,
        }
    }
}
impl<K, V, D> LogWriter<K, V, D> for DequeFileWriter<K, V, D> {
    fn save_one(&self, data: Atom<K, V, D>) {
        self.sender.send(data).unwrap();
    }
}
