use crate::line_tokens::LineToken;
use crate::types::ColNumber;

#[derive(Debug)]
pub struct CommentBlock {
    comments: Vec<String>,
}

impl CommentBlock {
    pub fn new(comments: Vec<String>) -> Self {
        CommentBlock { comments }
    }

    pub fn into_line_tokens(self) -> Vec<LineToken> {
        self.comments
            .into_iter()
            .map(|c| LineToken::Comment { contents: c })
            .collect()
    }

    pub fn apply_spaces(self, indent_depth: ColNumber) -> Self {
        let new_strings = self
            .comments
            .into_iter()
            .map(|c| {
                let spaces = (0..indent_depth)
                    .map(|_| " ".to_string())
                    .collect::<Vec<String>>()
                    .join("");
                format!("{}{}", spaces, c)
            })
            .collect();
        Self::new(new_strings)
    }

    pub fn has_comments(&self) -> bool {
        !self.comments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.comments.len()
    }

    pub fn merge(&mut self, mut other: CommentBlock) {
        self.comments.append(&mut other.comments);
    }
}
