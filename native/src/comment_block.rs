use crate::line_tokens::LineToken;

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
            .map(|v| LineToken::Comment { contents: v })
            .collect()
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
