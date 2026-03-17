use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use std::path::Path;

fn format_with_prism(source: &[u8]) {
    rubyfmt::format_buffer(source).expect("formatting failed");
}

fn collect_fixtures(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut fixtures = Vec::new();
    let base = Path::new("fixtures/large");

    for entry in fs::read_dir(dir).expect("failed to read dir") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            fixtures.extend(collect_fixtures(&path));
        } else if path.to_string_lossy().ends_with("_actual.rb") {
            let name = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .trim_end_matches("_actual.rb")
                .to_string();
            let source = fs::read(&path).expect("failed to read file");
            fixtures.push((name, source));
        }
    }

    fixtures
}

fn bench_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_tests");

    let files = [
        ("methods", "ci/methods_stress_test.rb"),
        ("array_literals", "ci/array_literals_stress_test.rb"),
        ("string_literals", "ci/string_literals_stress_test.rb"),
    ];

    for (name, path) in files {
        let source = fs::read(path).expect("failed to read file");
        group.bench_with_input(BenchmarkId::new("prism", name), &source, |b, source| {
            b.iter(|| format_with_prism(source));
        });
    }

    group.finish();
}

fn bench_large_fixtures(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_fixtures");

    let fixtures = collect_fixtures(Path::new("fixtures/large"));

    for (name, source) in &fixtures {
        group.bench_with_input(BenchmarkId::new("prism", name), source, |b, source| {
            b.iter(|| format_with_prism(source));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_stress_tests, bench_large_fixtures);
criterion_main!(benches);
