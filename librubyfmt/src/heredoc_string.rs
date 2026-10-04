use std::borrow::Cow;

use crate::types::ColNumber;
use crate::util::get_indent;

fn append_raw_segment(
    result: &mut Vec<u8>,
    content: &[u8],
    preserve_whitespace: bool,
    at_line_start: &mut bool,
) {
    if preserve_whitespace {
        // Quoted-string interiors are the literal's value. Do not trim, and do
        // not treat an interior newline as a heredoc line boundary: indenting
        // the following quote would mutate the string.
        result.extend_from_slice(content);
        if !content.is_empty() {
            *at_line_start = false;
        }
    } else {
        append_heredoc_lines(result, content, false, None);
        if !content.is_empty() {
            *at_line_start = content.ends_with(b"\n");
        }
    }
}

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
/// receive squiggly indentation and content that must be left alone.
#[derive(Debug, Clone)]
pub enum HeredocSegment {
    Normal(Vec<u8>),
    /// Content that must not receive squiggly indentation.
    ///
    /// `preserve_whitespace` is for quoted-string interiors nested in an
    /// interpolation: those bytes are the string's value, so they must not be
    /// trimmed, and a newline inside them is not a heredoc line boundary.
    /// Nested non-squiggly heredocs use `preserve_whitespace: false`.
    Raw {
        content: Vec<u8>,
        preserve_whitespace: bool,
    },
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
            // Indent Normal segments at real line starts. Raw segments (nested
            // non-squiggly heredocs, quoted-string interiors) are left unindented.
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
                    HeredocSegment::Raw {
                        content,
                        preserve_whitespace,
                    } => {
                        append_raw_segment(
                            &mut result,
                            &content,
                            preserve_whitespace,
                            &mut at_line_start,
                        );
                    }
                }
            }
            result
        } else {
            // Non-squiggly heredocs trim line endings of body text, but leave
            // preserving raw segments (quoted-string interiors) untouched.
            let mut result = Vec::new();
            let mut at_line_start = true;
            for segment in self.segments {
                match segment {
                    HeredocSegment::Raw {
                        content,
                        preserve_whitespace,
                    } => {
                        append_raw_segment(
                            &mut result,
                            &content,
                            preserve_whitespace,
                            &mut at_line_start,
                        );
                    }
                    HeredocSegment::Normal(content) => {
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
