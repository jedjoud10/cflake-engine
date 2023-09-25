use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn benchmark(c: &mut Criterion) {
}

criterion_group!(benches, benchmark);
criterion_main!(benches);