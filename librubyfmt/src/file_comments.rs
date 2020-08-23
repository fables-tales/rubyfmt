use std::collections::BTreeMap;

use crate::comment_block::CommentBlock;
use crate::types::LineNumber;
use crate::ruby::*;

#[derive(Debug)]
pub struct FileComments {
    comment_blocks: BTreeMap<LineNumber, String>,
    contiguous_starting_indices: Vec<LineNumber>,
    lowest_key: LineNumber,
}

impl FileComments {
    pub fn new() -> Self {
        FileComments {
            comment_blocks: BTreeMap::new(),
            contiguous_starting_indices: vec![],
            lowest_key: 0,
        }
    }

    pub fn from_ruby_hash(h: VALUE) -> Self {
        let mut fc = FileComments::new();
        unsafe {
            let keys = rb_funcall(h, intern!("keys"), 0);
            let values = rb_funcall(h, intern!("values"), 0);
            if rubyfmt_rb_ary_len(keys) != rubyfmt_rb_ary_len(values) {
                raise("expected keys and values to have same length, indicates error");
            }
            for i in 0..rubyfmt_rb_ary_len(keys) {
                let ruby_lineno = rb_ary_entry(keys, i);
                let ruby_comment = rb_ary_entry(values, i);
                let lineno = rubyfmt_rb_num2ll(ruby_lineno);
                if lineno < 0 {
                    raise("line number negative");
                }
                let lineno = lineno as u64;

                let comment_slice = std::slice::from_raw_parts(
                    rubyfmt_rstring_ptr(ruby_comment) as *const u8,
                    rubyfmt_rstring_len(ruby_comment) as usize
                );

                let comment = std::str::from_utf8_unchecked(comment_slice).trim().to_string();
                fc.push_comment(lineno, comment);
            }
        };
        fc
    }

    fn push_comment(&mut self, line_number: u64, l: String) {
        if self.lowest_key == 0 {
            self.lowest_key = line_number;
        }

        let last_line = self.contiguous_starting_indices.last();

        let should_push = line_number == 1
            || (last_line.is_some() && last_line.unwrap() == &(line_number - 1));
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
            sled.push(self.comment_blocks.remove(key).expect(&format!("we tracked it: {} {:?}", key, self.comment_blocks)));
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
