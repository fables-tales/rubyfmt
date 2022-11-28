use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::ruby::*;

type LineNumber = u64;

#[derive(Clone, Debug, Default)]
pub struct RubyComments {
    pub start_of_file_contiguous_comment_lines: Option<RubyCommentBlock>,
    pub other_comments: BTreeMap<LineNumber, String>,
    pub lines_with_ruby: BTreeSet<LineNumber>,
    pub last_lineno: LineNumber,
}

#[derive(Clone, Debug, Default)]
pub struct RubyCommentBlock {
    span: Range<LineNumber>,
    comments: Vec<String>,
}

impl RubyCommentBlock {
    pub fn following_line_number(&self) -> LineNumber {
        self.span.end
    }

    pub fn add_line(&mut self, line: String) {
        self.span.end += 1;
        self.comments.push(line);
    }
}

impl RubyComments {
    pub fn from_ruby_hash(h: VALUE, rl: VALUE, last_lineno: VALUE) -> Self {
        let mut fc = RubyComments::default();
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

    pub fn push_comment(&mut self, line_number: u64, l: String) {
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
                self.start_of_file_contiguous_comment_lines = Some(RubyCommentBlock {
                    span: 1..2,
                    comments: vec![l],
                });
            }
            (Some(sled), _) if sled.following_line_number() == line_number => {
                sled.add_line(l);
            }
            _ => {
                self.other_comments.insert(line_number, l);
            }
        }
    }
}
