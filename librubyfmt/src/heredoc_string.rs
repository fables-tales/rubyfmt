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
    pub fn from_string(kind_str: &str) -> Self {
        if kind_str.contains('~') {
            HeredocKind::Squiggly
        } else if kind_str.contains('-') {
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
    Normal(String),
    /// Content from nested non-squiggly heredocs, should never receive squiggly indentation.
    /// This includes both the heredoc content and the closing identifier.
    Raw(String),
}

#[derive(Debug, Clone)]
pub struct HeredocString<'src> {
    symbol: Cow<'src, str>,
    pub kind: HeredocKind,
    pub segments: Vec<HeredocSegment>,
    pub indent: ColNumber,
}

impl<'src> HeredocString<'src> {
    pub fn new(
        symbol: Cow<'src, str>,
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

    pub fn render_as_string(self) -> String {
        let indent = self.indent;

        if self.kind.is_squiggly() {
            // For squiggly heredocs, we need to apply indentation to Normal segments
            // but not to Raw segments (which come from nested non-squiggly heredocs).
            let mut result = String::new();
            for segment in self.segments {
                match segment {
                    HeredocSegment::Normal(content) => {
                        // Apply squiggly indentation to each line
                        for (i, line) in content.split('\n').enumerate() {
                            if i > 0 {
                                result.push('\n');
                            }
                            let indented = format!("{}{}", get_indent(indent as usize + 2), line);
                            result.push_str(indented.trim_end());
                        }
                    }
                    HeredocSegment::Raw(content) => {
                        // No indentation for raw content (nested non-squiggly heredocs)
                        for (i, line) in content.split('\n').enumerate() {
                            if i > 0 {
                                result.push('\n');
                            }
                            result.push_str(line.trim_end());
                        }
                    }
                }
            }
            result
        } else {
            // For non-squiggly heredocs, just join segments and trim line endings
            let mut result = String::new();
            for segment in self.segments {
                let content = match segment {
                    HeredocSegment::Normal(s) | HeredocSegment::Raw(s) => s,
                };
                for (i, line) in content.split('\n').enumerate() {
                    if i > 0 {
                        result.push('\n');
                    }
                    result.push_str(line.trim_end());
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
    pub fn closing_symbol(&self) -> String {
        self.symbol.replace(['\'', '"'], "")
    }
}
