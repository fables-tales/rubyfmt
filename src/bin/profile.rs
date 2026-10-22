use std::path::Path;
use std::time::{Duration, Instant};

fn collect_fixtures(dir: &Path) -> Vec<Vec<u8>> {
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(dir).expect("failed to read dir") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            fixtures.extend(collect_fixtures(&path));
        } else if path.to_string_lossy().ends_with("_actual.rb") {
            fixtures.push(std::fs::read(&path).expect("failed to read file"));
        }
    }
    fixtures
}

fn main() {
    let fixtures = collect_fixtures(Path::new("fixtures"));
    let duration = Duration::from_secs(30);
    let start = Instant::now();
    while start.elapsed() < duration {
        for source in &fixtures {
            rubyfmt::format_buffer(source).expect("formatting failed");
        }
    }
}
