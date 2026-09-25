use std::borrow::Cow;

use crate::types::ColNumber;
use crate::util::get_indent;

fn append_heredoc_lines(
    result: &mut Vec<u8>,
    content: &[u8],
    mut at_line_start: bool,
    squiggly_indent: Option<usize>,
) {
    for (i, line) in content.split(|&b| b == b'\n').enumerate() {
        if i > 0 {
            result.push(b'\n');
            at_line_start = true;
        }
        if at_line_start {
            if let Some(indent) = squiggly_indent {
                let mut indented = get_indent(indent).into_owned();
                indented.extend_from_slice(line);
                result.extend_from_slice(indented.trim_ascii_end());
            } else {
                result.extend_from_slice(line.trim_ascii_end());
            }
        } else {
            result.extend_from_slice(line.trim_ascii_end());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeredocKind {
    Bare,
    Dash,
    Squiggly,
}

impl HeredocKind {
    pub fn from_bytes(kind_bytes: &[u8]) -> Self {
        if kind_bytes.contains(&b'~') {
            HeredocKind::Squiggly
        } else if kind_bytes.contains(&b'-') {
            HeredocKind::Dash
        } else {
            HeredocKind::Bare
        }
    }

    pub fn is_squiggly(&self) -> bool {
        matches!(self, HeredocKind::Squiggly)
    }

    pub fn is_bare(&self) -> bool {
        matches!(self, HeredocKind::Bare)
    }
}

/// A segment of heredoc content. Used to distinguish between content that should
/// receive squiggly indentation and content from nested non-squiggly heredocs
/// that should not be indented.
#[derive(Debug, Clone)]
pub enum HeredocSegment {
    Normal(Vec<u8>),
    /// Content from nested non-squiggly heredocs, should never receive squiggly indentation.
    /// This includes both the heredoc content and the closing identifier.
    Raw(Vec<u8>),
    /// Interior of a nested quoted string. Newlines here are string content, so
    /// they must not receive squiggly indentation or trailing-whitespace trimming.
    Quoted(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct HeredocString<'src> {
    symbol: Cow<'src, [u8]>,
    pub kind: HeredocKind,
    pub segments: Vec<HeredocSegment>,
    pub indent: ColNumber,
}

impl<'src> HeredocString<'src> {
    pub fn new(
        symbol: Cow<'src, [u8]>,
        kind: HeredocKind,
        segments: Vec<HeredocSegment>,
        indent: ColNumber,
    ) -> Self {
        HeredocString {
            symbol,
            kind,
            segments,
            indent,
        }
    }

    pub fn render_as_bytes(self) -> Vec<u8> {
        let indent = self.indent;

        if self.kind.is_squiggly() {
            // For squiggly heredocs, apply indentation to Normal segments at real
            // line starts, but never to Raw (nested non-squiggly heredocs) or Quoted
            // (nested string interiors). Quoted newlines are string content.
            let mut result = Vec::new();
            let mut at_line_start = true;
            for segment in self.segments {
                match segment {
                    HeredocSegment::Normal(content) => {
                        append_heredoc_lines(
                            &mut result,
                            &content,
                            at_line_start,
                            Some(indent as usize + 2),
                        );
                        if !content.is_empty() {
                            at_line_start = content.ends_with(b"\n");
                        }
                    }
                    HeredocSegment::Raw(content) => {
                        append_heredoc_lines(&mut result, &content, false, None);
                        if !content.is_empty() {
                            at_line_start = content.ends_with(b"\n");
                        }
                    }
                    HeredocSegment::Quoted(content) => {
                        // Preserve nested string contents byte-for-byte.
                        // Newlines inside the literal are string content, not
                        // heredoc line boundaries: indenting the following
                        // tokens (the closing quote, etc.) would mutate the
                        // nested string's value.
                        result.extend_from_slice(&content);
                        if !content.is_empty() {
                            at_line_start = false;
                        }
                    }
                }
            }
            result
        } else {
            // For non-squiggly heredocs, join segments and trim line endings of
            // body text, but leave nested quoted-string interiors untouched.
            let mut result = Vec::new();
            for segment in self.segments {
                match segment {
                    HeredocSegment::Quoted(content) => {
                        result.extend_from_slice(&content);
                    }
                    HeredocSegment::Normal(content) | HeredocSegment::Raw(content) => {
                        append_heredoc_lines(&mut result, &content, false, None);
                    }
                }
            }
            result
        }
    }

    /// The symbol with any quotes stripped. We only
    /// store the opening symbol for heredocs, but this
    /// opening symbol can be surrounded with single quotes,
    /// for example:
    ///
    /// ```ruby
    /// <<~'RUBY'
    ///   puts "Hello, World!"
    /// RUBY
    /// ```
    ///
    /// However, the closing symbol should *not* have
    /// quotes, so we must strip them from the symbol when
    /// rendering the closing symbol.
    pub fn closing_symbol(&self) -> Vec<u8> {
        self.symbol
            .iter()
            .filter(|&&b| b != b'\'' && b != b'"')
            .copied()
            .collect()
    }
}
