use std::collections::{BTreeMap, HashMap};

use splay::SplayMap;

use crab_bucket::splay::Splay;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rng();
    let mut nums1: Vec<i32> = (1..100000).collect();
    let mut nums2: Vec<i32> = (1..100000).collect();
    nums1.shuffle(&mut rng);
    nums2.shuffle(&mut rng);
    nums1.truncate(50000);
    nums2.truncate(100);
    nums2 = nums2.iter().cycle().take(50000).map(|x| *x).collect();
    let nums1 = nums1;
    let nums2 = nums2;
    c.bench_function("get and set splay", |b| {
        b.iter(|| {
            let mut t: Splay<i32, i32> = Splay::new();
            for n in nums1.clone() {
                t.set(n, n);
            }
            for n in nums2.clone() {
                black_box(t.get(n));
            }
        })
    });
    c.bench_function("get and set hashmap", |b| {
        b.iter(|| {
            let mut t: HashMap<i32, i32> = HashMap::new();
            for n in nums1.clone() {
                t.insert(n, n);
            }
            for n in nums2.clone() {
                black_box(t.get(&n));
            }
        })
    });
    c.bench_function("get and set btreemap", |b| {
        b.iter(|| {
            let mut t: BTreeMap<i32, i32> = BTreeMap::new();
            for n in nums1.clone() {
                t.insert(n, n);
            }
            for n in nums2.clone() {
                black_box(t.get(&n));
            }
        })
    });
    c.bench_function("get and set splaymap", |b| {
        b.iter(|| {
            let mut t: SplayMap<i32, i32> = SplayMap::new();
            for n in nums1.clone() {
                t.insert(n, n);
            }
            for n in nums2.clone() {
                black_box(t.get(&n));
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
