use std::ops::Add;

use rand::{self, Rng};
use criterion::{criterion_group, criterion_main, Criterion};
use retable::{PropStorage, PropValueSp};
use retable::atom::{PropName, PropValue, EID};
use retable::db::Props;

// use retable::{PropValueHash};
// type I = PropValueHash; // 40%~160% faster than SparseSet, when indexing
type I = PropValueSp; // 12% faster than FxHashMap, when calculate

fn set_new_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());

    c.bench_function("set_new", |b| {
        b.iter(|| {
            props.set(eid, key, value.clone());
        })
    });
}

fn set_override_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    props.set(eid, key, value.clone());

    c.bench_function("set_override", |b| {
        b.iter(|| {
            props.set(eid, key, value.clone());
        })
    });
}

fn insert_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());

    c.bench_function("insert", |b| {
        b.iter(|| {
            props.insert(eid, key, value.clone());
        })
    });
}

fn get_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    props.insert(eid, key, value.clone());

    c.bench_function("get", |b| {
        b.iter(|| {
            props.get(eid, key);
        })
    });
}

fn update_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    props.update(eid, key, value.clone());

    c.bench_function("update", |b| {
        b.iter(|| {
            props.update(eid, key, value.clone());
        })
    });
}

fn drop_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let eid = EID(1);
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    props.insert(eid, key, value.clone());
    props.insert(EID(2), key, value.clone()); // 删除非最后一个，这种场景更加常见

    c.bench_function("drop", |b| {
        b.iter(|| {
            props.drop(eid, key);
        })
    });
}

// 综合速度)
fn generate_random_array(min: f64, max: f64) -> [f64;3] {
    let mut rng = rand::thread_rng();
    let mut array = [0.0;3];

    for i in 0..3{
        let random_number = rng.gen_range(min..=max);
        array[i] = random_number;
    }
    array
}

fn parse_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    EID::range(100000).for_each(|eid|{
        props.insert(
            eid, 
            PropName::Pos,
            PropValue::Vec(generate_random_array(-1000000.0, 1000000.0).into())
        );
    });

    c.bench_function("综合测试", |b| {
        b.iter(|| {
            props.get_prop_mut(PropName::Pos).unwrap().tick(
                |value: &mut PropValue| {
                    if let PropValue::Vec(pos) = value{
                        *pos = pos.add([1.0,2.0,3.0].into());
                    } else {
                        panic!("错误的类型")
                    }
                    
                }
            );
        })
    });
}

criterion_group!(
    props_benchmarks,
    set_new_benchmark,
    set_override_benchmark,
    insert_benchmark,
    get_benchmark,
    update_benchmark,
    drop_benchmark,
    parse_benchmark,
);
criterion_main!(props_benchmarks);
