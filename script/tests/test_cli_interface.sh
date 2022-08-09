#!/bin/bash
set -euxo pipefail

source "./script/functions.sh"

test_simple_stdout() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,3" > input.rb
    echo "a(1, 2, 3)" > expected.rb

    f_rubyfmt -- input.rb > out.rb

    diff_files o out.rb expected.rb
    )
}

test_stdin_stdout() {
    (
    cd "$(mktemp -d)"

    echo "a(1, 2, 3)" > expected.rb

    echo "a 1,2,3" | f_rubyfmt > out.rb

    diff_files o out.rb expected.rb
    )
}

test_stdin_stdout_respects_opt_in_header() {
    (
    cd "$(mktemp -d)"

    cat > input.rb <<- DIFF
a 1,2,3
DIFF

    cat > expected.rb <<- DIFF
a 1,2,3
DIFF

    f_rubyfmt --header-opt-in < input.rb > out.rb
    diff_files o out.rb expected.rb

    cat > input.rb <<- DIFF
# rubyfmt: true
a 1,2,3
DIFF

    cat > expected.rb <<- DIFF
# rubyfmt: true
a(1, 2, 3)
DIFF

    f_rubyfmt --header-opt-in < input.rb > out.rb
    diff_files o out.rb expected.rb
    )
}

test_stdin_stdout_respects_opt_out_header() {
    (
    cd "$(mktemp -d)"

    cat > input.rb <<- DIFF
a 1,2,3
DIFF

    cat > expected.rb <<- DIFF
a(1, 2, 3)
DIFF

    f_rubyfmt --header-opt-out < input.rb > out.rb
    diff_files o out.rb expected.rb

    cat > input.rb <<- DIFF
# rubyfmt: false
a 1,2,3
DIFF

    cat > expected.rb <<- DIFF
# rubyfmt: false
a 1,2,3
DIFF

    f_rubyfmt --header-opt-out < input.rb > out.rb

    diff_files o out.rb expected.rb
    )
}


test_dir_no_i_flag() {
    (
    cd "$(mktemp -d)"

    mkdir bees/
    echo "a 1,2,3" > bees/a_ruby_file_1.rb
    echo "a 1,2,5" > bees/a_ruby_file_2.rb
    echo "a(1, 2, 3)" > expected_1.rb
    echo "a(1, 2, 5)" > expected_2.rb

    f_rubyfmt bees/

    diff_files o bees/a_ruby_file_1.rb expected_1.rb
    diff_files o bees/a_ruby_file_2.rb expected_2.rb
    )
}

test_i_flag() {
    (
    cd "$(mktemp -d)"

    mkdir bees/
    mkdir bees/sub
    echo "a 1,2,3" > bees/a_ruby_file_1.rb
    echo "a 1,2,5" > bees/a_ruby_file_2.rb
    echo "a 1,2,6" > bees/sub/a_ruby_file_3.rb
    echo "a 1,2,7" > cows.rb

    echo "a(1, 2, 3)" > expected_1.rb
    echo "a(1, 2, 5)" > expected_2.rb
    echo "a(1, 2, 6)" > expected_3.rb
    echo "a(1, 2, 7)" > expected_4.rb

    f_rubyfmt -i bees/ cows.rb

    diff_files o bees/a_ruby_file_1.rb expected_1.rb
    diff_files o bees/a_ruby_file_2.rb expected_2.rb
    diff_files o bees/sub/a_ruby_file_3.rb expected_3.rb
    diff_files o cows.rb expected_4.rb
    )
}

test_check_flag_with_changes() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,3" > a_ruby_file_1.rb

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check -- a_ruby_file_1.rb > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
--- a_ruby_file_1.rb
+++ a_ruby_file_1.rb
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_multiple_files_with_changes() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,3" > a_ruby_file_1.rb
    echo "a 4,5,6,7" > a_ruby_file_2.rb

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check -- a_ruby_file_1.rb a_ruby_file_2.rb > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
--- a_ruby_file_1.rb
+++ a_ruby_file_1.rb
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
--- a_ruby_file_2.rb
+++ a_ruby_file_2.rb
@@ -1 +1 @@
-a 4,5,6,7
+a(4, 5, 6, 7)
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_directory_with_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner

    echo "a 1,2,3" > inner/a_ruby_file_1.rb

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check -- inner/ > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
--- inner/a_ruby_file_1.rb
+++ inner/a_ruby_file_1.rb
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_input_file_with_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner

    echo "a 1,2,3" > inner/a_ruby_file_1.rb

    echo "inner/a_ruby_file_1.rb" > input_file.txt

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check -- @input_file.txt > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
--- inner/a_ruby_file_1.rb
+++ inner/a_ruby_file_1.rb
@@ -1 +1 @@
-a 1,2,3
+a(1, 2, 3)
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_respects_opt_in_header() {
    (
    cd "$(mktemp -d)"

    cat > a_ruby_file_1.rb <<- DIFF
# rubyfmt: true
a 1,2,3
DIFF

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check --header-opt-in -- a_ruby_file_1.rb > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
--- a_ruby_file_1.rb
+++ a_ruby_file_1.rb
@@ -1,2 +1,2 @@
 # rubyfmt: true
-a 1,2,3
+a(1, 2, 3)
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_respects_opt_out_header() {
    (
    cd "$(mktemp -d)"

    cat > a_ruby_file_1.rb <<- DIFF
# rubyfmt: false
a 1,2,3
DIFF

    # --check returns non-zero when there are no changes
    set +e
    f_rubyfmt --check --header-opt-out -- a_ruby_file_1.rb > fmt.diff
    set -e

    cat > expected.diff <<- DIFF
DIFF

    cat expected.diff
    cat fmt.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_without_changes() {
    (
    cd "$(mktemp -d)"

    echo "a(1, 2, 3)" > a_ruby_file_1.rb

    f_rubyfmt --check -- a_ruby_file_1.rb > fmt.diff

    # printf instead of echo so we don't get a newline
    printf "" > expected.diff

    diff_files o expected.diff fmt.diff
    )
}


test_check_flag_multiple_files_without_changes() {
    (
    cd "$(mktemp -d)"

    echo "a(1, 2, 3)" > a_ruby_file_1.rb
    echo "a(4, 5, 6, 7)" > a_ruby_file_2.rb

    # --check returns non-zero when there are no changes
    f_rubyfmt --check -- a_ruby_file_1.rb a_ruby_file_2.rb > fmt.diff

    # printf instead of echo so we don't get a newline
    printf "" > expected.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_directory_without_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner

    echo "a(1, 2, 3)" > inner/a_ruby_file_1.rb

    # --check returns non-zero when there are no changes
    f_rubyfmt --check -- inner/ > fmt.diff

    # printf instead of echo so we don't get a newline
    printf "" > expected.diff

    diff_files o expected.diff fmt.diff
    )
}

test_check_flag_input_file_without_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner

    echo "a(1, 2, 3)" > inner/a_ruby_file_1.rb

    echo "inner/a_ruby_file_1.rb" > input_file.txt

    # --check returns non-zero when there are no changes
    f_rubyfmt --check -- @input_file.txt > fmt.diff

    # printf instead of echo so we don't get a newline
    printf "" > expected.diff

    diff_files o expected.diff fmt.diff
    )
}

test_format_with_changes() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > a_ruby_file_1.rb

    f_rubyfmt -i -- a_ruby_file_1.rb
    cat > expected.rb <<- DIFF
a(1, 2, 3)
DIFF

    diff_files o expected.rb a_ruby_file_1.rb
    )
}

test_format_multiple_files_with_changes() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > a_ruby_file_1.rb
    echo "a 4, 5, 6" > a_ruby_file_2.rb

    f_rubyfmt -i -- a_ruby_file_1.rb a_ruby_file_2.rb
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > a_ruby_file_2_expected.rb <<- DIFF
a(4, 5, 6)
DIFF

    diff_files o a_ruby_file_1_expected.rb a_ruby_file_1.rb
    diff_files o a_ruby_file_2_expected.rb a_ruby_file_2.rb
    )
}

test_format_directory_with_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner/
    echo "a 1, 2, 3" > inner/a_ruby_file_1.rb
    echo "a 4, 5, 6" > inner/a_ruby_file_2.rb

    f_rubyfmt -i -- inner/
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > a_ruby_file_2_expected.rb <<- DIFF
a(4, 5, 6)
DIFF

    diff_files o a_ruby_file_1_expected.rb inner/a_ruby_file_1.rb
    diff_files o a_ruby_file_2_expected.rb inner/a_ruby_file_2.rb
    )
}

test_format_input_file_with_changes() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > a_ruby_file_1.rb
    echo "a_ruby_file_1.rb" > input.txt

    f_rubyfmt -i -- @input.txt
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF

    diff_files o a_ruby_file_1_expected.rb a_ruby_file_1.rb
    )
}


test_format_respects_opt_in_header() {
    (
    cd "$(mktemp -d)"

    cat > a_ruby_file_1.rb <<- DIFF
# rubyfmt: true
a 1,2,3
DIFF

    set +e
    f_rubyfmt -i --header-opt-in -- a_ruby_file_1.rb
    set -e

    cat > expected.rb <<- DIFF
# rubyfmt: true
a(1, 2, 3)
DIFF

    diff_files o expected.rb a_ruby_file_1.rb
    )
}

test_format_respects_opt_out_header() {
    (
    cd "$(mktemp -d)"

    cat > a_ruby_file_1.rb <<- DIFF
# rubyfmt: false
a 1,2,3
DIFF

    set +e
    f_rubyfmt -i --header-opt-out -- a_ruby_file_1.rb
    set -e

    cat > expected.rb <<- DIFF
# rubyfmt: false
a 1,2,3
DIFF

    diff_files o expected.rb a_ruby_file_1.rb
    )
}

test_format_without_changes() {
    (
    cd "$(mktemp -d)"
    echo "a(1, 2, 3)" > a_ruby_file_1.rb

    f_rubyfmt -i -- a_ruby_file_1.rb
    cat > expected.rb <<- DIFF
a(1, 2, 3)
DIFF

    diff_files o expected.rb a_ruby_file_1.rb
    )
}

test_format_multiple_files_without_changes() {
    (
    cd "$(mktemp -d)"
    echo "a(1, 2, 3)" > a_ruby_file_1.rb
    echo "a(4, 5, 6)" > a_ruby_file_2.rb

    f_rubyfmt -i -- a_ruby_file_1.rb a_ruby_file_2.rb
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > a_ruby_file_2_expected.rb <<- DIFF
a(4, 5, 6)
DIFF

    diff_files o a_ruby_file_1_expected.rb a_ruby_file_1.rb
    diff_files o a_ruby_file_2_expected.rb a_ruby_file_2.rb
    )
}

test_format_directory_without_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner/
    echo "a(1, 2, 3)" > inner/a_ruby_file_1.rb
    echo "a(4, 5, 6)" > inner/a_ruby_file_2.rb

    f_rubyfmt -i -- inner/
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > a_ruby_file_2_expected.rb <<- DIFF
a(4, 5, 6)
DIFF

    diff_files o a_ruby_file_1_expected.rb inner/a_ruby_file_1.rb
    diff_files o a_ruby_file_2_expected.rb inner/a_ruby_file_2.rb
    )
}

test_format_input_file_without_changes() {
    (
    cd "$(mktemp -d)"
    mkdir inner/
    echo "a(1, 2, 3)" > a_ruby_file_1.rb
    echo "a_ruby_file_1.rb" > input_file.txt

    f_rubyfmt -i -- input_file.txt
    cat > a_ruby_file_1_expected.rb <<- DIFF
a(1, 2, 3)
DIFF

    diff_files o a_ruby_file_1_expected.rb a_ruby_file_1.rb
    )
}

test_respects_gitignore() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > in_scope.rb
    echo "a 4, 5, 6" > out_of_scope.rb

    # Make a fake git repo
    mkdir .git/
    echo "out_of_scope.rb" > .gitignore

    f_rubyfmt -i -- .

    cat > in_scope_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > out_of_scope_expected.rb <<- DIFF
a 4, 5, 6
DIFF

    diff_files o in_scope_expected.rb in_scope.rb
    diff_files o out_of_scope_expected.rb out_of_scope.rb
    )
}

test_includes_gitignore() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > in_scope.rb
    echo "a 4, 5, 6" > out_of_scope.rb

    # Make a fake git repo
    mkdir .git/
    echo "out_of_scope.rb" > .gitignore

    f_rubyfmt -i --include-gitignored -- .

    cat > in_scope_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > out_of_scope_expected.rb <<- DIFF
a(4, 5, 6)
DIFF

    diff_files o in_scope_expected.rb in_scope.rb
    diff_files o out_of_scope_expected.rb out_of_scope.rb
    )
}

test_respects_rubyfmt_ignore_file() {
    (
    cd "$(mktemp -d)"
    echo "a 1, 2, 3" > in_scope.rb
    echo "a 4, 5, 6" > out_of_scope.rb

    echo "out_of_scope.rb" > .rubyfmtignore

    f_rubyfmt -i -- .

    cat > in_scope_expected.rb <<- DIFF
a(1, 2, 3)
DIFF
    cat > out_of_scope_expected.rb <<- DIFF
a 4, 5, 6
DIFF

    diff_files o in_scope_expected.rb in_scope.rb
    diff_files o out_of_scope_expected.rb out_of_scope.rb
    )
}

test_simple_stdout

test_stdin_stdout
test_stdin_stdout_respects_opt_in_header
test_stdin_stdout_respects_opt_out_header

test_check_flag_with_changes
test_check_flag_multiple_files_with_changes
test_check_flag_directory_with_changes
test_check_flag_input_file_with_changes
test_check_flag_respects_opt_in_header
test_check_flag_respects_opt_out_header

test_check_flag_without_changes
test_check_flag_multiple_files_without_changes
test_check_flag_directory_without_changes
test_check_flag_input_file_without_changes

test_format_with_changes
test_format_multiple_files_with_changes
test_format_directory_with_changes
test_format_input_file_with_changes
test_format_respects_opt_in_header
test_format_respects_opt_out_header

test_format_without_changes
test_format_multiple_files_without_changes
test_format_directory_without_changes
test_format_input_file_without_changes

test_respects_gitignore
test_includes_gitignore

test_respects_rubyfmt_ignore_file
