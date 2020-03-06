use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use std::collections::HashMap;
use std::fs;
use std::io;

use fluent_syntax::parser::lexer::Lexer;

fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

fn get_strings(tests: &[&'static str]) -> HashMap<&'static str, String> {
    let mut ftl_strings = HashMap::new();
    for test in tests {
        let path = format!("./benches/{}.ftl", test);
        ftl_strings.insert(*test, read_file(&path).expect("Couldn't load file"));
    }
    return ftl_strings;
}

fn lexer_bench(c: &mut Criterion) {
    let tests = &["simple", "preferences", "menubar"];
    let ftl_strings = get_strings(tests);

    c.bench_function_over_inputs(
        "lexer_bench",
        move |b, &name| {
            let source = &ftl_strings[name];
            b.iter(|| {
                let lexer = Lexer::new(source.as_bytes());
                let _: Vec<_> = lexer.collect();
            });
        },
        tests,
    );
}

fn get_ctxs(tests: &[&'static str]) -> HashMap<&'static str, Vec<String>> {
    let mut ftl_strings = HashMap::new();
    for test in tests {
        let paths = fs::read_dir(format!("./benches/contexts/{}", test)).unwrap();
        let strings = paths
            .into_iter()
            .map(|p| {
                let p = p.unwrap().path();
                let path = p.to_str().unwrap();
                read_file(path).unwrap()
            })
            .collect::<Vec<_>>();
        ftl_strings.insert(*test, strings);
    }
    return ftl_strings;
}

fn lexer_ctx_bench(c: &mut Criterion) {
    let tests = &["browser", "preferences"];
    let ftl_strings = get_ctxs(tests);

    c.bench_function_over_inputs(
        "lexer_ctx",
        move |b, &&name| {
            let sources = &ftl_strings[name];
            b.iter(|| {
                for source in sources {
                    let lexer = Lexer::new(source.as_bytes());
                    let _: Vec<_> = lexer.collect();
                }
            })
        },
        tests,
    );
}

criterion_group!(benches, lexer_bench, lexer_ctx_bench);
criterion_main!(benches);
