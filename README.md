# Retable

WIP, DO NOT USE

A Rust library for Atomic-like double-indexed entity-attribute data structures.  


## Atom

Atom is a way to describe a single key-value pair.

Atom use an UUID as the global type definition.

| Field Name   | Data Type | Size or MaxSize | Description                        |
| ------------ | --------- | --------------- | ---------------------------------- |
| Meta         | u64       | 8               |                                    |
| PropertyName | u8        | 1 bytes         |                                    |
| Index        | u64       | 8 bytes         | Key                                |
| Value        |           | <=2^64 bytes    | A value to describe the attribute. |

* Endian: Little Endian, rust native.
* Codec: Using rust [zerocopy](https://docs.rs/zerocopy/latest/zerocopy/index.html) to encode/decode.


## Roadmap

- [x] Basic implementation.
- [ ] Binlog to do persistent storage.
- [ ] Support async non-instent op.
- [ ] Documentation.
- [ ] Message queue support.
- [ ] Benchmark.
- [ ] Auto shrink inner data to make more density.

---

Written by RuofengX · Used in entropy-rs, the Game

License by MIT.
