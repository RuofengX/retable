# Retable

WIP, DO NOT USE

A Rust library for Atomic-like double-indexed entity-attribute data structures.  


## Atom

Atom is a way to describe a single entity-attribute pair.

| Field Name   | Size(in bytes) | Description                                    |
| ------------ | -------------- | ---------------------------------------------- |
| EID          | 8bytes         | Entity ID, a unique ID for an entity.          |
| PropertyName | 8bytes         | A fixed size string to describe the attribute. |
| Value        | ??bytes        | A value to describe the attribute.             |

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
