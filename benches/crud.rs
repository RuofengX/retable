use criterion::{criterion_group, criterion_main, Criterion};
use retable::dense::Dense;
use retable::protocol::{Atomic, MergeAssign};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes};

#[derive(Debug, AsBytes, FromBytes, FromZeroes, Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct MockValue(pub u64);

#[derive(Debug, AsBytes, FromBytes, FromZeroes, Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct MockDelta(pub u64);

impl MergeAssign for MockValue {
    type Delta = MockDelta;

    fn merge(&mut self, delta: Self::Delta) {
        self.0 += delta.0
    }
}

fn bench_dense(c: &mut Criterion) {
    let prop = Dense::<u64, MockValue, MockDelta>::default();
    c.bench_function("1M u64 set_set_merge_set", |b| {
        b.iter(|| {
            (0..1000000).for_each(|x| {
                let value = MockValue(x);
                let value2 = MockValue(x + 1);
                let delta = MockDelta(1);
                let value3 = MockValue(x + 2);

                prop.set(&x, Some(&value));
                assert_eq!(prop.get(&x).unwrap(), value);

                prop.set(&x, Some(&value2));
                assert_eq!(prop.get(&x).unwrap(), value2);

                prop.merge(&x, &delta);
                assert_eq!(prop.get(&x).unwrap(), value3);

                prop.set(&x, None);
                assert!(prop.get(&x).is_none());
            })
        })
    });
}
fn bench_dense_unsafe(c: &mut Criterion) {
    let prop = Dense::<u64, MockValue, MockDelta>::default();
    c.bench_function("1M u64 unsafe_crumd", |b| {
        b.iter(|| {
            (0..1000000).for_each(|x| {
                let value = MockValue(x);
                let value2 = MockValue(x + 1);
                let delta = MockDelta(1);
                let value3 = MockValue(x + 2);
                unsafe {
                    prop.create_unchecked(&x, &value);
                    assert_eq!(prop.read_unchecked(&x), value);

                    prop.update_unchecked(&x, &value2);
                    assert_eq!(prop.read_unchecked(&x), value2);

                    prop.merge_unchecked(&x, &delta);
                    assert_eq!(prop.read_unchecked(&x), value3);

                    prop.delete_unchecked(&x);
                };
            })
        })
    });
}

fn bench_dense_set(c: &mut Criterion) {
    let prop = Dense::<u64, MockValue, MockDelta>::default();
    (0..1000000).for_each(|x| {
        let value = MockValue(x);
        prop.set(&x, Some(&value));
    });
    c.bench_function("1M u64 get_set udpate", |b| {
        b.iter(|| {
            (0..1000000).for_each(|x| {
                let mut value = prop.get(&x).unwrap();
                value.0 += 1;
                prop.set(&x, Some(&value));
            });
        })
    });
}
fn bench_dense_merge(c: &mut Criterion) {
    let prop = Dense::<u64, MockValue, MockDelta>::default();
    (0..1000000).for_each(|x| {
        let value = MockValue(x);
        prop.set(&x, Some(&value));
    });

    c.bench_function("1M u64 merge", |b| {
        b.iter(|| {
            (0..1000000).for_each(|x| {
                let delta= MockDelta(x);
                unsafe { prop.merge_unchecked(&x, &delta) };
            });
        })
    });
}

criterion_group!(crud_benches, bench_dense, bench_dense_unsafe);
criterion_group!(perf_benches, bench_dense_set, bench_dense_merge);
criterion_main!(crud_benches, perf_benches);
