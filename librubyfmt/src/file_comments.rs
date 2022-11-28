use std::mem;

use log::debug;

use crate::comment_block::CommentBlock;
use crate::parser_state::line_difference_requires_newline;
use crate::types::LineNumber;

#[derive(Clone, Debug, Default)]
pub struct FileComments {
    start_of_file_contiguous_comment_lines: Option<CommentBlock>,
    raw_comments: ruby_ops::RubyComments,
}

impl FileComments {
    pub fn new(raw_comments: ruby_ops::RubyComments) -> Self {
        FileComments {
            raw_comments,
            start_of_file_contiguous_comment_lines: None,
        }
    }
    pub fn still_in_file(&self, line_number: LineNumber) -> bool {
        line_number < self.raw_comments.last_lineno
    }

    pub fn has_line(&self, line_number: LineNumber) -> bool {
        self.raw_comments.other_comments.contains_key(&line_number)
    }

    /// Add a new comment. If the beginning of this file is a comment block,
    /// each of those comment lines must be pushed before any other line, or
    /// the end of the block from the start of the file will be incorrectly calculated

    pub fn is_empty_line(&self, line_number: LineNumber) -> bool {
        debug!("{:?}", self.raw_comments.lines_with_ruby);
        !self.raw_comments.lines_with_ruby.contains(&line_number)
    }

    pub fn take_start_of_file_contiguous_comment_lines(&mut self) -> Option<CommentBlock> {
        self.start_of_file_contiguous_comment_lines.take()
    }

    pub fn has_comments_in_lines(&self, start_line: LineNumber, end_line: LineNumber) -> bool {
        let line_range = start_line..end_line;
        self.raw_comments
            .other_comments
            .keys()
            .any(|key| line_range.contains(key))
    }

    pub fn extract_comments_to_line(
        &mut self,
        starting_line_number: LineNumber,
        line_number: LineNumber,
    ) -> Option<(CommentBlock, LineNumber)> {
        self.raw_comments
            .other_comments
            .keys()
            .next()
            .copied()
            .map(|lowest_line| {
                let remaining_comments = self
                    .raw_comments
                    .other_comments
                    .split_off(&(&line_number + 1));
                let comments =
                    mem::replace(&mut self.raw_comments.other_comments, remaining_comments)
                        .into_iter()
                        .collect::<Vec<(_, _)>>();
                if comments.is_empty() {
                    return (
                        CommentBlock::new(lowest_line..line_number + 1, Vec::new()),
                        starting_line_number,
                    );
                }

                let mut comment_block_with_spaces: Vec<String> = Vec::new();
                let mut last_line = None;

                if line_difference_requires_newline(
                    comments.first().unwrap().0,
                    starting_line_number,
                ) {
                    comment_block_with_spaces.push(String::new());
                }

                for (index, comment_contents) in comments {
                    if last_line.is_some()
                        && line_difference_requires_newline(index, last_line.unwrap())
                    {
                        comment_block_with_spaces.push(String::new());
                    }
                    last_line = Some(index);
                    comment_block_with_spaces.push(comment_contents);
                }

                if line_number > last_line.unwrap() + 1 {
                    last_line = Some(line_number);
                    comment_block_with_spaces.push(String::new());
                }

                (
                    CommentBlock::new(lowest_line..line_number + 1, comment_block_with_spaces),
                    last_line.unwrap(),
                )
            })
    }
}
