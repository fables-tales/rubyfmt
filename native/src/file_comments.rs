use std::collections::BTreeMap;
use std::io::{self, BufRead, Read};

use crate::comment_block::CommentBlock;
use crate::types::LineNumber;

use regex::Regex;

#[derive(Debug)]
pub struct FileComments {
    comment_blocks: BTreeMap<LineNumber, String>,
    lowest_key: LineNumber,
}

impl FileComments {
    pub fn new() -> Self {
        FileComments {
            comment_blocks: BTreeMap::new(),
            lowest_key: 0,
        }
    }

    pub fn from_buf<R: Read>(r: io::BufReader<R>) -> io::Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^ *#").unwrap();
        }
        let mut res = Self::new();
        for (idx, line) in r.lines().enumerate() {
            let l = line?;
            if RE.is_match(&l) {
                let line_number = (idx + 1) as LineNumber;
                if res.lowest_key == 0 {
                    res.lowest_key = line_number;
                }
                res.comment_blocks.insert(line_number, l);
            }
        }
        Ok(res)
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
