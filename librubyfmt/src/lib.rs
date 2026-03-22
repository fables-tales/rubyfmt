#![deny(warnings, missing_copy_implementations)]

use std::io::{Cursor, Write};

#[cfg(all(feature = "use_jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub type RawStatus = i64;

mod comment_block;
mod delimiters;
mod file_comments;
mod format_prism;
mod heredoc_string;
mod intermediary;
mod line_metadata;
mod line_tokens;
mod parser_state;
mod render_queue_writer;
mod render_targets;
mod string_escape;
mod types;
mod util;

use file_comments::FileComments;
use parser_state::ParserState;

#[cfg(debug_assertions)]
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

#[derive(Debug)]
pub enum RichFormatError {
    SyntaxError,
    IOError(std::io::Error),
}

impl RichFormatError {
    pub fn as_exit_code(&self) -> i32 {
        self.as_format_error() as i32
    }

    fn as_format_error(&self) -> FormatError {
        match self {
            RichFormatError::SyntaxError => FormatError::SyntaxError,
            RichFormatError::IOError(_) => FormatError::IOError,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FormatError {
    OK = 0,
    SyntaxError = 1,
    RipperParseFailure = 2,
    IOError = 3,
    OtherRubyError = 4,
    // Diffs are only necessary in --check mode
    DiffDetected = 5,
}

pub fn format_buffer(buf: &[u8]) -> Result<Vec<u8>, RichFormatError> {
    let out_data = vec![];
    let mut output = Cursor::new(out_data);

    let parse_result = ruby_prism::parse(buf);
    if parse_result.errors().next().is_some() {
        return Err(RichFormatError::SyntaxError);
    }
    let end_data = parse_result.data_loc();

    toplevel_format_program_with_prism(
        &mut output,
        parse_result.node(),
        parse_result.comments(),
        buf,
        end_data,
    )?;

    output.flush().expect("flushing to a vec should never fail");
    Ok(output.into_inner())
}

pub fn toplevel_format_program_with_prism<W: Write>(
    writer: &mut W,
    tree: ruby_prism::Node,
    comments: ruby_prism::Comments,
    source: &[u8],
    data: Option<ruby_prism::Location>,
) -> Result<(), RichFormatError> {
    let mut ps = ParserState::new(FileComments::from_prism_comments(comments, source));
    ps.flush_start_of_file_comments();

    format_prism::format_program(&mut ps, tree.as_program_node().unwrap(), data);

    ps.write(writer).map_err(RichFormatError::IOError)?;
    writer.flush().map_err(RichFormatError::IOError)?;
    Ok(())
}

pub fn init_logger() {
    #[cfg(debug_assertions)]
    {
        TermLogger::init(
            LevelFilter::Debug,
            ConfigBuilder::new()
                .set_time_level(LevelFilter::Off)
                .build(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .expect("making a term logger");
    }
}
