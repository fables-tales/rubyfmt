use std::collections::BTreeMap;

use crate::comment_block::CommentBlock;
use crate::ruby::*;
use crate::types::LineNumber;

#[derive(Debug, Default)]
pub struct FileComments {
    comment_blocks: BTreeMap<LineNumber, String>,
    contiguous_starting_indices: Vec<LineNumber>,
    lowest_key: LineNumber,
}

impl FileComments {
    pub fn from_ruby_hash(h: VALUE) -> Self {
        let mut fc = FileComments::default();
        let keys;
        let values;
        unsafe {
            keys = ruby_array_to_slice(rb_funcall(h, intern!("keys"), 0));
            values = ruby_array_to_slice(rb_funcall(h, intern!("values"), 0));
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
        fc
    }

    fn push_comment(&mut self, line_number: u64, l: String) {
        if self.lowest_key == 0 {
            self.lowest_key = line_number;
        }

        let last_line = self.contiguous_starting_indices.last();

        let should_push =
            line_number == 1 || (last_line.is_some() && last_line.unwrap() == &(line_number - 1));
        if should_push {
            self.contiguous_starting_indices.push(line_number);
        }
        self.comment_blocks.insert(line_number, l);
    }

    pub fn has_start_of_file_sled(&self) -> bool {
        self.lowest_key == 1
    }

    pub fn take_start_of_file_sled(&mut self) -> Option<CommentBlock> {
        if !self.has_start_of_file_sled() {
            return None;
        }

        let mut sled = Vec::with_capacity(self.contiguous_starting_indices.len());
        for key in self.contiguous_starting_indices.iter() {
            sled.push(
                self.comment_blocks
                    .remove(key)
                    .unwrap_or_else(|| panic!("we tracked it: {} {:?}", key, self.comment_blocks)),
            );
        }

        Some(CommentBlock::new(sled))
    }

    pub fn extract_comments_to_line(&mut self, line_number: LineNumber) -> Option<CommentBlock> {
        if line_number < self.lowest_key {
            return None;
        }

        let mut values = Vec::new();
        let keys: Vec<_> = self
            .comment_blocks
            .range(self.lowest_key..=line_number)
            .map(|(&k, &_)| k)
            .collect();
        for key in keys {
            let v = self.comment_blocks.remove(&key).expect("came from key");
            values.push(v);
        }

        Some(CommentBlock::new(values))
    }
}
