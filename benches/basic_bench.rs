extern crate std;

use core::hint::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use ini_roundtrip::Parser;

const DOCUMENT: &str = "\
key before = first section

[SECTION]
;this is a comment
Key = Value

# Another comment
[[whateven]][is this]
ugh
Actually a key[$d]

[other section]
k = V  ";

fn parse(doc: &str) {
    let value: std::vec::Vec<_> = Parser::new(black_box(doc)).collect();
    black_box(value);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ini-parser");
    group.significance_level(0.1).sample_size(30);
    group.bench_function("parse", |b| b.iter(|| parse(black_box(DOCUMENT))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
