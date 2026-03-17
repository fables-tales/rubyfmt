use std::borrow::Cow;

use crate::types::ColNumber;
use crate::util::get_indent;

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
            // For squiggly heredocs, we need to apply indentation to Normal segments
            // but not to Raw segments (which come from nested non-squiggly heredocs).
            let mut result = Vec::new();
            for segment in self.segments {
                match segment {
                    HeredocSegment::Normal(content) => {
                        // Apply squiggly indentation to each line
                        for (i, line) in content.split(|&b| b == b'\n').enumerate() {
                            if i > 0 {
                                result.push(b'\n');
                            }
                            let mut indented = get_indent(indent as usize + 2).into_owned();
                            indented.extend_from_slice(line);
                            result.extend_from_slice(indented.trim_ascii_end());
                        }
                    }
                    HeredocSegment::Raw(content) => {
                        // No indentation for raw content (nested non-squiggly heredocs)
                        for (i, line) in content.split(|&b| b == b'\n').enumerate() {
                            if i > 0 {
                                result.push(b'\n');
                            }
                            result.extend_from_slice(line.trim_ascii_end());
                        }
                    }
                }
            }
            result
        } else {
            // For non-squiggly heredocs, just join segments and trim line endings
            let mut result = Vec::new();
            for segment in self.segments {
                let content = match segment {
                    HeredocSegment::Normal(s) | HeredocSegment::Raw(s) => s,
                };
                for (i, line) in content.split(|&b| b == b'\n').enumerate() {
                    if i > 0 {
                        result.push(b'\n');
                    }
                    result.extend_from_slice(line.trim_ascii_end());
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
