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

extern "C" fn hash_iter_callback(ba: VALUE, v: VALUE, argc: libc::c_int, _argv: *const VALUE) -> VALUE {
    unsafe {
        let fc_ptr = v.as_void_ptr() as *mut FileComments;
        debug_inspect(ba);
        eprintln!("here");
        if argc != 1 {
            raise("u fucked up ur hash iteration");
        }
        let ruby_lineno = rb_ary_entry(ba, 0);
        let lineno = rubyfmt_rb_num2ll(ruby_lineno);
        if lineno < 0 {
            raise("line number negative");
        }
        let lineno = lineno as u64;
        let ruby_comment = rb_ary_entry(ba, 1);

        let comment_slice = std::slice::from_raw_parts(
            rubyfmt_rstring_ptr(ruby_comment) as *const u8,
            rubyfmt_rstring_len(ruby_comment) as usize
        );

        eprintln!("here3");
        let comment = std::str::from_utf8_unchecked(comment_slice).to_string();
        eprintln!("here4");
        (&mut *fc_ptr).push_comment(lineno, comment);
        eprintln!("here2");
        return Qnil;
    }
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
            eprint!("{:?}", rubyfmt_rb_type(h));
        }
        unsafe {
            let _result = rb_block_call(h,
                intern!("each"),
                0,
                std::ptr::null(),
                hash_iter_callback,
                VALUE::from_void_ptr(&mut fc as *mut FileComments as _),
            );
        };
        fc
    }

    fn push_comment(&mut self, line_number: u64, l: String) {
        eprintln!("here5");
        if self.lowest_key == 0 {
            self.lowest_key = line_number;
        }
        eprintln!("here6");

        let last_line = self.contiguous_starting_indices.last();
        eprintln!("here7");

        let should_push = line_number == 1
            || (last_line.is_some() && last_line.unwrap() == &(line_number - 1));
        eprintln!("here8");
        if should_push {
            self.contiguous_starting_indices.push(line_number);
        }
        eprintln!("here9");
        self.comment_blocks.insert(line_number, l);
        eprintln!("here10");
    }

    pub fn has_start_of_file_sled(&self) -> bool {
        self.lowest_key == 1
    }

    pub fn take_start_of_file_sled(&mut self) -> Option<CommentBlock> {
        eprintln!("s: {:?}", self);
        if !self.has_start_of_file_sled() {
            return None;
        }
        eprintln!("{:?}", self);

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
