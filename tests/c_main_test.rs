use std::{
    fs::read_to_string,
    io::Write,
    process::{Command, Stdio},
};

use assert_cmd::output::OutputOkExt;

#[test]
fn test_c_main() {
    let output = Command::new("make").arg("target/c_main_release").unwrap();
    assert!(output.status.success());

    let c_main = Command::new("./target/c_main_release")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c_main
        .stdin
        .as_ref()
        .unwrap()
        .write_all(
            read_to_string("fixtures/small/numbers_actual.rb")
                .unwrap()
                .as_bytes(),
        )
        .unwrap();

    let c_main_output = c_main.wait_with_output().unwrap();
    assert!(c_main_output.status.success());

    let c_main_output = dbg!(String::from_utf8(c_main_output.stdout).unwrap());
    let expected = read_to_string("fixtures/small/numbers_expected.rb").unwrap();

    assert_eq!(expected, c_main_output);
}
