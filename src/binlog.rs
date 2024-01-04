use parking_lot::Mutex;

use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    marker::PhantomData,
    path::Path,
    sync::{mpsc, Arc},
};

use uuid::Uuid;
use zerocopy::{AsBytes, FromBytes, FromZeroes};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};

use crate::Prop;

pub trait Exchangable: FromZeroes + FromBytes + AsBytes {}
impl<T> Exchangable for T where T: FromZeroes + FromBytes + AsBytes {}

mod op {
    pub type Operate = u8;

    // Create a key-value pair which does not exist before.
    pub const CREATE: Operate = 0b00;

    // Update an exist key-value pair.
    pub const UPDATE: Operate = 0b01;

    // Merge a value. Atom do not include the delta funtion.
    // pub const MERGE: Operate = 0b10;

    // Delete a value.
    pub const DELETE: Operate = 0b11;
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
    pub const fn len() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub struct AtomBuffer<K, V, IO>
where
    K: Exchangable,
    V: Exchangable,
    IO: Read + Write,
{
    buffer_writer: Arc<mpsc::Sender<Atom<K, V>>>,
    buffer_reader: mpsc::Receiver<Atom<K, V>>,
    to: Mutex<IO>,
}

impl<K, V, IO> AtomBuffer<K, V, IO>
where
    K: Exchangable,
    V: Exchangable,
    IO: Read + Write,
{
    pub fn new(io: IO) -> Self {
        let (buffer_writer, buffer_reader) = mpsc::channel();
        AtomBuffer {
            buffer_writer: Arc::new(buffer_writer),
            buffer_reader,
            to: Mutex::new(io),
        }
    }
    pub fn get_committer(&self) -> Arc<mpsc::Sender<Atom<K, V>>> {
        self.buffer_writer.clone()
    }

    pub fn commit(&self, data: Atom<K, V>) -> () {
        self.buffer_writer.send(data).unwrap();
    }

    pub fn flush_exact(&self, n: usize) -> usize {
        let mut rtn = 0;
        for _ in 0..n {
            if let Ok(data) = self.buffer_reader.try_recv() {
                self.to.lock().write(data.as_bytes()).unwrap();
                rtn += 1;
            }
        }
        self.to.lock().flush().unwrap();
        rtn
    }

    pub fn flush(&self) {
        self.buffer_reader.try_iter().for_each(|data| {
            self.to.lock().write(data.as_bytes()).unwrap();
        });
        self.to.lock().flush().unwrap();
    }
}

impl<K, V, IO> Read for AtomBuffer<K, V, IO>
where
    K: Exchangable,
    V: Exchangable,
    IO: Read + Write,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.to.lock().read(buf)
    }
}

pub const NAMESPACE_ATOM: Uuid = Uuid::nil();

pub struct Binlog<K, V, D = ()>
where
    K: Ord + Copy + Exchangable,
    V: Clone + Default + Exchangable,
{
    atom_len: u32,
    to: AtomBuffer<K, V, File>,
    _d: PhantomData<D>,
}

impl<K, V, D> Binlog<K, V, D>
where
    K: Ord + Copy + Exchangable,
    V: Clone + Default + Exchangable,
{
    pub fn new(path: String, prop_name: String) -> Result<Self, sled::Error> {
        let p = format!("{}/{}.binlog", path, prop_name);
        let file_path = Path::new::<String>(&p);
        let mut atom_len: u32 = 0;
        let file = match fs::metadata(file_path) {
            Ok(_) => {
                let mut file = File::options()
                    .append(true)
                    .read(true)
                    .write(true)
                    .open(file_path)?;

                file.read(&mut atom_len.as_bytes_mut())?;

                let mut uuid_buf = [0u8; 16];
                file.read_exact(uuid_buf.as_mut())?;
                let prop_uuid = Uuid::from_bytes(uuid_buf);
                assert_eq!(
                    prop_uuid,
                    Uuid::new_v5(&NAMESPACE_ATOM, prop_name.as_bytes()),
                );
                file
            }
            Err(_) => {
                let mut file = File::options().create(true).write(true).open(file_path)?;

                atom_len = Atom::<K, V>::len() as u32;

                file.write_all(atom_len.as_bytes()).unwrap();

                let uuid = Uuid::new_v5(&NAMESPACE_ATOM, prop_name.as_bytes());
                file.write_all(uuid.as_bytes()).unwrap();
                file
            }
        };

        Ok(Binlog {
            atom_len,
            to: AtomBuffer::new(file),
            _d: PhantomData,
        })
    }

    pub fn into_iter(self) -> IntoIter<K, V, AtomBuffer<K, V, File>> {
        IntoIter {
            read_buf: BufReader::new(self.to),
            atom_len: self.atom_len,
            _k: PhantomData,
            _v: PhantomData,
        }
    }

    pub fn into_prop(self) -> Prop<K, V, D> {
        let p = Prop::new();
        self.into_iter().for_each(|x| {
            let key = x.key;
            match x.op {
                op::CREATE => p.set(&key, x.value),
                op::DELETE => p.remove(&key),
                op::UPDATE => p.set(&key, x.value),
                _ => None,
            };
        });
        p
    }

    pub fn get_committer(&self) -> Arc<mpsc::Sender<Atom<K, V>>> {
        self.to.get_committer()
    }

    pub fn commit(&self, data: Atom<K, V>) -> () {
        self.to.commit(data);
    }

    pub fn flush(&self) {
        self.to.flush();
    }
}

pub struct IntoIter<K, V, IO>
where
    K: Ord + Copy + Exchangable,
    V: Clone + Default + Exchangable,
    IO: Read,
{
    read_buf: BufReader<IO>,
    atom_len: u32,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl<K, V, IO> Iterator for IntoIter<K, V, IO>
where
    K: Ord + Copy + Exchangable,
    V: Clone + Default + Exchangable,
    IO: Read,
{
    type Item = Atom<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut atom_buf = vec![0u8; self.atom_len as usize];
        self.read_buf.read_exact(&mut atom_buf).unwrap();
        Atom::read_from(&atom_buf)
    }
}

mod test {

    #[test]
    fn test_buf() {
        use super::{op, Atom, AtomBuffer};
        use std::io::Cursor;
        use std::io::Write;
        use zerocopy::AsBytes;

        let to = Cursor::new(vec![]);

        let log = AtomBuffer::<u8, [u8; 3], Cursor<Vec<u8>>>::new(to);

        for i in 0u8..3 {
            let data = Atom::<u8, [u8; 3]>::new(op::UPDATE, i, [111u8; 3]);
            println!("{:?}", data.as_bytes());
            log.commit(data);
        }

        assert_eq!(log.flush_exact(1), 1);

        let mut mock_buff = Cursor::new(vec![1, 0, 111, 111, 111]);
        mock_buff.set_position(5);
        assert_eq!(*log.to.lock(), mock_buff);

        assert_eq!(log.flush_exact(3), 2);

        mock_buff
            .write_all(&vec![1, 1, 111, 111, 111, 1, 2, 111, 111, 111])
            .unwrap();
        assert_eq!(*log.to.lock(), mock_buff);

        assert_eq!(log.flush_exact(1), 0);
    }

    #[test]
    fn test_binlog() {
        use super::Binlog;
        use crate::binlog::{op, Atom};
        use std::fs::File;
        use std::io::Read;

        let dir = tempfile::TempDir::new().unwrap();
        let log = Binlog::<u64, [u8; 4]>::new(
            dir.path().to_str().unwrap().to_string(),
            "test".to_string(),
        )
        .unwrap();

        log.commit(Atom::new(op::CREATE, 1, [1, 2, 3, 4]));
        log.flush();

        let path = format!("{}/test.binlog", dir.path().to_str().unwrap());
        let mut file = File::open(path).unwrap();

        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(
            content,
            vec![
                // atom_len
                13, 0, 0, 0, // uuid
                232, 183, 100, 218, 95, 229, 81, 237, 138, 248, 197, 198, 236, 162, 141, 122,
                // data

                // 1st data
                0, // op
                1, 0, 0, 0, 0, 0, 0, 0, // key
                1, 2, 3, 4 // value
            ]
        );

        log.commit(Atom::new(op::UPDATE, 1, [2; 4]));
        log.commit(Atom::new(op::DELETE, 1, [0; 4]));
        log.flush();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(
            content,
            vec![
                // atom_len
                13, 0, 0, 0, // uuid
                232, 183, 100, 218, 95, 229, 81, 237, 138, 248, 197, 198, 236, 162, 141, 122,
                // data

                // 1st data
                0, // op
                1, 0, 0, 0, 0, 0, 0, 0, // key
                1, 2, 3, 4, // value
                // 2nd data
                1, // op
                1, 0, 0, 0, 0, 0, 0, 0, // key
                2, 2, 2, 2, // value
                // 3rd data
                3, // op
                1, 0, 0, 0, 0, 0, 0, 0, // key
                0, 0, 0, 0, // value
            ]
        );
    }
}
