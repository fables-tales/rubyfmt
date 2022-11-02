use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use log::debug;

use crate::comment_block::CommentBlock;
use crate::parser_state::line_difference_requires_newline;
use crate::ruby::*;
use crate::types::LineNumber;

#[derive(Clone, Debug, Default)]
pub struct FileComments {
    start_of_file_contiguous_comment_lines: Option<CommentBlock>,
    other_comments: BTreeMap<LineNumber, String>,
    lines_with_ruby: BTreeSet<LineNumber>,
    last_lineno: LineNumber,
}

impl FileComments {
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
        self.other_comments.contains_key(&line_number)
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
                self.other_comments.insert(line_number, l);
            }
        }
    }

    pub fn is_empty_line(&self, line_number: LineNumber) -> bool {
        debug!("{:?}", self.lines_with_ruby);
        !self.lines_with_ruby.contains(&line_number)
    }

    pub fn take_start_of_file_contiguous_comment_lines(&mut self) -> Option<CommentBlock> {
        self.start_of_file_contiguous_comment_lines.take()
    }

    pub fn has_comments_in_lines(&self, start_line: LineNumber, end_line: LineNumber) -> bool {
        let line_range = start_line..end_line;
        self.other_comments
            .keys()
            .any(|key| line_range.contains(key))
    }

    pub fn extract_comments_to_line(
        &mut self,
        starting_line_number: LineNumber,
        line_number: LineNumber,
    ) -> Option<(CommentBlock, LineNumber)> {
        self.other_comments
            .keys()
            .next()
            .copied()
            .map(|lowest_line| {
                let remaining_comments = self.other_comments.split_off(&(&line_number + 1));
                let comments = mem::replace(&mut self.other_comments, remaining_comments)
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
