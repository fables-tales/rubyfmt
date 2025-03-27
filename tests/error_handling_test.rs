use std::io::Write;

use assert_cmd::{cargo::cargo_bin, Command};
use tempfile::NamedTempFile;

#[test]
fn test_syntax_error_stdin() {
    Command::new(cargo_bin("rubyfmt-main"))
        .write_stdin("a 1,2,,")
        .assert()
        .code(1)
        .failure();
}

#[test]
fn test_syntax_error_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2,,").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .assert()
        .code(0)
        .success();
}

#[test]
fn test_syntax_error_files() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a 1,2,,").unwrap();
    let mut file_two = NamedTempFile::new().unwrap();
    writeln!(file_two, "a(1, 2)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file_one.path())
        .arg(file_two.path())
        .assert()
        .code(0)
        .success();
}

#[test]
fn test_syntax_error_fail_fast() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2,,").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--fail-fast")
        .arg(file.path())
        .assert()
        .code(1)
        .failure();
}

#[test]
fn test_syntax_error_files_fail_fast() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a 1,2,,").unwrap();
    let mut file_two = NamedTempFile::new().unwrap();
    writeln!(file_two, "a(1, 2)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--fail-fast")
        .arg(file_one.path())
        .arg(file_two.path())
        .assert()
        .code(1)
        .failure();
}

#[test]
fn test_io_error_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2").unwrap();
    let mut perms = std::fs::metadata(file.path()).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(file.path(), perms).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .assert()
        .code(0)
        .success();
}

#[test]
fn test_io_error_files() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2").unwrap();
    let mut perms = std::fs::metadata(file.path()).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(file.path(), perms).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .arg("somefiledoesntexist.rb")
        .assert()
        .code(0)
        .success();
}

#[test]
fn test_io_error_file_fail_fast() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--fail-fast")
        .arg("somenonexistentfile.rb")
        .assert()
        .code(3)
        .failure();
}

#[test]
fn test_io_error_files_fail_fast() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--fail-fast")
        .arg(file.path())
        .arg("somefiledoesntexist.rb")
        .assert()
        .code(3)
        .failure();
}

#[test]
fn test_input_file_doesnt_exist() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--fail-fast")
        .arg("@doesntexist.txt")
        .assert()
        .code(3)
        .failure();
}
