use crate::types::ColNumber;

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

#[derive(Debug, Clone)]
pub struct HeredocString {
    symbol: String,
    pub kind: HeredocKind,
    pub buf: Vec<u8>,
    pub indent: ColNumber,
}

impl HeredocString {
    pub fn new(symbol: String, kind: HeredocKind, buf: Vec<u8>, indent: ColNumber) -> Self {
        HeredocString {
            symbol,
            kind,
            buf,
            indent,
        }
    }

    pub fn render_as_string(self) -> String {
        let indent = self.indent;
        let string = String::from_utf8(self.buf).expect("heredoc is utf8");

        if self.kind.is_squiggly() {
            string
                .split('\n')
                .map(|l| {
                    String::from(format!("{}{}", " ".repeat(indent as usize + 2), l).trim_end())
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            string
                .split('\n')
                .map(|l| l.trim_end())
                .collect::<Vec<&str>>()
                .join("\n")
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
        self.symbol.replace('\'', "").replace('"', "")
    }
}
