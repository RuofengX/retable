# Retable

A Rust library for Atomic-like double-indexed entity-attribute data structures.  

This lib comes with:
+ Atom protocol, a protocol to describe the modify of the database.
+ A simple implementation of the atom protocol, using [`im`].

## Atom

Atom is a way to describe the modify of the database.


| Data Part | Data Type  | Field length | Description                    |
| --------- | ---------- | ------------ | ------------------------------ |
| Opration  | u8         | 1 byte       | Enum of four basic modify ops. |
| Key       | K(Generic) | Zero or Any  | The type of index key.         |
| Value     | V(Generic) | Zero or Any  | The type of storage value.     |
| Delta     | D(Generic) | Zero or Any  | The type of modify value.      |

+ Endian: Little Endian, rust native.
+ Codec: Using rust [zerocopy](https://docs.rs/zerocopy/latest/zerocopy/index.html) to encode/decode.

Note that since the data struct is defined by user, there is no placeholder for empty field. In an other word, Atom use a packed layout to store the data.
Therefore, these structs have the same memory layout:
+ Atom<u8,u16,()>
+ Atom<u8,(),u16>
+ Atom<u8,u8,u8>
+ Atom<u16,u8,()>
+ etc...



## Roadmap

- [x] Freeze Atom protocol .
- [ ] Use im asBasic implementation.
- [ ] Support async non-block op.
- [ ] Documentation.
- [ ] External Message queue support.
- [ ] Benchmark.
- [ ] Auto shrink inner data to make more density.

---

Written by RuofengX · Used in entropy-rs, the Game

License by MIT.
