use std::ops::Add;

use rand::{self, Rng};
use criterion::{criterion_group, criterion_main, Criterion};
use retable::{PropStorage, PropValueSp};
use retable::atom::{PropName, PropValue, EID, EntityProp};
use retable::db::Props;

// use retable::{PropValueHash};
// type I = PropValueHash; // 40%~160% faster than SparseSet, when indexing
type I = PropValueSp; // 15%~20% faster than FxHashMap, when calculate a lot of data

fn spawn_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    let mut entity_prop = EntityProp::default();
    entity_prop.insert(key, value);

    c.bench_function("spawn", |b| {
        b.iter(|| {
            props.spawn(entity_prop.clone());
        })
    });
}

fn get_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    let mut entity_prop = EntityProp::default();
    entity_prop.insert(key, value);
    let eid = props.spawn(entity_prop);

    c.bench_function("get", |b| {
        b.iter(|| {
            props.get(eid, &key);
        })
    });
}

fn update_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());

    let mut entity_prop = EntityProp::default();
    entity_prop.insert(key, value);

    let eid = props.spawn(entity_prop);
    let value = PropValue::Str("value_new".to_string());

    c.bench_function("update", |b| {
        b.iter(|| {
            props.update(eid, key, value.clone());
        })
    });
}

fn remove_benchmark(c: &mut Criterion) {
    let mut props = Props::<I>::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    let mut entity_prop = EntityProp::default();
    entity_prop.insert(key, value);
    assert_eq!(props.spawn(entity_prop.clone()), EID(0));
    assert_eq!(props.spawn(entity_prop.clone()), EID(1));

    c.bench_function("remove", |b| {
        b.iter(|| {
            props.remove(EID(0), key);
        })
    });
}

// 综合速度
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
    (0..1000000)
    .for_each(|_|{
        let key = PropName::Pos;
        let value = PropValue::Vec(generate_random_array(-1000000.0, 1000000.0).into());
        let mut entity_prop = EntityProp::default();
        entity_prop.insert(key, value);
        let _ = props.spawn(entity_prop);
    });

    c.bench_function("计算测试", |b| {
        b.iter(|| {
            props.get_prop_mut(&PropName::Pos).unwrap().tick(
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
    spawn_benchmark,
    get_benchmark,
    update_benchmark,
    remove_benchmark,
    parse_benchmark,
);
criterion_main!(props_benchmarks);
