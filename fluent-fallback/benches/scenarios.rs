use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use fluent_bundle::FluentArgs;
use fluent_fallback::testing::MockGenerator;
use fluent_fallback::Localization;
use fluent_testing::get_scenarios;

fn scenarios_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios");

    for scenario in get_scenarios() {
        let gen = MockGenerator::new(
            scenario
                .locales
                .iter()
                .map(|l| l.parse().unwrap())
                .collect(),
            scenario.file_sources.clone(),
        );

        let l10n_keys: Vec<(String, Option<FluentArgs>)> = scenario
            .queries
            .iter()
            .map(|q| {
                (
                    q.input.id.clone(),
                    q.input.args.as_ref().map(|args| {
                        let mut result = FluentArgs::new();
                        for arg in args.as_slice() {
                            result.add(arg.id.clone(), arg.value.clone().into());
                        }
                        result
                    }),
                )
            })
            .collect();

        group.bench_function(&format!("sync/{}", scenario.name), move |b| {
            b.iter(|| {
                let loc = Localization::with_generator(scenario.res_ids.clone(), true, gen.clone());
                let mut errors = vec![];
                for key in &l10n_keys {
                    let _ = loc.format_value_sync(&key.0, key.1.as_ref(), &mut errors);
                }
            })
        });
    }

    #[cfg(feature = "tokio")]
    {
        for scenario in get_scenarios() {
            let gen = MockGenerator::new(
                scenario
                    .locales
                    .iter()
                    .map(|l| l.parse().unwrap())
                    .collect(),
                scenario.file_sources.clone(),
            );

            let rt = tokio::runtime::Runtime::new().unwrap();

            group.bench_function(&format!("async/{}", scenario.name), move |b| {
                b.iter(|| {
                    let loc =
                        Localization::with_generator(scenario.res_ids.clone(), false, gen.clone());
                    let mut errors = vec![];
                    rt.block_on(async {
                        for query in scenario.queries.iter() {
                            let _ = loc.format_value(&query.input.id, None, &mut errors).await;
                        }
                    });
                })
            });
        }
    }
    group.finish();
}

criterion_group!(benches, scenarios_bench);
criterion_main!(benches);
