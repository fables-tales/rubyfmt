use std::collections::BTreeMap;
use std::io::{self, BufRead, Read};

use crate::comment_block::CommentBlock;
use crate::types::LineNumber;

use regex::Regex;

#[derive(Debug)]
pub struct LineMetadata {
    comment_blocks: BTreeMap<LineNumber, String>,
    lowest_key: LineNumber,
}

impl LineMetadata {
    fn new() -> Self {
        LineMetadata {
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
            .range(self.lowest_key..line_number + 1)
            .map(|(&k, &_)| k)
            .collect();
        for key in keys {
            let v = self.comment_blocks.remove(&key).expect("came from key");
            values.push(v);
        }

        Some(CommentBlock::new(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulls_lines_correctly() {
        let data = "# foo\ndef bees\n  # comment 2";
        let parsed = LineMetadata::from_buf(io::BufReader::new(data.as_bytes())).expect("ok");
        assert_eq!(parsed.lowest_key, 1);
        assert_eq!(parsed.comment_blocks.get(&1).unwrap(), &"# foo".to_string());
        assert_eq!(
            parsed.comment_blocks.get(&3).unwrap(),
            &"  # comment 2".to_string()
        );
    }

    #[test]
    fn test_extract_to_line() {
        let data = "# foo\ndef bees\n  # comment 2";
        let mut parsed = LineMetadata::from_buf(io::BufReader::new(data.as_bytes())).expect("ok");
        let res = parsed.extract_comments_to_line(1).expect("the comments");
        assert_eq!(res.get_comments()[0], "# foo");
    }

    #[test]
    fn text_extract_to_line_before() {
        let data = "a\nb\n# foo\n";
        let mut parsed = LineMetadata::from_buf(io::BufReader::new(data.as_bytes())).expect("ok");
        let res = parsed.extract_comments_to_line(1);
        assert!(res.is_none());
    }
}
