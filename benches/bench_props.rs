use criterion::{criterion_group, criterion_main, Criterion};
use retable::atom::{PropName, PropValue, EID};
use retable::db::Props;

fn set_new_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());

    c.bench_function("set_new", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.set(eid, key, value.clone());
            })
        })
    });
}

fn set_override_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    EID::range(100).for_each(|eid| {
        props.set(eid, key, value.clone());
    });

    c.bench_function("set_override", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.set(eid, key, value.clone());
            })
        })
    });
}

fn insert_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());

    c.bench_function("insert", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.insert(eid, key, value.clone());
            })
        })
    });
}

fn get_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    EID::range(100).for_each(|eid| {
        props.insert(eid, key, value.clone());
    });

    c.bench_function("get", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.get(eid, key);
            });
        })
    });
}

fn update_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    EID::range(100).for_each(|eid| {
        props.update(eid, key, value.clone());
    });

    c.bench_function("update", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.update(eid, key, value.clone());
            });
        })
    });
}

fn drop_benchmark(c: &mut Criterion) {
    let mut props = Props::new();
    let key = PropName::Infomation;
    let value = PropValue::Str("value".to_string());
    EID::range(100).for_each(|eid| {
        props.insert(eid, key, value.clone());
    });

    c.bench_function("drop", |b| {
        b.iter(|| {
            EID::range(100).for_each(|eid| {
                props.drop(eid, key);
            });
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
);
criterion_main!(props_benchmarks);
