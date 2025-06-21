use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use kiteticker_async_manager::{Mode, Tick};
use std::time::Duration;

fn benchmark_tick_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_creation");
    
    group.bench_function("create_tick", |b| {
        b.iter(|| {
            Tick {
                mode: Mode::LTP,
                instrument_token: 256265,
                ..Default::default()
            }
        })
    });
    
    group.finish();
}

fn benchmark_mode_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mode_operations");
    
    group.bench_function("mode_comparison", |b| {
        let mode1 = Mode::LTP;
        let mode2 = Mode::Quote;
        b.iter(|| {
            mode1 == mode2
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_tick_creation,
    benchmark_mode_operations
);
criterion_main!(benches);
