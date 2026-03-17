use libtest_mimic::{Arguments, Trial};
use std::{
    collections::HashMap,
    fs::{self, read},
    io,
    path::{Path, PathBuf},
};

use assert_cmd::Command;

fn main() -> io::Result<()> {
    let args = Arguments::from_args();

    let mut tests = Vec::new();

    let fixtures = collect_fixtures("fixtures".into())?;

    tests.extend(fixtures.iter().map(|f| f.to_trial()));

    libtest_mimic::run(&args, tests).exit();
}

#[derive(Debug)]
struct Fixture {
    name: String,
    actual: Option<PathBuf>,
    expected: Option<PathBuf>,
}

impl Fixture {
    fn to_trial(&self) -> Trial {
        let name = format!("test_{}", self.name);
        let actual = self.actual.clone();
        let expected = self.expected.clone();
        Trial::test(name.clone(), move || {
            let Some(actual) = actual else {
                return Result::Err(format!("Test {} is missing an _actual.rb file", name).into());
            };

            let Some(expected) = expected else {
                return Result::Err(
                    format!("Test {} is missing an _expected.rb file", name).into(),
                );
            };

            let expected_text = read(&expected)?;

            // Test if the formatting works as expected
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(actual.to_str().unwrap());

            cmd.assert().success().stdout(expected_text.clone());

            // Test if the formatting is idempotent
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(expected.to_str().unwrap());

            cmd.assert().success().stdout(expected_text);
            Ok(())
        })
    }
}

fn collect_fixtures(path: PathBuf) -> io::Result<Vec<Fixture>> {
    #[derive(Default)]
    struct Partial {
        actual: Option<PathBuf>,
        expected: Option<PathBuf>,
    }

    fn recurse(results: &mut Vec<Fixture>, path: &Path) -> io::Result<()> {
        let mut partial: HashMap<String, Partial> = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type()?.is_dir() {
                recurse(results, &path)?;
                continue;
            }

            let file = path.to_str().unwrap();
            if file.ends_with("_actual.rb") {
                let len = file.len() - "_actual.rb".len();
                let prefix = file[0..len].to_owned();
                let p = partial.entry(prefix).or_default();
                assert!(p.actual.is_none());
                p.actual = Some(path);
            } else if file.ends_with("_expected.rb") {
                let len = file.len() - "_expected.rb".len();
                let prefix = file[0..len].to_owned();
                let p = partial.entry(prefix).or_default();
                assert!(p.expected.is_none());
                p.expected = Some(path);
            }
        }

        for (mut k, v) in partial {
            k.drain(0.."fixtures/".len());
            results.push(Fixture {
                name: k.replace('/', "_"),
                actual: v.actual,
                expected: v.expected,
            });
        }

        Ok(())
    }

    let mut results = Vec::new();
    recurse(&mut results, &path)?;
    Ok(results)
}
