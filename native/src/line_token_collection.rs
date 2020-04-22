use crate::comment_block::CommentBlock;
use crate::line_tokens::LineToken;

#[derive(Debug, Clone)]
pub struct LineTokenCollection {
    contents: Vec<LineToken>,
}

fn insert_at<T>(idx: usize, target: &mut Vec<T>, input: &mut Vec<T>) {
    let drain = input.drain(..);
    let mut idx = idx;
    for item in drain {
        target.insert(idx, item);
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_insert_at() {
        let mut a1 = vec![3, 2, 1];
        let mut a2 = a1.clone();
        a1.insert(1, 4);
        super::insert_at(1, &mut a2, &mut vec![4]);
        assert_eq!(a1, a2);
    }
}

impl LineTokenCollection {
    pub fn new() -> Self {
        LineTokenCollection { contents: vec![] }
    }

    pub fn into_line_tokens(self) -> Vec<LineToken> {
        self.contents
    }

    pub fn push(&mut self, lt: LineToken) {
        self.contents.push(lt);
    }

    pub fn insert_extra_newline_at_last_newline(&mut self) {
        let idx = self.index_of_prev_hard_newline();
        let insert_idx = match idx {
            Some(idx) => idx + 1,
            None => 0,
        };

        self.contents.insert(insert_idx, LineToken::HardNewLine);
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.contents
            .last()
            .map(|x| x.is_newline())
            .unwrap_or(false)
    }

    fn index_of_prev_hard_newline(&self) -> Option<usize> {
        self.contents
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment())
    }

    pub fn single_line_string_length(&self) -> usize {
        self.contents
            .iter()
            .map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby().len())
            .sum()
    }

    pub fn insert_comments_at_last_hard_newline(&mut self, cb: CommentBlock) {
        let idx_of_prev_hard_newline = self.index_of_prev_hard_newline();
        let insert_index = match idx_of_prev_hard_newline {
            Some(idx) => idx + 1,
            None => 0,
        };

        insert_at(insert_index, &mut self.contents, &mut cb.into_line_tokens());
    }
}
