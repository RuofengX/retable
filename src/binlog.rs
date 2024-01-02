#![allow(unused)]

use std::{
    io::{BufWriter, Read, Write, Cursor, Seek, SeekFrom},
    marker::PhantomData,
    slice::IterMut,
    sync::{
        atomic::{AtomicBool, AtomicI8, Ordering},
        mpsc, Arc,
    }, str::FromStr,
};

use parking_lot::RwLock;
use uuid::Uuid;
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
pub struct Atom<K, V> {
    op: op::Operate,
    key: K,
    value: V,
}
impl<K, V> Atom<K, V> {
    pub fn new(op: op::Operate, key: K, value: V) -> Self {
        Atom { op, key, value }
    }
}

pub struct AtomBuffer<K, V>
where
    K: Exchangable,
    V: Exchangable,
{
    buffer_writer: Arc<mpsc::Sender<Atom<K, V>>>,
    buffer_reader: mpsc::Receiver<Atom<K, V>>,
    buffer: Vec<u8>,
}

impl<K, V> AtomBuffer<K, V>
where
    K: Exchangable,
    V: Exchangable,
{
    pub fn new() -> Self {
        let (buffer_writer, buffer_reader) = mpsc::channel();
        let buffer = Vec::new();
        AtomBuffer {
            buffer_writer: Arc::new(buffer_writer),
            buffer_reader,
            buffer,
        }
    }
    pub fn get_committer(&self) -> Arc<mpsc::Sender<Atom<K, V>>> {
        self.buffer_writer.clone()
    }

    pub fn commit(&self, data: Atom<K, V>) -> () {
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

pub const NAMESPACE_ATOM: Uuid = Uuid::nil();

pub struct AtomBeam<IO>
where
    IO: Write + Read + Seek,
{
    prop_uuid: Uuid, // the unique id for this type beam.
    inner: IO,
}

impl<IO> AtomBeam<IO>
where
    IO: Write + Read + Seek,
{
    pub fn new<T: Exchangable>(mut io: IO) -> AtomBeam<IO> {
        let type_name = std::any::type_name::<T>().to_string();
        let uuid = Uuid::new_v5(&NAMESPACE_ATOM, type_name.as_bytes());
        io.seek(SeekFrom::Start(0));
        io.write(type_name.as_bytes());
        Self {
            prop_uuid: uuid,
            inner: io,
        }
    }
    // TODO! Add read method here
    // and manage more than one prop.
}

mod test {

    #[test]
    fn test_binlog() {
        use super::{op, Atom, AtomBuffer};
        use std::io::Cursor;
        use zerocopy::AsBytes;

        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let log = AtomBuffer::<u8, [u8; 1]>::new();

        for i in 0u8..3 {
            let data = Atom::new(op::UPDATE, i, [1; 1]);
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
