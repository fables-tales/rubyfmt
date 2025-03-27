use std::{fs::read_to_string, process::Command};

use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};

#[macro_export]
macro_rules! fixture {
    ($test_name:ident, $path:expr) => {
        #[test]
        fn $test_name() {
            test_fixture($path)
        }
    };
}

pub fn test_fixture(name: &str) {
    let expected = read_to_string(format!("fixtures/{}_expected.rb", name)).unwrap();

    // Test if the formatting works as expected
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(format!("fixtures/{}_actual.rb", name))
        .assert()
        .success()
        .stdout(expected.clone());

    // Test if the formatting is idempotent
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(format!("fixtures/{}_expected.rb", name))
        .assert()
        .success()
        .stdout(expected);
}
