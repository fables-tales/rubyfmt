#![deny(warnings, missing_copy_implementations)]

use clap::Parser;
use ignore::WalkBuilder;
use regex::Regex;
use similar::TextDiff;
use std::ffi::OsStr;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MAGIC_COMMENT_REGEX: Regex =
        Regex::new(r"(?m)^#\s*rubyfmt:\s*(?P<enabled>true|false)\s*$").unwrap();
}

/// Simple Enum to exit on errors or not
#[derive(Debug, PartialEq, Copy, Clone)]
enum ErrorExit {
    NoExit,
    Exit,
}

/// Error enum representing errors in the cli.
#[derive(Debug)]
enum ExecutionError {
    // Errors seen when rubyfmt is executing
    RubyfmtError(rubyfmt::RichFormatError, String),
    // Errors seen when performing IO s
    IOError(io::Error, String),
    // Errors seen when grepping for files
    FileSearchFailure(ignore::Error),
}

/// Rubyfmt CLI
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CommandlineOpts {
    /// Turn on check mode. This outputs diffs of inputs to STDOUT. Will exit non-zero when differences are detected.
    #[clap(short, long)]
    check: bool,

    /// Turn on to ignore gitignored files. Gitignored files are not considered by rubyfmt by default.
    #[clap(long, name = "include-gitignored")]
    include_gitignored: bool,

    /// Only format ruby files containing the magic `# rubyfmt: true` header
    #[clap(long, name = "header-opt-in")]
    header_opt_in: bool,

    /// Do not format ruby files containing the magic `# rubyfmt: false` header
    #[clap(long, name = "header-opt-out")]
    header_opt_out: bool,

    /// Fail on all syntax and io errors early. Warnings otherwise.
    #[clap(long, name = "fail-fast")]
    fail_fast: bool,

    /// Write files back in place, do not write output to STDOUT.
    #[clap(short, long, name = "in-place")]
    in_place: bool,

    /// Paths for rubyfmt to analyze. By default the output will be printed to STDOUT. See `--in-place` to write files back in-place.
    /// Acceptable paths are:{n}
    /// - File paths (i.e lib/foo/bar.rb){n}
    /// - Directories (i.e. lib/foo/){n}
    /// - Input files (i.e. @/tmp/files.txt). These files must contain one file path or directory per line
    /// rubyfmt will use these as input.{n}
    #[clap(name = "include-paths")]
    include_paths: Vec<String>,
}

/******************************************************/
/* Error handling                                     */
/******************************************************/

fn handle_io_error(err: io::Error, source: &String, error_exit: ErrorExit) {
    let msg = format!("Rubyfmt experienced an IO error: {}", err);
    print_error(&msg, Some(source));

    if error_exit == ErrorExit::Exit {
        exit(rubyfmt::FormatError::IOError as i32);
    }
}

fn handle_ignore_error(err: ignore::Error, error_exit: ErrorExit) {
    let msg = format!("Rubyfmt experienced an error searching for files: {}", err);
    print_error(&msg, None);
    if error_exit == ErrorExit::Exit {
        exit(rubyfmt::FormatError::IOError as i32);
    }
}

fn handle_rubyfmt_error(err: rubyfmt::RichFormatError, source: &String, error_exit: ErrorExit) {
    use rubyfmt::RichFormatError::*;
    let exit_code = err.as_exit_code();
    let e = || {
        if error_exit == ErrorExit::Exit {
            exit(exit_code);
        }
    };
    match err {
        SyntaxError => {
            let msg = "Rubyfmt detected a syntax error in the ruby code being executed";
            print_error(msg, Some(source));
            e();
        }
        rubyfmt::RichFormatError::RipperParseFailure(_) => {
            let bug_report = "
!!! Ruby Tree Deserialization Error !!!

Rubyfmt failed to correctly deserialize a tree from ripper. This is a bug that needs to be reported.
File a bug report at https://github.com/penelopezone/rubyfmt/issues/new.
Ideally you would include the full source code of the program you ran rubyfmt with.
If you can't do that for some reason, the best thing you can do is rerun rubyfmt on this program 
with the debug binary with `2>log_file` on the end and then send us the log file that gets generated.
";
            print_error(bug_report, Some(source));
            e();
        }
        IOError(ioe) => {
            let msg = format!("Rubyfmt experienced an IO error: {}", ioe);
            print_error(&msg, Some(source));
            e();
        }
        rubyfmt::RichFormatError::OtherRubyError(s) => {
            let msg = format!("Rubyfmt experienced an unexpected ruby error: {}", s);
            print_error(&msg, Some(source));
            exit(exit_code);
        }
    }
}

fn print_error(msg: &str, file_path: Option<&str>) {
    let mut first_line: String = "Error!".to_string();

    if let Some(line) = file_path {
        first_line = format!("Error! source: {}", line);
    }

    eprintln!("{}\n{}", first_line, msg);
}

fn handle_execution_error(opts: &CommandlineOpts, err: ExecutionError) {
    let mut exit_type = ErrorExit::NoExit;
    // If include_paths are empty, this is operating on STDIN which should always exit
    if opts.fail_fast || opts.include_paths.is_empty() {
        exit_type = ErrorExit::Exit;
    }

    match err {
        ExecutionError::RubyfmtError(e, path) => handle_rubyfmt_error(e, &path, exit_type),
        ExecutionError::IOError(e, path) => handle_io_error(e, &path, exit_type),
        ExecutionError::FileSearchFailure(e) => handle_ignore_error(e, exit_type),
    }
}

/******************************************************/
/* Rubyfmt Integration                                */
/******************************************************/

fn rubyfmt_string(
    &CommandlineOpts {
        header_opt_in,
        header_opt_out,
        ..
    }: &CommandlineOpts,
    buffer: &str,
) -> Result<Option<String>, rubyfmt::RichFormatError> {
    if header_opt_in || header_opt_out {
        // Only look at the first 500 bytes for the magic header.
        // This is for performance
        let mut slice = buffer;
        let mut slice_size = 500;
        let blength = buffer.len();

        if blength > slice_size {
            while !buffer.is_char_boundary(slice_size) && slice_size < blength {
                slice_size += 1;
            }
            slice = &buffer[..slice_size]
        }

        let matched = MAGIC_COMMENT_REGEX
            .captures(slice)
            .and_then(|c| c.name("enabled"))
            .map(|s| s.as_str());

        // If opted in to magic "# rubyfmt: true" header and true is not
        // in the file, return early
        if header_opt_in && Some("true") != matched {
            return Ok(None);
        }

        // If opted in to magic "# rubyfmt: false" header and false is
        // in the file, return early
        if header_opt_out && Some("false") == matched {
            return Ok(None);
        }
    }

    rubyfmt::format_buffer(buffer).map(Some)
}

fn initialize_rubyfmt() {
    let res = rubyfmt::rubyfmt_init();
    if res != rubyfmt::InitStatus::OK as libc::c_int {
        panic!(
            "bad init status: {}",
            rubyfmt::ruby::current_exception_as_rust_string()
        );
    }
}

/******************************************************/
/* Helpers                                            */
/******************************************************/

fn file_walker_builder(
    CommandlineOpts {
        include_paths,
        include_gitignored,
        ..
    }: &CommandlineOpts,
) -> WalkBuilder {
    // WalkBuilder does not have an API for adding multiple inputs.
    // Must pass the first input to the constructor, and the tail afterwards.
    // Safe to unwrap here.
    let (include_head, include_tail) = include_paths.split_first().unwrap();
    let mut builder = WalkBuilder::new(include_head);

    for path in include_tail {
        builder.add(path);
    }

    builder.git_ignore(!*include_gitignored);
    builder.add_custom_ignore_filename(".rubyfmtignore");
    builder
}

// Parse command line arguments. Expand any input files.
fn get_command_line_options() -> CommandlineOpts {
    let opts = CommandlineOpts::parse();

    let mut expanded_paths: Vec<String> = Vec::new();

    for path in opts.include_paths {
        // Expand input files
        if let Some(file_name) = path.strip_prefix('@') {
            match File::open(file_name) {
                Ok(file) => {
                    let buf = BufReader::new(file);
                    let lines: Vec<String> = buf
                        .lines()
                        .map(|l| l.expect("Could not parse line"))
                        .collect();
                    for line in lines {
                        expanded_paths.push(line);
                    }
                }
                Err(e) => handle_io_error(e, &path, ErrorExit::Exit),
            }
        } else {
            expanded_paths.push(path);
        }
    }

    CommandlineOpts {
        include_paths: expanded_paths,
        ..opts
    }
}

fn iterate_input_files(opts: &CommandlineOpts, f: &dyn Fn((&Path, &String))) {
    if opts.include_paths.is_empty() {
        // If not include paths are present, assume user is passing via STDIN
        let mut buffer = String::new();

        if atty::is(atty::Stream::Stdin) {
            // Call executable with `--help` args to print help statement
            let mut command = Command::new(std::env::current_exe().unwrap());
            command.arg("--help");
            command.spawn().unwrap().wait().unwrap();
            return;
        }

        io::stdin()
            .read_to_string(&mut buffer)
            .expect("reading from stdin to not fail");
        f((Path::new("stdin"), &buffer))
    } else {
        for result in file_walker_builder(opts).build() {
            match result {
                Ok(pp) => {
                    let file_path = pp.path();

                    if file_path.is_file()
                        && file_path.extension().and_then(OsStr::to_str) == Some("rb")
                    {
                        let buffer_res = read_to_string(file_path);

                        match buffer_res {
                            Ok(buffer) => f((file_path, &buffer)),
                            Err(e) => handle_execution_error(
                                opts,
                                ExecutionError::IOError(e, file_path.display().to_string()),
                            ),
                        }
                    }
                }
                Err(e) => handle_execution_error(opts, ExecutionError::FileSearchFailure(e)),
            }
        }
    }
}

type FormattingFunc<'a> = &'a dyn Fn((&Path, &String, Option<String>));

fn iterate_formatted(opts: &CommandlineOpts, f: FormattingFunc) {
    iterate_input_files(
        opts,
        &|(file_path, before)| match rubyfmt_string(opts, before) {
            Ok(r) => f((file_path, before, r)),
            Err(e) => handle_execution_error(
                opts,
                ExecutionError::RubyfmtError(e, file_path.display().to_string()),
            ),
        },
    );
}

fn puts_stdout(input: &String) {
    write!(io::stdout(), "{}", input).expect("Could not write to stdout");
    io::stdout().flush().expect("flush works");
}

fn main() {
    ctrlc::set_handler(move || {
        eprintln!("`rubyfmt` process was terminated. Exiting...");
        exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    let opts = get_command_line_options();

    match opts {
        CommandlineOpts { check: true, .. } => {
            initialize_rubyfmt();
            let text_diffs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

            iterate_formatted(&opts, &|(file_path, before, after)| match after {
                None => {}
                Some(fmtted) => {
                    let diff = TextDiff::from_lines(before, &fmtted);
                    let path_string = file_path.to_str().unwrap();
                    text_diffs.lock().unwrap().push(format!(
                        "{}",
                        diff.unified_diff().header(path_string, path_string)
                    ));
                }
            });

            let all_diffs = text_diffs.lock().unwrap();

            let mut diffs_reported = 0;

            for diff in all_diffs.iter() {
                if !diff.is_empty() {
                    puts_stdout(diff);
                    diffs_reported += 1
                }
            }
            if diffs_reported > 0 {
                exit(rubyfmt::FormatError::DiffDetected as i32);
            } else {
                exit(0)
            }
        }

        CommandlineOpts { in_place: true, .. } => {
            initialize_rubyfmt();
            iterate_formatted(&opts, &|(file_path, before, after)| match after {
                None => {}
                Some(fmtted) => {
                    if fmtted.ne(before) {
                        let file_write = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(file_path)
                            .and_then(|mut file| write!(file, "{}", fmtted));

                        match file_write {
                            Ok(_) => {}
                            Err(e) => handle_execution_error(
                                &opts,
                                ExecutionError::IOError(e, file_path.display().to_string()),
                            ),
                        }
                    }
                }
            })
        }

        _ => {
            initialize_rubyfmt();
            iterate_formatted(&opts, &|(_, before, after)| match after {
                Some(fmtted) => puts_stdout(&fmtted),
                None => puts_stdout(before),
            })
        }
    }
}
