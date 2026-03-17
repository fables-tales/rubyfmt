use std::borrow::Cow;
use std::io::BufRead;
use std::ops::Range;

use crate::line_tokens::ConcreteLineToken;
use crate::types::{ColNumber, LineNumber};
use crate::util::get_indent;

#[derive(Clone, Debug)]
pub struct CommentBlock {
    span: Range<LineNumber>,
    comments: Vec<Cow<'static, [u8]>>,
}

impl CommentBlock {
    pub fn new(span: Range<LineNumber>, comments: Vec<Cow<'static, [u8]>>) -> Self {
        CommentBlock { span, comments }
    }

    pub fn following_line_number(&self) -> LineNumber {
        self.span.end
    }

    pub fn add_line(&mut self, line: Cow<'static, [u8]>) {
        self.span.end += 1;
        self.comments.push(line);
    }

    pub fn into_line_tokens<'src>(self) -> impl Iterator<Item = ConcreteLineToken<'src>> {
        let comments = self.comments;
        let len = comments.len();

        comments.into_iter().enumerate().flat_map(move |(i, c)| {
            if c.is_empty() {
                // Empty vecs represent blank lines
                // If this is a trailing empty comment (at the end), keep it as an empty Comment token
                // to bypass the HardNewLine deduplication logic. Otherwise convert to just HardNewLine.
                if i == len - 1 {
                    // Trailing empty - keep as empty comment to preserve blank lines
                    vec![
                        ConcreteLineToken::Comment { contents: c },
                        ConcreteLineToken::HardNewLine,
                    ]
                } else {
                    // Between comments - convert to just HardNewLine
                    vec![ConcreteLineToken::HardNewLine]
                }
            } else {
                vec![
                    ConcreteLineToken::Comment { contents: c },
                    ConcreteLineToken::HardNewLine,
                ]
            }
        })
    }

    pub fn apply_spaces(mut self, indent_depth: ColNumber) -> Self {
        if indent_depth == 0 {
            return self;
        }

        let indent = get_indent(indent_depth as usize);
        for comment in &mut self.comments {
            // Ignore empty vecs -- these represent blank lines between
            // groups of comments
            if !comment.is_empty() && !comment.starts_with(b"=begin") {
                comment.to_mut().splice(0..0, indent.iter().copied());
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
