use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;

use fluent_bundle::FluentArgs;
use fluent_fallback::{types::L10nKey, Localization};
use fluent_testing::{get_scenarios, MockFileSystem};

fn preferences_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("localization/scenarios");

    for scenario in get_scenarios() {
        group.bench_function(format!("{}/format_messages_sync", scenario.name), |b| {
            b.iter(|| {
            })
        });
    }

    group.finish();
}

criterion_group!(benches, preferences_bench);
criterion_main!(benches);
