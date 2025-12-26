use std::ops::Range;

use crate::line_tokens::ConcreteLineToken;
use crate::types::{ColNumber, LineNumber};

#[derive(Clone, Debug)]
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

    pub fn into_line_tokens(self) -> impl Iterator<Item = ConcreteLineToken> {
        self.comments.into_iter().flat_map(|c| {
            [
                ConcreteLineToken::Comment { contents: c },
                ConcreteLineToken::HardNewLine,
            ]
        })
    }

    pub fn apply_spaces(mut self, indent_depth: ColNumber) -> Self {
        let indent = str::repeat(" ", indent_depth as _);
        for comment in &mut self.comments {
            // Ignore empty strings -- these represent blank lines between
            // groups of comments
            if !comment.is_empty() && !comment.starts_with("=begin") {
                comment.insert_str(0, &indent);
            }
        }
        self
    }

    pub fn has_comments(&self) -> bool {
        !self.comments.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.comments
            .iter()
            .map(|comment| comment.lines().count())
            .sum()
    }

    pub fn is_trailing(&self) -> bool {
        self.span.start + 1 == self.span.end
    }
}

pub trait Merge<Other = Self> {
    fn merge(&mut self, other: Other);
}

impl Merge for CommentBlock {
    fn merge(&mut self, mut other: CommentBlock) {
        self.comments.append(&mut other.comments);
    }
}

impl Merge<CommentBlock> for Option<CommentBlock> {
    fn merge(&mut self, other: CommentBlock) {
        if let Some(this) = self {
            this.merge(other)
        } else {
            *self = Some(other)
        }
    }
}
