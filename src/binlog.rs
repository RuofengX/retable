use std::{
    fs::{self, File},
    io::{Error, Write},
    marker::PhantomData,
    path::Path,
    thread::{self, JoinHandle},
    time::Duration,
};

use zerocopy::{AsBytes, FromBytes, FromZeroes};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};

pub trait Exchangable: FromZeroes + FromBytes + AsBytes {}
impl<T> Exchangable for T where T: FromZeroes + FromBytes + AsBytes {}

pub mod op {
    pub type Operate = u8;

    // Create a key-value pair which does not exist before.
    pub const CREATE: Operate = 0b00;

    // Update an exist key-value pair.
    pub const UPDATE: Operate = 0b01;

    // Merge a value. Atom do not include the delta funtion.
    pub const MERGE: Operate = 0b10;

    // Delete a value.
    pub const DELETE: Operate = 0b11;
}

#[derive(Debug, AsBytes, FromBytes, FromZeroes, Unaligned, PartialEq, Eq)]
#[repr(packed)]
pub struct Atom<K, V, D>
where
    K: Exchangable,
    V: Exchangable,
    D: Exchangable,
{
    op: op::Operate,
    key: K,
    value: V, // If not use, incoming zero will benefit the compress algorithm.
    delta: D,
}
impl<K, V, D> Atom<K, V, D>
where
    K: Exchangable,
    V: Exchangable,
    D: Exchangable,
{
    pub fn new(op: op::Operate, key: K, value: V, delta: D) -> Self {
        Atom {
            op,
            key,
            value,
            delta,
        }
    }
    pub const fn len() -> usize {
        std::mem::size_of::<Self>()
    }
    pub fn name() -> &'static str {
        std::any::type_name::<Atom<K, V, D>>()
    }
}

pub struct AtomArchive<K, V, D>
where
    K: Exchangable,
    V: Exchangable,
    D: Exchangable,
{
    socket: zmq::Socket,
    file: File,
    _a: PhantomData<Atom<K, V, D>>,
}
impl<K, V, D> AtomArchive<K, V, D>
where
    K: Exchangable + Send + Sync + 'static,
    V: Exchangable + Send + Sync + 'static,
    D: Exchangable + Send + Sync + 'static,
{
    pub fn new(ctx: &zmq::Context, folder_path: &Path) -> Result<Self, Error> {
        let name = Atom::<K, V, D>::name();
        let path = folder_path.join(name);

        let file = File::options()
            .create(true)
            .append(true)
            .write(true)
            .read(true)
            .open(path)
            .unwrap();

        let socket = ctx.socket(zmq::SocketType::PULL).unwrap();
        socket
            .bind(format!("inproc://atom.archive/{}", Atom::<K, V, D>::name()).as_str())
            .unwrap();
        Ok(Self {
            socket,
            file,
            _a: PhantomData,
        })
    }
    pub fn endpoint(&self) -> String {
        format!("inproc://atom.archive/{}", Atom::<K, V, D>::name())
    }
    pub fn pull(&mut self) -> usize {
        let mut atom_buf = Atom::<K, V, D>::new_zeroed();
        let mut count = 0;
        while let Ok(_) = self
            .socket
            .recv_into(atom_buf.as_bytes_mut(), zmq::DONTWAIT)
        {
            count += 1;
            self.file.write(atom_buf.as_bytes()).unwrap();
        }
        self.file.flush().unwrap();
        count
    }

    pub fn run_forever(mut self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            self.pull();
            thread::sleep(Duration::from_millis(1000));
        })
    }
}

mod test {
    use std::io::Seek;

    #[test]
    fn test_atom() {
        use super::{op, Atom};
        use zerocopy::AsBytes;
        use zerocopy::FromBytes;

        let a = Atom::new(op::CREATE, 1, 2, ());
        let b = a.as_bytes();
        assert_eq!(Atom::<i32, i32, ()>::len(), 9);
        assert_eq!(b, [0, 1, 0, 0, 0, 2, 0, 0, 0]);
        let aa = Atom::<i32, i32, ()>::read_from(b).unwrap();
        assert_eq!(aa, a);

        let c = Atom::<i32, (), i32>::read_from(b).unwrap();
        assert_eq!(c, Atom::new(op::CREATE, 1, (), 2));
    }

    #[test]
    fn test_binlog() {
        use super::op;
        use super::Atom;
        use super::AtomArchive;
        use std::io::Read;
        use tempfile::tempdir;
        use zerocopy::AsBytes;

        let ctx = zmq::Context::new();
        let dir = tempdir().unwrap();

        let path = dir.path();

        let mut aa = AtomArchive::<u8, u8, ()>::new(&ctx, path).unwrap();

        let client = ctx.socket(zmq::PUSH).unwrap();
        client.connect(aa.endpoint().as_str()).unwrap();

        let i = 42;
        let atom = Atom::new(op::CREATE, i, i, ());
        client.send(atom.as_bytes(), zmq::DONTWAIT).unwrap();

        aa.pull();

        let mock_buf = vec![0, 42, 0];
        let mut content = vec![];

        aa.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        aa.file.read_to_end(&mut content).unwrap();

        assert_eq!(mock_buf, content);
    }
}
