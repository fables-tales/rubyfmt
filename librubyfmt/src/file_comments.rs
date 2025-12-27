use std::collections::{BTreeSet, HashSet};

use crate::comment_block::CommentBlock;
use crate::parser_state::line_difference_requires_newline;
use crate::ruby::*;
use crate::types::{LineNumber, SourceOffset};
use crate::util::{u8_to_str, u8_to_string};

/// A vector of offsets in the source code where lines start, which
/// we use to detect what line a given offset is one.
///
/// Note: The Prism Ruby API does track lines, but it's currently not exposed
/// by the Rust crate, so this is our alternative while we're building Prism support
/// and can always clean it up later.
#[derive(Debug, Clone, Default)]
pub struct LineIndex {
    // Stores the byte offset of each line's start position
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(file_contents: &[u8]) -> Self {
        let mut line_starts = Vec::new();

        // First line always starts at position 0
        line_starts.push(0);

        for (i, &byte) in file_contents.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }

        LineIndex { line_starts }
    }

    // Get the line number (1-based) for a given byte offset
    pub fn get_line_number(&self, offset: usize) -> usize {
        let line = match self.line_starts.binary_search(&offset) {
            // Exact match means we're at the start of a line
            Ok(line) => line,

            // No exact match, so the insertion_point is the index of the next line.
            // We want the current line, which is insertion_point - 1
            Err(insertion_point) => {
                // Should only happen if we're checking an offset
                // in the middle of the first line
                if insertion_point == 0 {
                    0
                } else {
                    insertion_point - 1
                }
            }
        };
        // These are one-offset to be equivalent to Ripper's behavior.
        // Once we remove the Ripper version, we can 0-index these,
        // since they're not user-facing anywhere
        line + 1
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileComments {
    start_of_file_contiguous_comment_lines: Option<CommentBlock>,
    /// A list of comments, sorted in order by `LineNumber`
    other_comments: Vec<(LineNumber, String)>,
    lines_with_ruby: BTreeSet<LineNumber>,
    last_lineno: LineNumber,
    line_index: LineIndex,
    comment_start_offsets: HashSet<usize>,
}

impl FileComments {
    pub fn from_prism_comments(comments: ruby_prism::Comments, source: &[u8]) -> FileComments {
        let line_index = LineIndex::new(source);
        let mut file_comments = FileComments::default();
        for comment in comments {
            file_comments.push_comment(
                line_index.get_line_number(comment.location().start_offset()) as u64,
                u8_to_string(comment.text().trim_ascii_end()),
            );
            file_comments
                .comment_start_offsets
                .insert(comment.location().start_offset());
        }

        // Lookup lines that have any Ruby
        let mut inside_embdoc = false;
        u8_to_str(source)
            .lines()
            .enumerate()
            .filter(|(_lineno, line_contents)| {
                let contents = line_contents.trim();
                if contents.starts_with("=begin") {
                    inside_embdoc = true;
                    return false;
                }
                if contents.starts_with("=end") {
                    inside_embdoc = false;
                    return false;
                }
                if inside_embdoc {
                    return false;
                }
                !(contents.starts_with("#") || contents.is_empty())
            })
            .for_each(|(lineno, _)| {
                file_comments
                    .lines_with_ruby
                    // Insert as one-offset to work with Ripper.
                    // This (and elsewhere) can be zero-offset once Ripper is removed
                    .insert((lineno + 1) as u64);
            });

        file_comments.last_lineno = line_index.line_starts.len() as u64;
        file_comments.line_index = line_index;
        file_comments
    }

    pub fn from_ruby_hash(h: VALUE, rl: VALUE, last_lineno: VALUE) -> Self {
        let mut fc = FileComments::default();
        let keys;
        let values;
        let lines;
        unsafe {
            keys = ruby_array_to_slice(rb_funcall(h, intern!("keys"), 0));
            values = ruby_array_to_slice(rb_funcall(h, intern!("values"), 0));
            lines = ruby_array_to_slice(rb_funcall(rl, intern!("keys"), 0));
            fc.last_lineno = rubyfmt_rb_num2ll(last_lineno) as LineNumber;
        }
        if keys.len() != values.len() {
            raise("expected keys and values to have same length, indicates error");
        }
        for (ruby_lineno, ruby_comment) in keys.iter().zip(values) {
            let lineno = unsafe { rubyfmt_rb_num2ll(*ruby_lineno) };
            if lineno < 0 {
                raise("line number negative");
            }
            let comment = unsafe { ruby_string_to_str(*ruby_comment) }
                .trim()
                .to_owned();
            fc.push_comment(lineno as _, comment);
        }
        for ruby_lineno in lines.iter() {
            let lineno = unsafe { rubyfmt_rb_num2ll(*ruby_lineno) };
            if lineno < 0 {
                raise("line number negative");
            }
            fc.lines_with_ruby.insert(lineno as LineNumber);
        }
        fc
    }

    pub fn still_in_file(&self, line_number: LineNumber) -> bool {
        line_number < self.last_lineno
    }

    pub fn has_line(&self, line_number: LineNumber) -> bool {
        self.other_comments
            .binary_search_by_key(&line_number, |(ln, _)| *ln)
            .is_ok()
    }

    /// Add a new comment. If the beginning of this file is a comment block,
    /// each of those comment lines must be pushed before any other line, or
    /// the end of the block from the start of the file will be incorrectly calculated.
    fn push_comment(&mut self, line_number: u64, l: String) {
        match (
            &mut self.start_of_file_contiguous_comment_lines,
            line_number,
        ) {
            (None, 1) => {
                debug_assert!(
                    self.other_comments.is_empty(),
                    "If we have a start of file sled, it needs to come first,
                     otherwise we won't know where the last line is",
                );
                self.start_of_file_contiguous_comment_lines =
                    Some(CommentBlock::new(1..2, vec![l]));
            }
            (Some(sled), _) if sled.following_line_number() == line_number => {
                sled.add_line(l);
            }
            _ => {
                debug_assert!(
                    self.other_comments
                        .last()
                        .map(|(last_line_number, _)| *last_line_number < line_number)
                        .unwrap_or(true),
                    "Expected comments to be inserted in order"
                );

                self.other_comments.push((line_number, l));
            }
        }
    }

    pub fn is_empty_line(&self, line_number: LineNumber) -> bool {
        !self.lines_with_ruby.contains(&line_number)
    }

    pub fn take_start_of_file_contiguous_comment_lines(&mut self) -> Option<CommentBlock> {
        self.start_of_file_contiguous_comment_lines.take()
    }

    pub fn has_comments_in_lines(&self, start_line: LineNumber, end_line: LineNumber) -> bool {
        let line_range = start_line..end_line;
        self.other_comments
            .iter()
            .any(|(ln, _)| line_range.contains(ln))
    }

    pub fn has_comment_in_offsets(
        &self,
        start_offset: SourceOffset,
        end_offset: SourceOffset,
    ) -> bool {
        let range = start_offset..end_offset;

        self.comment_start_offsets
            .iter()
            .any(|offset| range.contains(offset))
    }

    pub fn extract_comments_to_line(
        &mut self,
        starting_line_number: LineNumber,
        line_number: LineNumber,
    ) -> Option<(CommentBlock, LineNumber)> {
        let lowest_line = self.other_comments.first().map(|(ln, _)| *ln)?;
        let split_point = self
            .other_comments
            .partition_point(|(ln, _)| *ln <= line_number);
        let comments: Vec<_> = self.other_comments.drain(..split_point).collect();

        if comments.is_empty() {
            return Some((
                CommentBlock::new(lowest_line..line_number + 1, Vec::new()),
                starting_line_number,
            ));
        }

        let mut comment_block_with_spaces: Vec<String> = Vec::new();
        let mut last_line = None;

        if line_difference_requires_newline(comments.first().unwrap().0, starting_line_number) {
            comment_block_with_spaces.push(String::new());
        }

        for (index, comment_contents) in comments {
            if let Some(last_line) = last_line
                && line_difference_requires_newline(index, last_line)
            {
                comment_block_with_spaces.push(String::new());
            }
            let line_count = comment_contents.lines().count() as u64;
            last_line = Some(index + line_count - 1);
            comment_block_with_spaces.push(comment_contents);
        }

        if line_number > last_line.unwrap() + 1 {
            last_line = Some(line_number);
            comment_block_with_spaces.push(String::new());
        }

        Some((
            CommentBlock::new(lowest_line..line_number + 1, comment_block_with_spaces),
            last_line.unwrap(),
        ))
    }

    // Note: this is currently only used for Prism support, see the details
    // on the `LineIndex` struct for context
    pub fn get_line_number_for_offset(&self, source_offset: usize) -> u64 {
        self.line_index.get_line_number(source_offset) as u64
    }
}
