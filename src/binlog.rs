#![allow(unused)]

use std::{
    io::{BufWriter, Write},
    slice::IterMut,
    sync::{
        atomic::{AtomicBool, AtomicI8, Ordering},
        mpsc, Arc,
    },
};

use parking_lot::RwLock;
use zerocopy::{AsBytes, FromBytes, FromZeroes, Unaligned};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};

pub trait Exchangable: FromZeroes + FromBytes + AsBytes + Unaligned {}
impl<T> Exchangable for T where T: FromZeroes + FromBytes + AsBytes + Unaligned {}

mod op {
    pub type Operate = u8;

    // Create a key-value pair which does not exist before.
    pub const CREATE: Operate = 0b00;

    // Update an exist key-value pair.
    pub const UPDATE: Operate = 0b01;

    // Delete a value.
    pub const DELETE: Operate = 0b10;
}
#[derive(AsBytes, FromBytes, FromZeroes, Unaligned)]
#[repr(packed)]
pub struct Commit<K, V> {
    op: op::Operate,
    key: K,
    value: V,
}
impl<K, V> Commit<K, V> {
    pub fn new(op: op::Operate, key: K, value: V) -> Self {
        Commit { op, key, value }
    }
}

pub struct CommitBuffer<K, V>
where
    K: Exchangable,
    V: Exchangable,
{
    buffer_writer: Arc<mpsc::Sender<Commit<K, V>>>,
    buffer_reader: mpsc::Receiver<Commit<K, V>>,
    buffer: Vec<u8>,
}

impl<K, V> CommitBuffer<K, V>
where
    K: Exchangable,
    V: Exchangable,
{
    pub fn new() -> Self {
        let (buffer_writer, buffer_reader) = mpsc::channel();
        let buffer = Vec::new();
        CommitBuffer {
            buffer_writer: Arc::new(buffer_writer),
            buffer_reader,
            buffer,
        }
    }
    pub fn get_committer(&self) -> Arc<mpsc::Sender<Commit<K, V>>> {
        self.buffer_writer.clone()
    }

    pub fn commit(&self, data: Commit<K, V>) -> () {
        self.buffer_writer.send(data).unwrap();
    }

    pub fn save(&self, n: usize, f: &mut impl Write) -> usize {
        let mut rtn = 0;
        for _ in (0..n) {
            if let Ok(data) = self.buffer_reader.try_recv() {
                f.write(data.as_bytes());
                rtn += 1;
            }
        }
        rtn
    }

    pub fn save_all(&self, n: usize, f: &mut impl Write) {
        self.buffer_reader.try_iter().for_each(|data| {
            f.write(data.as_bytes());
        })
    }
}

mod test {

    #[test]
    fn test_binlog() {
        use super::{op, Commit, CommitBuffer};
        use std::io::Cursor;
        use zerocopy::AsBytes;

        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let log = CommitBuffer::<u8, [u8; 1]>::new();

        for i in 0u8..3 {
            let data = Commit::new(op::UPDATE, i, [1; 1]);
            println!("{:?}", data.as_bytes());
            log.commit(data);
        }

        assert_eq!(log.save(1, &mut buffer), 1);
        let mut mock_buff = Cursor::new(vec![1, 0, 1]);
        mock_buff.set_position(3);
        assert_eq!(buffer, mock_buff);

        assert_eq!(log.save(3, &mut buffer), 2);
        let mut mock_buff = Cursor::new(vec![1, 0, 1, 1, 1, 1, 1, 2, 1]);
        mock_buff.set_position(9);
        assert_eq!(buffer, mock_buff);

        assert_eq!(log.save(1, &mut buffer), 0);
    }
}
