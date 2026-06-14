use assert_cmd::cargo::CommandCargoExt;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn squiggly_heredoc_whitespace_only_lines_do_not_impact_indentation() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "  2 spaces\n",
            " \n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "2 spaces\n\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "    2 spaces\n",
            "\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_one_tab_equals_eight_spaces() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "        8 spaces\n",
            "\t1 tab\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "8 spaces\n1 tab\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "    8 spaces\n",
            "    1 tab\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_one_space_plus_one_tab_equals_eight_spaces() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "         9 spaces\n",
            " \t1 space plus 1 tab\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        " 9 spaces\n1 space plus 1 tab\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "     9 spaces\n",
            "    1 space plus 1 tab\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_one_tab_plus_one_space_equals_nine_spaces() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "         9 spaces\n",
            "\t 1 tab plus 1 space\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "9 spaces\n1 tab plus 1 space\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "    9 spaces\n",
            "    1 tab plus 1 space\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_one_tab_is_greater_than_seven_spaces() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "       7 spaces\n",
            "\t1 tab\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "7 spaces\n\t1 tab\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "    7 spaces\n",
            "    \t1 tab\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_a_single_whitespace_line_is_preserved() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZFOOBARBAZ\n",
            " \n",
            "FOOBARBAZFOOBARBAZ\n",
            ")\n",
        ),
        " \n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZFOOBARBAZ\n",
            " \n",
            "  FOOBARBAZFOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn normal_heredoc_a_single_whitespace_line_is_preserved() {
    string_test(
        concat!(
            "puts(\n",
            "<<-FOOBARBAZFOOBARBAZ\n",
            " \n",
            "FOOBARBAZFOOBARBAZ\n",
            ")\n",
        ),
        " \n",
        concat!(
            "puts(\n",
            "  <<-FOOBARBAZFOOBARBAZ\n",
            " \n",
            "  FOOBARBAZFOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn squiggly_heredoc_trailing_whitespace_is_preserved() {
    string_test(
        concat!(
            "puts(\n",
            "<<~FOOBARBAZ\n",
            "  foo \n",
            "  bar\t\n",
            "   \n",
            "\t\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "foo \nbar\t\n \n\t\n",
        concat!(
            "puts(\n",
            "  <<~FOOBARBAZ\n",
            "    foo \n",
            "    bar\t\n",
            "     \n",
            "    \t\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

#[test]
fn normal_heredoc_trailing_whitespace_is_preserved() {
    string_test(
        concat!(
            "puts(\n",
            "<<-FOOBARBAZ\n",
            "foo \n",
            "bar\t\n",
            " \n",
            "\t\n",
            "FOOBARBAZ\n",
            ")\n",
        ),
        "foo \nbar\t\n \n\t\n",
        concat!(
            "puts(\n",
            "  <<-FOOBARBAZ\n",
            "foo \n",
            "bar\t\n",
            " \n",
            "\t\n",
            "  FOOBARBAZ\n",
            ")\n",
        ),
    );
}

/// Runs a single string test:
///
/// 1. Runs `ruby` on `input` and verifies it produces `ruby_output`.
/// 2. Runs `rubyfmt` on `input` and verifies it produces `rubyfmt_output`.
/// 3. Runs `ruby` on `rubyfmt_output` and verifies it produces `ruby_output`.
fn string_test(input: &str, ruby_output: &str, rubyfmt_output: &str) {
    fn pipe_to(mut cmd: Command, input: &[u8]) -> std::process::Output {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().expect("failed to spawn process");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input)
            .expect("failed to write to stdin");
        child
            .wait_with_output()
            .expect("failed to wait for process")
    }

    // Verify original behaviour with ruby.
    let original_ruby = pipe_to(Command::new("ruby"), input.as_bytes());
    assert!(
        original_ruby.status.success(),
        "ruby failed on input:\n{input}\nStderr: {}",
        String::from_utf8_lossy(&original_ruby.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&original_ruby.stdout),
        ruby_output,
        "ruby output before formatting does not match expected output"
    );

    // Verify rubyfmt output.
    let rubyfmt_cmd = Command::cargo_bin("rubyfmt-main").unwrap();
    let rubyfmt_result = pipe_to(rubyfmt_cmd, input.as_bytes());
    assert!(
        rubyfmt_result.status.success(),
        "rubyfmt failed on input:\n{input}\nStderr: {}",
        String::from_utf8_lossy(&rubyfmt_result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&rubyfmt_result.stdout),
        rubyfmt_output,
        "rubyfmt output does not match expected formatted output"
    );

    // Verify `ruby` after formatting.
    let formatted_ruby = pipe_to(Command::new("ruby"), &rubyfmt_result.stdout);
    assert!(
        formatted_ruby.status.success(),
        "ruby failed on formatted output:\n{}\nStderr: {}",
        rubyfmt_output,
        String::from_utf8_lossy(&formatted_ruby.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&formatted_ruby.stdout),
        ruby_output,
        "ruby output after formatting does not match expected output"
    );
}
