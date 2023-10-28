use criterion::{black_box, criterion_group, criterion_main, Criterion};
use retable::{nested_index::PropValueSlab, atom::{EID, PropValue}};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn many_insert(n: u64) -> (){
    let slab: PropValueSlab = PropValueSlab::new();
    
    for i in 0..n{
        slab.insert(EID(i), PropValue::UInt(i)).unwrap();
    }
}
fn many_insert_get(n: u64) -> (){
    let slab: PropValueSlab = PropValueSlab::new();
    
    for i in 0..n{
        slab.insert(EID(i), PropValue::UInt(i)).unwrap();
    }
    for i in 0..n{
        drop(slab.get(EID(i)).unwrap().read().unwrap());
    }
}
fn many_insert_drop(n: u64) -> (){
    let slab: PropValueSlab = PropValueSlab::new();
    
    for i in 0..n{
        slab.insert(EID(i), PropValue::UInt(i)).unwrap();
    }
    for i in 0..n{
        slab.remove(EID(i)).unwrap();
    }
}
fn many_insert_update(n: u64) -> (){
    let slab: PropValueSlab = PropValueSlab::new();
    
    for i in 0..n{
        slab.insert(EID(i), PropValue::UInt(i)).unwrap();
    }
    for i in 0..n{
        let atom = slab.get(EID(i)).unwrap();
        *atom.write().unwrap() = PropValue::Str("我不是尧神".to_string());
    }
}
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("many_insert 10000", |b| b.iter(|| many_insert(black_box(10000))));
    c.bench_function("many_insert_get 10000", |b| b.iter(|| many_insert_get(black_box(10000))));
    c.bench_function("many_insert_drop 10000", |b| b.iter(|| many_insert_drop(black_box(10000))));
    c.bench_function("many_insert_update 10000", |b| b.iter(|| many_insert_update(black_box(10000))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);