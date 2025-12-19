use libtest_mimic::{Arguments, Trial};
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use assert_cmd::Command;

// These tests are disabled due to ripper issues
const DISABLED_RIPPER_TESTS: &[&'static str] = &[
    // TODO: The Ripper implementation does not currently support args forwarding with additional
    // arguments passed in.
    // https://github.com/fables-tales/rubyfmt/issues/474
    "small_args_forwarding_additional_args",
    "small_alias_global_var",
];

fn main() -> io::Result<()> {
    let args = Arguments::from_args();

    let mut tests = Vec::new();

    let fixtures = collect_fixtures("fixtures".into())?;

    tests.extend(fixtures.iter().filter_map(|f| f.to_trial(Flavor::Ripper)));
    tests.extend(fixtures.iter().filter_map(|f| f.to_trial(Flavor::Prism)));

    libtest_mimic::run(&args, tests).exit();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flavor {
    Ripper,
    Prism,
}

impl Flavor {
    fn to_str(&self) -> &'static str {
        match self {
            Flavor::Ripper => "ripper",
            Flavor::Prism => "prism",
        }
    }

    fn disabled_list(&self) -> &[&'static str] {
        match self {
            Flavor::Ripper => DISABLED_RIPPER_TESTS,
            Flavor::Prism => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FlavorRestriction {
    RipperOnly,
    PrismOnly,
    Both,
}

#[derive(Debug)]
struct Fixture {
    name: String,
    actual: Option<PathBuf>,
    expected: Option<PathBuf>,
    restriction: FlavorRestriction,
}

impl Fixture {
    fn to_trial(&self, flavor: Flavor) -> Option<Trial> {
        match (self.restriction, flavor) {
            (FlavorRestriction::RipperOnly, Flavor::Prism) => return None,
            (FlavorRestriction::PrismOnly, Flavor::Ripper) => return None,
            _ => {}
        }

        let name = format!("test_{}_{}", flavor.to_str(), self.name);
        let is_ignored = flavor.disabled_list().contains(&self.name.as_str());
        let actual = self.actual.clone();
        let expected = self.expected.clone();
        let trial = Trial::test(name.clone(), move || {
            let Some(actual) = actual else {
                return Result::Err(format!("Test {} is missing an _actual.rb file", name).into());
            };

            let Some(expected) = expected else {
                return Result::Err(
                    format!("Test {} is missing an _expected.rb file", name).into(),
                );
            };

            let expected_text = read_to_string(&expected)?;

            // Test if the formatting works as expected
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(actual.to_str().unwrap());

            if flavor == Flavor::Prism {
                cmd.arg("--prism");
            }

            cmd.assert().success().stdout(expected_text.clone());

            // Test if the formatting is idempotent
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(expected.to_str().unwrap());

            if flavor == Flavor::Prism {
                cmd.arg("--prism");
            }

            cmd.assert().success().stdout(expected_text);
            Ok(())
        })
        .with_ignored_flag(is_ignored);

        Some(trial)
    }
}

fn collect_fixtures(path: PathBuf) -> io::Result<Vec<Fixture>> {
    #[derive(Default)]
    struct Partial {
        actual: Option<PathBuf>,
        expected: Option<PathBuf>,
    }

    fn recurse(
        results: &mut Vec<Fixture>,
        path: &Path,
        restriction: FlavorRestriction,
    ) -> io::Result<()> {
        let mut partial: HashMap<String, Partial> = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name();
                let dir_name_str = dir_name.to_str().unwrap_or("");
                let child_restriction = match dir_name_str {
                    "ripper" => FlavorRestriction::RipperOnly,
                    "prism" => FlavorRestriction::PrismOnly,
                    _ => restriction,
                };
                recurse(results, &path, child_restriction)?;
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
                restriction,
            });
        }

        Ok(())
    }

    let mut results = Vec::new();
    recurse(&mut results, &path, FlavorRestriction::Both)?;
    Ok(results)
}
