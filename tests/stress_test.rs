use assert_cmd::cargo::CommandCargoExt;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn methods_stress_test() {
    stress_test("ci/methods_stress_test.rb", false).unwrap()
}

#[test]
fn arrays_stress_test() {
    stress_test("ci/array_literals_stress_test.rb", false).unwrap()
}

#[test]
fn string_literals_stress_test() {
    stress_test("ci/string_literals_stress_test.rb", false).unwrap()
}

#[test]
fn methods_prism_stress_test() {
    stress_test("ci/methods_stress_test.rb", true).unwrap()
}

#[test]
fn string_literals_prism_stress_test() {
    stress_test("ci/string_literals_stress_test.rb", true).unwrap()
}

#[test]
fn arrays_prism_stress_test() {
    stress_test("ci/array_literals_stress_test.rb", true).unwrap()
}

/// Test helper for running "stress tests", tests that
/// execute a ruby script before and after being formatted
/// to confirm that its behavior/output is the same.
fn stress_test(path: &str, prism: bool) -> Result<(), ()> {
    let methods_expected = {
        let output = Command::new("ruby").arg(path).output().unwrap();
        assert!(output.status.success());
        output.stdout
    };

    let methods_actual = {
        let mut command = Command::cargo_bin("rubyfmt-main").unwrap();
        command.arg(path);

        if prism {
            command.arg("--prism");
        }

        let rubyfmt_output = command.output().unwrap();

        assert!(rubyfmt_output.status.success());

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&rubyfmt_output.stdout).unwrap();

        let ruby_output = Command::new("ruby").arg(file.path()).output().unwrap();
        assert!(ruby_output.status.success());

        ruby_output.stdout
    };

    assert_eq!(methods_expected, methods_actual);

    Ok(())
}
