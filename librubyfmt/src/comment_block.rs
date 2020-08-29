use std::ops::{AddAssign, Range};

use crate::line_tokens::LineToken;
use crate::types::{ColNumber, LineNumber};

#[derive(Debug)]
pub struct CommentBlock {
    span: Range<LineNumber>,
    comments: Vec<String>,
}

impl CommentBlock {
    pub fn new(span: Range<LineNumber>, comments: Vec<String>) -> Self {
        CommentBlock { span, comments }
    }

    pub fn following_line_number(&self) -> LineNumber {
        self.span.end
    }

    pub fn add_line(&mut self, line: String) {
        self.span.end += 1;
        self.comments.push(line);
    }

    pub fn into_line_tokens(self) -> Vec<LineToken> {
        self.comments
            .into_iter()
            .map(|c| LineToken::Comment { contents: c })
            .collect()
    }

    // FIXME: This should be the responsibility of the formatter
    pub fn apply_spaces(mut self, indent_depth: ColNumber) -> Self {
        for comment in &mut self.comments {
            *comment = str::repeat(" ", indent_depth as _) + comment;
        }
        self
    }

    pub fn has_comments(&self) -> bool {
        !self.comments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.comments.len()
    }
}

impl AddAssign for CommentBlock {
    fn add_assign(&mut self, mut rhs: CommentBlock) {
        self.comments.append(&mut rhs.comments);
    }
}

impl AddAssign<CommentBlock> for Option<CommentBlock> {
    fn add_assign(&mut self, rhs: CommentBlock) {
        if let Some(this) = self {
            *this += rhs
        } else {
            *self = Some(rhs)
        }
    }
}
