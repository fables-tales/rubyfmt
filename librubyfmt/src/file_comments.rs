use std::borrow::Cow;
use std::io::BufRead;

use memchr::memchr_iter;

use crate::comment_block::CommentBlock;
use crate::parser_state::line_difference_requires_newline;
use crate::types::{LineNumber, SourceOffset};

#[derive(Clone, Debug, Default)]
pub struct FileComments {
    start_of_file_contiguous_comment_lines: Option<CommentBlock>,
    /// A list of comments, sorted in order by `LineNumber`
    other_comments: Vec<(LineNumber, Vec<u8>)>,
    /// Sorted list of line numbers that contain Ruby code (not comments/blank)
    lines_with_ruby: Vec<LineNumber>,
    last_lineno: LineNumber,
    /// Sorted list of byte offsets where comments start
    comment_start_offsets: Vec<SourceOffset>,
}

impl FileComments {
    pub fn from_prism_comments(
        comments: ruby_prism::Comments,
        source: &[u8],
        end_line: i32,
    ) -> FileComments {
        let mut lines_with_ruby = Vec::new();

        let mut line_start = 0;
        let mut lineno = 1;
        let mut inside_embdoc = false;

        for i in memchr_iter(b'\n', source) {
            if Self::line_has_ruby(&source[line_start..i], &mut inside_embdoc) {
                lines_with_ruby.push(lineno);
            }

            line_start = i + 1;
            lineno += 1;
        }

        // Handle last line if no trailing newline
        if line_start < source.len() {
            let line = &source[line_start..];
            if Self::line_has_ruby(line, &mut inside_embdoc) {
                lines_with_ruby.push(lineno);
            }
        }

        let mut file_comments = FileComments::default();
        for comment in comments {
            file_comments.push_comment(
                comment.location().start_line(),
                comment.text().trim_ascii_end().to_vec(),
            );
            file_comments
                .comment_start_offsets
                .push(comment.location().start());
        }

        file_comments.lines_with_ruby = lines_with_ruby;
        file_comments.last_lineno = end_line;
        file_comments
    }

    fn line_has_ruby(line: &[u8], inside_embdoc: &mut bool) -> bool {
        let first_non_ws = line.iter().position(|b| !u8::is_ascii_whitespace(b));
        let Some(idx) = first_non_ws else {
            return false;
        };

        let trimmed = &line[idx..];

        if trimmed.starts_with(b"=begin") {
            *inside_embdoc = true;
            return false;
        }
        if trimmed.starts_with(b"=end") {
            *inside_embdoc = false;
            return false;
        }
        if *inside_embdoc {
            return false;
        }

        // Check if it's a comment
        trimmed[0] != b'#'
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
    fn push_comment(&mut self, line_number: LineNumber, l: Vec<u8>) {
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
                    Some(CommentBlock::new(1..2, vec![l.into()]));
            }
            (Some(sled), _) if sled.following_line_number() == line_number => {
                sled.add_line(l.into());
            }
            _ => {
                debug_assert!(
                    self.other_comments
                        .last()
                        .is_none_or(|(last_line_number, _)| *last_line_number < line_number),
                    "Expected comments to be inserted in order"
                );

                self.other_comments.push((line_number, l));
            }
        }
    }

    pub fn is_empty_line(&self, line_number: LineNumber) -> bool {
        self.lines_with_ruby.binary_search(&line_number).is_err()
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
        let idx = self
            .comment_start_offsets
            .partition_point(|&offset| offset < start_offset);

        self.comment_start_offsets
            .get(idx)
            .is_some_and(|&offset| offset < end_offset)
    }

    pub fn extract_comments_to_line(
        &mut self,
        starting_line_number: LineNumber,
        line_number: LineNumber,
    ) -> Option<(CommentBlock, LineNumber)> {
        let lowest_line = self.other_comments.first().map(|(ln, _)| *ln)?;
        if lowest_line > line_number {
            return None;
        }

        let split_point = self
            .other_comments
            .partition_point(|(ln, _)| *ln <= line_number);

        let mut comment_block_with_spaces = Vec::new();
        let mut last_line = None;

        if line_difference_requires_newline(
            self.other_comments.first().unwrap().0,
            starting_line_number,
        ) {
            comment_block_with_spaces.push(b"".into());
        }

        for (index, comment_contents) in self.other_comments.drain(..split_point) {
            let comment_contents: Cow<'_, [u8]> = Cow::Owned(comment_contents);
            if let Some(last_line) = last_line
                && line_difference_requires_newline(index, last_line)
            {
                comment_block_with_spaces.push(b"".into());
            }
            let line_count = comment_contents.lines().count() as i32;
            last_line = Some(index + line_count - 1);
            comment_block_with_spaces.push(comment_contents);
        }

        if line_number > last_line.unwrap() + 1 {
            last_line = Some(line_number);
            comment_block_with_spaces.push(b"".into());
        }

        Some((
            CommentBlock::new(lowest_line..line_number + 1, comment_block_with_spaces),
            last_line.unwrap(),
        ))
    }
}
