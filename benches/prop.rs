use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use retable::Prop;

fn set(prop: &Prop<i64, (f64, f64, f64)>, key: i64, value: (f64, f64, f64)) {
    prop.set(&key, value);
}

fn multi_set(c: &mut Criterion) {
    let mut group = c.benchmark_group("ranged_set");
    for key in (16..20).map(|x| 2_i64.pow(x)) {
        group.throughput(Throughput::Elements(key as u64));
        group.bench_with_input(BenchmarkId::from_parameter(key), &key, |b, &key| {
            b.iter(|| {
                let prop = Prop::new();
                (0..key)
                    .into_iter()
                    .for_each(|i| set(&prop, i, black_box((0.618, -0.618, 10086.0))));
            });
        });
    }
    group.finish();
}

fn set_get(prop: &Prop<i64, (f64, f64, f64)>, key: i64, value: (f64, f64, f64)) {
    prop.set(&key, value);
    let _ = prop.get(&key);
}

fn multi_set_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("ranged_set_get");
    for key in (16..20).map(|x| 2_i64.pow(x)) {
        let prop = Prop::new();
        group.throughput(Throughput::Elements(key as u64));
        group.bench_with_input(BenchmarkId::from_parameter(key), &key, |b, &key| {
            b.iter(|| {
                (0..key)
                    .into_iter()
                    .for_each(|i| set_get(&prop, i, black_box((0.618, -0.618, 10086.0))));
            });
        });
    }
    group.finish();
}

fn get_modify_set(prop: &Prop<i64, (f64, f64, f64)>, key: i64) {
    let old = prop.get(&key).unwrap();
    prop.set(&key, (old.0 + 1.0, old.1 + 1.0, old.2 + 1.0));
}

fn multi_get_modify_set(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_modify_set");
    for key in (16..20).map(|x| 2_i64.pow(x)) {
        let prop = Prop::new();
        (0..key).into_iter().for_each(|x| {
            let v = x as f64;
            prop.set(&x, (v, v, v));
        });

        group.throughput(Throughput::Elements(key as u64));
        group.bench_with_input(BenchmarkId::from_parameter(key), &key, |b, &key| {
            b.iter(|| {
                (0..key).into_iter().for_each(|i| get_modify_set(&prop, i));
            });
        });
    }
    group.finish();
}

criterion_group!(crud_benches, multi_set, multi_set_get);
criterion_group!(modify_benches, multi_get_modify_set);
criterion_main!(crud_benches, modify_benches);
