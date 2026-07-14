use assert_cmd::Command;
use std::{
    fs::{self, create_dir, read_to_string},
    io::Write,
};
use tempfile::{NamedTempFile, tempdir};

#[test]
fn test_simple_stdout() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2,3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_stdout() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_stdout_respects_opt_in_header() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--header-opt-in")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a 1,2,3")
        .code(0)
        .success();
}

#[test]
fn test_stdin_stdout_respects_opt_out_header() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--header-opt-out")
        .write_stdin("# rubyfmt:false\na 1,2,3")
        .assert()
        .stdout("# rubyfmt:false\na 1,2,3")
        .code(0)
        .success();
}

#[test]
fn test_i_flag() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/bees").unwrap();
    create_dir(dir_name.clone() + "/bees/sub").unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/bees")
        .unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/bees")
        .unwrap();
    let mut file_three: NamedTempFile = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/bees/sub")
        .unwrap();
    let mut file_four = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone())
        .unwrap();
    writeln!(file_one, "a 1,2,3").unwrap();
    writeln!(file_two, "a 1,2,4").unwrap();
    writeln!(file_three, "a 1,2,5").unwrap();
    writeln!(file_four, "a 1,2,6").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(dir.path())
        .arg("-i")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!(read_to_string(file_one.path()).unwrap(), "a(1, 2, 3)\n");
    assert_eq!(read_to_string(file_two.path()).unwrap(), "a(1, 2, 4)\n");
    assert_eq!(read_to_string(file_three.path()).unwrap(), "a(1, 2, 5)\n");
    assert_eq!(read_to_string(file_four.path()).unwrap(), "a(1, 2, 6)\n");
}

#[test]
fn test_i_flag_no_changes() {
    let mut unchanged_file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile()
        .unwrap();
    writeln!(unchanged_file, "a(1, 2, 3)").unwrap();
    let unchanged_file_start_mtime = std::fs::metadata(unchanged_file.path())
        .unwrap()
        .modified()
        .unwrap();
    let mut changed_file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile()
        .unwrap();
    writeln!(changed_file, "a 1,2,3").unwrap();
    let changed_file_start_mtime = std::fs::metadata(changed_file.path())
        .unwrap()
        .modified()
        .unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(unchanged_file.path())
        .arg(changed_file.path())
        .arg("-i")
        .assert()
        .stdout("")
        .code(0)
        .success();
    let unchanged_file_end_mtime = std::fs::metadata(unchanged_file.path())
        .unwrap()
        .modified()
        .unwrap();
    let changed_file_end_mtime = std::fs::metadata(changed_file.path())
        .unwrap()
        .modified()
        .unwrap();

    assert_eq!(unchanged_file_start_mtime, unchanged_file_end_mtime);
    assert_ne!(changed_file_start_mtime, changed_file_end_mtime)
}

#[test]
fn test_check_flag_with_no_changes() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2,3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .arg("--check")
        .assert()
        .stdout(format!(
            "--- {}
+++ {}
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
",
            file.path().to_str().unwrap(),
            file.path().to_str().unwrap()
        ))
        .code(5)
        .failure();
}

#[test]
fn test_check_flag_multiple_files_with_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a 1,2,3").unwrap();
    let mut file_two = NamedTempFile::new().unwrap();
    writeln!(file_two, "a 4,5,6,7").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file_one.path())
        .arg(file_two.path())
        .arg("--check")
        .assert()
        .stdout(format!(
            "--- {}
+++ {}
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
--- {}
+++ {}
@@ -1 +1 @@
-a 4,5,6,7
+a(4, 5, 6, 7)
",
            file_one.path().to_str().unwrap(),
            file_one.path().to_str().unwrap(),
            file_two.path().to_str().unwrap(),
            file_two.path().to_str().unwrap()
        ))
        .code(5)
        .failure();
}

#[test]
fn test_check_flag_directory_with_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/inner").unwrap();

    let mut file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    writeln!(file, "a 1,2,3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(dir_name + "/inner")
        .arg("--check")
        .assert()
        .stdout(format!(
            "--- {}
+++ {}
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
",
            file.path().to_str().unwrap(),
            file.path().to_str().unwrap()
        ))
        .code(5)
        .failure();
}

#[test]
fn test_check_flag_input_file_with_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/inner").unwrap();

    let mut file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    writeln!(file, "a 1,2,3").unwrap();
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "{}", file.path().to_str().unwrap()).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("@".to_string() + input_file.path().to_str().unwrap())
        .arg("--check")
        .assert()
        .stdout(format!(
            "--- {}
+++ {}
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
",
            file.path().to_str().unwrap(),
            file.path().to_str().unwrap()
        ))
        .code(5)
        .failure();
}

#[test]
fn test_check_flag_respects_opt_in_header() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--check")
        .arg("--header-opt-in")
        .write_stdin("# rubyfmt: true\na 1,2,3\n")
        .assert()
        .stdout(
            "--- stdin
+++ stdin
@@ -1,2 +1,2 @@
 # rubyfmt: true
-a 1,2,3
+a(1, 2, 3)
",
        )
        .code(5)
        .failure();
}

#[test]
fn test_check_flag_respects_opt_out_header() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--check")
        .arg("--header-opt-out")
        .write_stdin("# rubyfmt: false\na 1,2,3\n")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_check_flag_without_changes() {
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--check")
        .write_stdin("a(1, 2, 3)\n")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_check_flag_multiple_files_without_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone())
        .unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone())
        .unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();
    writeln!(file_two, "a(4, 5, 6, 7)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file_one.path())
        .arg(file_two.path())
        .arg("--check")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_check_flag_directory_without_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/inner").unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(dir_name)
        .arg("--check")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_check_flag_input_file_without_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/inner").unwrap();

    let mut file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    writeln!(file, "a(1, 2, 3)").unwrap();
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "{}", file.path().to_str().unwrap()).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("@".to_string() + input_file.path().to_str().unwrap())
        .arg("--check")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_format_with_changes() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1, 2, 3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(file.path())
        .assert()
        .stdout("")
        .code(0)
        .success();
    assert_eq!("a(1, 2, 3)\n", read_to_string(file.path()).unwrap());
}

#[test]
fn test_format_multiple_files_with_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    let mut file_two = NamedTempFile::new().unwrap();
    writeln!(file_two, "a 4, 5, 6").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(file_one.path())
        .arg(file_two.path())
        .assert()
        .stdout("")
        .code(0)
        .success();
    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a(4, 5, 6)\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn test_format_directory_with_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();
    create_dir(dir_name.clone() + "/inner").unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone() + "/inner")
        .unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    writeln!(file_two, "a 4, 5, 6").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(dir_name)
        .arg("-i")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a(4, 5, 6)\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn format_input_file_with_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "{}", file_one.path().to_str().unwrap()).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg("@".to_string() + input_file.path().to_str().unwrap())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
}

#[test]
fn format_input_file_without_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "{}", file_one.path().to_str().unwrap()).unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg("@".to_string() + input_file.path().to_str().unwrap())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
}

#[test]
fn test_check_flag_with_syntax_error() {
    let mut file = NamedTempFile::new().unwrap();
    // Use actually invalid Ruby syntax (incomplete def statement)
    writeln!(file, "def (").unwrap();

    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(file.path())
        .arg("--check")
        .assert()
        .code(1)
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("syntax error"),
        "Expected stderr to contain 'syntax error', got: {}",
        stderr
    );
}

#[test]
fn test_check_flag_stdin_with_syntax_error() {
    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--check")
        .write_stdin("def (")
        .assert()
        .code(1)
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("syntax error"),
        "Expected stderr to contain 'syntax error', got: {}",
        stderr
    );
}

#[test]
fn test_format_respects_opt_in_header() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "# rubyfmt: true\na 1, 2, 3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--header-opt-in")
        .arg("-i")
        .arg(file_one.path())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!(
        "# rubyfmt: true\na(1, 2, 3)\n",
        read_to_string(file_one.path()).unwrap()
    );
}

#[test]
fn test_format_respects_opt_out_header() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "# rubyfmt: false\na 1, 2, 3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--header-opt-out")
        .arg("-i")
        .arg(file_one.path())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!(
        "# rubyfmt: false\na 1, 2, 3\n",
        read_to_string(file_one.path()).unwrap()
    );
}

#[test]
fn test_format_without_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(file_one.path())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
}

#[test]
fn test_format_multiple_files_without_changes() {
    let mut file_one = NamedTempFile::new().unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();
    let mut file_two = NamedTempFile::new().unwrap();
    writeln!(file_two, "a(4, 5, 6)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(file_one.path())
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a(4, 5, 6)\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn test_format_directory_without_changes() {
    let dir = tempdir().unwrap();
    let dir_name = dir.path().to_str().unwrap().to_owned();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone())
        .unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir_name.clone())
        .unwrap();
    writeln!(file_one, "a(1, 2, 3)").unwrap();
    writeln!(file_two, "a(4, 5, 6)").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(dir_name)
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a(4, 5, 6)\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn test_includes_gitignored() {
    let dir = tempdir().unwrap();
    // Fake a git repo
    create_dir(dir.path().join(".git")).unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_two, "a 4, 5, 6").unwrap();
    fs::write(
        dir.path().join(".gitignore"),
        file_two.path().file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        format!("{}", file_two.path().file_name().unwrap().to_str().unwrap()),
        read_to_string(dir.path().join(".gitignore")).unwrap()
    );

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg("--")
        .arg(".")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a 4, 5, 6\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn test_respects_rubyfmtignore() {
    let dir = tempdir().unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_two, "a 4, 5, 6").unwrap();
    fs::write(
        dir.path().join(".rubyfmtignore"),
        file_two.path().file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg(".")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a 4, 5, 6\n", read_to_string(file_two.path()).unwrap());
}

#[test]
fn test_respects_rubyfmtignore_with_multiple_directory_arguments() {
    for arguments in [["app", "db"], ["db", "app"]] {
        let dir = tempdir().unwrap();
        create_dir(dir.path().join("app")).unwrap();
        create_dir(dir.path().join("db")).unwrap();

        let app_file = dir.path().join("app/application.rb");
        let schema_file = dir.path().join("db/schema.rb");
        fs::write(&app_file, "a 1, 2, 3\n").unwrap();
        fs::write(&schema_file, "a 4, 5, 6\n").unwrap();
        fs::write(dir.path().join(".rubyfmtignore"), "db/schema.rb\n").unwrap();

        Command::cargo_bin("rubyfmt-main")
            .unwrap()
            .current_dir(dir.path())
            .arg("-i")
            .args(arguments)
            .assert()
            .stdout("")
            .code(0)
            .success();

        assert_eq!("a(1, 2, 3)\n", read_to_string(app_file).unwrap());
        assert_eq!("a 4, 5, 6\n", read_to_string(schema_file).unwrap());
    }
}

#[test]
fn test_respects_gitignore() {
    let dir = tempdir().unwrap();
    // Fake a git repo
    create_dir(dir.path().join(".git")).unwrap();

    let mut file_one = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_one, "a 1, 2, 3").unwrap();
    let mut file_two = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file_two, "a 4, 5, 6").unwrap();
    fs::write(
        dir.path().join(".gitignore"),
        file_two.path().file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg(".")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(file_one.path()).unwrap());
    assert_eq!("a 4, 5, 6\n", read_to_string(file_two.path()).unwrap());
}

// Reproduces https://github.com/BurntSushi/ripgrep/issues/829 for .gitignore
#[test]
fn test_respects_gitignore_with_subdirectory_argument() {
    let dir = tempdir().unwrap();
    // Fake a git repo
    create_dir(dir.path().join(".git")).unwrap();

    create_dir(dir.path().join("a")).unwrap();
    create_dir(dir.path().join("a").join("b")).unwrap();

    // Create a file in a/ (should be formatted)
    let file_one_path = dir.path().join("a").join("test.rb");
    fs::write(&file_one_path, "a 1, 2, 3\n").unwrap();

    // Create a file in a/b/ (should be ignored)
    let file_two_path = dir.path().join("a").join("b").join("test.rb");
    fs::write(&file_two_path, "a 4, 5, 6\n").unwrap();

    fs::write(dir.path().join(".gitignore"), "/a/b/\n").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg("a")
        .assert()
        .stdout("")
        .code(0)
        .success();

    // a/test.rb should be formatted
    assert_eq!("a(1, 2, 3)\n", read_to_string(&file_one_path).unwrap());
    // a/b/test.rb is gitignored
    assert_eq!("a 4, 5, 6\n", read_to_string(&file_two_path).unwrap());
}

// Reproduces https://github.com/BurntSushi/ripgrep/issues/829 for .rubyfmtignore
#[test]
fn test_respects_rubyfmtignore_with_subdirectory_argument() {
    let dir = tempdir().unwrap();

    // Create nested directory structure: a/b/
    create_dir(dir.path().join("a")).unwrap();
    create_dir(dir.path().join("a").join("b")).unwrap();

    // Create a file in a/ (should be formatted)
    let file_one_path = dir.path().join("a").join("test.rb");
    fs::write(&file_one_path, "a 1, 2, 3\n").unwrap();

    // Create a file in a/b/ (should be ignored)
    let file_two_path = dir.path().join("a").join("b").join("test.rb");
    fs::write(&file_two_path, "a 4, 5, 6\n").unwrap();

    fs::write(dir.path().join(".rubyfmtignore"), "/a/b/\n").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg("a")
        .assert()
        .stdout("")
        .code(0)
        .success();

    assert_eq!("a(1, 2, 3)\n", read_to_string(&file_one_path).unwrap());
    assert_eq!("a 4, 5, 6\n", read_to_string(&file_two_path).unwrap());
}

#[test]
fn test_formats_non_rb_files() {
    let mut file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rake")
        .tempfile()
        .unwrap();
    writeln!(file, "a 1, 2, 3").unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-i")
        .arg(file.path())
        .assert()
        .stdout("")
        .code(0)
        .success();
    assert_eq!("a(1, 2, 3)\n", read_to_string(file.path()).unwrap());
}

#[test]
fn test_version_uses_rubyfmt_name() {
    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-V")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("rubyfmt "),
        "Version output should start with 'rubyfmt ', got: {stdout}"
    );
    assert!(
        !stdout.contains("rubyfmt-main"),
        "Version output should not contain 'rubyfmt-main', got: {stdout}"
    );
}

#[test]
fn test_help_uses_rubyfmt_name() {
    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("-h")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage: rubyfmt"),
        "Help output should contain 'Usage: rubyfmt', got: {stdout}"
    );
    assert!(
        !stdout.contains("rubyfmt-main"),
        "Help output should not contain 'rubyfmt-main', got: {stdout}"
    );
}

#[test]
fn test_error_uses_rubyfmt_name() {
    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--not-a-real-flag")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("rubyfmt"),
        "Error output should contain 'rubyfmt', got: {stderr}"
    );
    assert!(
        !stderr.contains("rubyfmt-main"),
        "Error output should not contain 'rubyfmt-main', got: {stderr}"
    );
}

#[test]
fn test_stdin_filepath_respects_rubyfmtignore() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".rubyfmtignore"), "ignored.rb").unwrap();

    // File matching .rubyfmtignore pattern should not be formatted, but output unchanged
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a 1,2,3")
        .code(0)
        .success();

    // File not matching .rubyfmtignore pattern should be formatted
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("not_ignored.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_filepath_respects_gitignore() {
    let dir = tempdir().unwrap();
    // Fake a git repo
    create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".gitignore"), "ignored.rb").unwrap();

    // File matching .gitignore pattern should not be formatted, but output unchanged
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a 1,2,3")
        .code(0)
        .success();

    // File matching .gitignore but with --include-gitignored should be formatted
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored.rb")
        .arg("--include-gitignored")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_filepath_uses_path_in_check_output() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--check")
        .arg("--stdin-filepath")
        .arg("lib/foo.rb")
        .write_stdin("a 1,2,3\n")
        .assert()
        .stdout(
            "--- lib/foo.rb
+++ lib/foo.rb
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
",
        )
        .code(5)
        .failure();
}

#[test]
fn test_stdin_filepath_respects_rubyfmtignore_directory_pattern() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".rubyfmtignore"), "ignored/").unwrap();

    // File in ignored directory should not be formatted, but output unchanged
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored/foo.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a 1,2,3")
        .code(0)
        .success();

    // File not in ignored directory should be formatted
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("not_ignored/foo.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_filepath_respects_gitignore_directory_pattern() {
    let dir = tempdir().unwrap();
    create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".gitignore"), "ignored/").unwrap();

    // File in ignored directory should not be formatted, but output unchanged
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored/foo.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a 1,2,3")
        .code(0)
        .success();

    // File in ignored directory with --include-gitignored should be formatted
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--stdin-filepath")
        .arg("ignored/foo.rb")
        .arg("--include-gitignored")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("a(1, 2, 3)\n")
        .code(0)
        .success();
}

#[test]
fn test_stdin_filepath_check_mode_ignored_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".rubyfmtignore"), "ignored.rb").unwrap();

    // Check mode with ignored file should produce no output and exit 0
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("--check")
        .arg("--stdin-filepath")
        .arg("ignored.rb")
        .write_stdin("a 1,2,3")
        .assert()
        .stdout("")
        .code(0)
        .success();
}

#[test]
fn test_stdin_filepath_conflicts_with_file_paths() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a 1,2,3").unwrap();

    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--stdin-filepath")
        .arg("lib/foo.rb")
        .arg(file.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--stdin-filepath") && stderr.contains("cannot be used with"),
        "Expected error about --stdin-filepath conflict, got: {}",
        stderr
    );
}

#[test]
fn test_stdin_filepath_conflicts_with_in_place() {
    let output = Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg("--stdin-filepath")
        .arg("lib/foo.rb")
        .arg("-i")
        .write_stdin("a 1,2,3")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--stdin-filepath") && stderr.contains("cannot be used with"),
        "Expected error about --stdin-filepath conflict with -i, got: {}",
        stderr
    );
}

#[test]
fn test_explicit_file_path_ignores_rubyfmtignore() {
    // When a file is passed explicitly (not via directory traversal),
    // ignore patterns should not apply - the user explicitly asked for this file.
    let dir = tempdir().unwrap();

    let mut file = tempfile::Builder::new()
        .prefix("rubyfmt")
        .suffix(".rb")
        .tempfile_in(dir.path())
        .unwrap();
    writeln!(file, "a 1,2,3").unwrap();

    fs::write(
        dir.path().join(".rubyfmtignore"),
        file.path().file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();

    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .current_dir(dir.path())
        .arg("-i")
        .arg(file.path())
        .assert()
        .stdout("")
        .code(0)
        .success();

    // File should be formatted despite being in .rubyfmtignore
    assert_eq!("a(1, 2, 3)\n", read_to_string(file.path()).unwrap());
}
