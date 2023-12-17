# Retable, WIP, NOT USE

A Rust library for Atomic-like double-indexed entity-attribute data structures.  


## Atom

Atom is a way to describe a single entity-attribute pair.

| Field Name   | Size(in bytes) | Description                                    |
| ------------ | -------------- | ---------------------------------------------- |
| EID          | 8bytes         | Entity ID, a unique ID for an entity.          |
| PropertyName | 8bytes         | A fixed size string to describe the attribute. |
| Value        | 32bytes        | A value to describe the attribute, see below.  |

* Endian: Little Endian, rust native.
* Codec: Using rust [Bincode](https://docs.rs/bincode/latest/bincode/) to encode/decode.

## Value Enum

1 byte to describe the value type enum.
31 bytes contain the data.

Supports the following types(in rust):
```rust
pub enum Value {
    Bool(bool),
    EID(EID),
    UInt(u64),
    Int(i64),
    Float(f64),
    UInt3([u64; 3]),
    Int3([i64; 3]),
    Float3([f64; 3]),
    UInt2([u64; 2]),
    Int2([i64; 2]),
    Float2([f64; 2]),
    Mark(Marker), // a 31-bytes length fix size string.
}

```

## Features 

- Double indexed by Entity ID and Property Name.
- Built on top of giants, [sled](https://docs.rs/sled/latest/sled/index.html) for database, [moka](https://docs.rs/moka/latest/moka/index.html) for cache and [rayon](https://docs.rs/rayon/latest/rayon/index.html) for parallel.
- Thread-safe
- No unsafe code

## Roadmap

- [x] Basic implementation.
- [x] Documentation.
- [ ] Benchmark.
- [ ] Use sparse index to fetch more dense in-memory performance.

---

Written by RuofengX · Used in entropy-rs, the Game

License by MIT.
