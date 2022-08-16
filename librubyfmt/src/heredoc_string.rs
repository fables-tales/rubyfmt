use crate::types::ColNumber;

#[derive(Debug, Clone)]
pub enum HeredocKind {
    Bare,
    Dash,
    Squiggly,
}

impl HeredocKind {
    pub fn from_string(kind_str: String) -> Self {
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
    pub symbol: String,
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

    pub fn render_as_string(&self) -> String {
        let indent = self.indent;
        let kind = self.kind.clone();
        let mut string = String::from_utf8(self.buf.clone()).expect("heredoc is utf8");

        while string.ends_with('\n') {
            string.pop();
        }

        if kind.is_squiggly() {
            string
                .split('\n')
                .map(|l| String::from(format!("{}{}", " ".repeat(indent as usize), l).trim_end()))
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
}
