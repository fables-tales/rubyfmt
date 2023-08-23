use crate::heredoc_string::{HeredocKind, HeredocString};
use crate::render_targets::{
    AbstractTokenTarget, BreakableCallChainEntry, BreakableEntry, ConvertType,
};
use crate::types::ColNumber;

pub fn cltats_hard_newline() -> ConcreteLineTokenAndTargets {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HardNewLine)
}

pub fn clats_direct_part(part: String) -> ConcreteLineTokenAndTargets {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart { part })
}

pub fn clats_heredoc_close(symbol: String) -> ConcreteLineTokenAndTargets {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HeredocClose { symbol })
}

pub fn clats_indent(depth: ColNumber) -> ConcreteLineTokenAndTargets {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Indent { depth })
}

// represents something that will actually end up as a ruby token, as opposed to
// something that has to be transformed to become a ruby token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcreteLineToken {
    HardNewLine,
    Indent { depth: u32 },
    Keyword { keyword: String },
    DefKeyword,
    ClassKeyword,
    ModuleKeyword,
    DoKeyword,
    ModKeyword { contents: String },
    ConditionalKeyword { contents: String },
    DirectPart { part: String },
    CommaSpace,
    Comma,
    Space,
    Dot,
    Ellipsis,
    ColonColon,
    LonelyOperator,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenCurlyBracket,
    CloseCurlyBracket,
    OpenParen,
    CloseParen,
    Op { op: String },
    DoubleQuote,
    LTStringContent { content: String },
    SingleSlash,
    Comment { contents: String },
    Delim { contents: String },
    End,
    HeredocClose { symbol: String },
    DataEnd,
    // These are "magic" tokens. They have no concrete representation,
    // but they're meaningful inside of the render queue
    AfterCallChain,
    BeginCallChainIndent,
    EndCallChainIndent,
    HeredocStart { kind: HeredocKind, symbol: String },
}

impl ConcreteLineToken {
    pub fn into_ruby(self) -> String {
        match self {
            Self::HardNewLine => "\n".to_string(),
            Self::Indent { depth } => (0..depth).map(|_| ' ').collect(),
            Self::Keyword { keyword } => keyword,
            Self::ModKeyword { contents } => contents,
            Self::ConditionalKeyword { contents } => contents,
            Self::DoKeyword => "do".to_string(),
            Self::ClassKeyword => "class".to_string(),
            Self::DefKeyword => "def".to_string(),
            Self::ModuleKeyword => "module".to_string(),
            Self::DirectPart { part } => part,
            Self::CommaSpace => ", ".to_string(),
            Self::Comma => ",".to_string(),
            Self::Space => " ".to_string(),
            Self::Dot => ".".to_string(),
            Self::Ellipsis => "...".to_string(),
            Self::ColonColon => "::".to_string(),
            Self::LonelyOperator => "&.".to_string(),
            Self::OpenSquareBracket => "[".to_string(),
            Self::CloseSquareBracket => "]".to_string(),
            Self::OpenCurlyBracket => "{".to_string(),
            Self::CloseCurlyBracket => "}".to_string(),
            Self::OpenParen => "(".to_string(),
            Self::CloseParen => ")".to_string(),
            Self::Op { op } => op,
            Self::DoubleQuote => "\"".to_string(),
            Self::LTStringContent { content } => content,
            Self::SingleSlash => "\\".to_string(),
            Self::Comment { contents } => contents,
            Self::Delim { contents } => contents,
            Self::End => "end".to_string(),
            Self::HeredocClose { symbol } => symbol,
            Self::DataEnd => "__END__".to_string(),
            Self::HeredocStart { kind, symbol } => {
                let mut kind_str = match kind {
                    HeredocKind::Bare => "<<".to_string(),
                    HeredocKind::Dash => "<<-".to_string(),
                    HeredocKind::Squiggly => "<<~".to_string(),
                };
                kind_str.push_str(&symbol);
                kind_str
            }
            // no-op, this is purely semantic information
            // for the render queue
            Self::AfterCallChain | Self::BeginCallChainIndent | Self::EndCallChainIndent => {
                "".to_string()
            }
        }
    }

    /// The length of the token's string representation
    pub fn len(&self) -> usize {
        use ConcreteLineToken::*;
        // The alternative to this match condition would be to clone and render
        // each individual string token, which would increase the allocations of rubyfmt
        // by an order of magnitude
        match self {
            AfterCallChain | BeginCallChainIndent | EndCallChainIndent => 0, // purely semantic tokens, don't render
            HeredocStart { kind, symbol } => {
                symbol.len()
                    + match kind {
                        HeredocKind::Bare => 2,                         // <<
                        HeredocKind::Dash | HeredocKind::Squiggly => 3, // <<- or <<~
                    }
            }
            Indent { depth } => *depth as usize,
            Keyword { keyword: contents }
            | Op { op: contents }
            | DirectPart { part: contents }
            | LTStringContent { content: contents }
            | Comment { contents }
            | Delim { contents }
            | ConditionalKeyword { contents }
            | HeredocClose { symbol: contents }
            | ModKeyword { contents } => contents.len(),
            HardNewLine | Comma | Space | Dot | OpenSquareBracket | CloseSquareBracket
            | OpenCurlyBracket | CloseCurlyBracket | OpenParen | CloseParen | SingleSlash
            | DoubleQuote => 1,
            DoKeyword | CommaSpace | LonelyOperator | ColonColon => 2,
            DefKeyword | Ellipsis | End => 3, // "def"/"..."/"end"
            ClassKeyword => 5,                // "class"
            ModuleKeyword => 6,               // "module"
            DataEnd => 7,                     // "__END__"
        }
    }

    fn is_block_closing_token(&self) -> bool {
        match self {
            Self::End => true,
            Self::DirectPart { part } => part == "}" || part == "]" || part == ")",
            Self::Delim { contents } => contents == "}" || contents == "]" || contents == ")",
            _ => false,
        }
    }

    fn is_conditional_spaced_token(&self) -> bool {
        match self {
            Self::ConditionalKeyword { contents } => !(contents == "else" || contents == "elsif"),
            Self::Dot => false,
            Self::DirectPart { part } => part != "&.",
            _ => true,
        }
    }

    pub fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::DirectPart { part } => part == &"".to_string(),
            Self::Comma => true,
            Self::Space => true,
            _ => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::HardNewLine => true,
            Self::DirectPart { part } => {
                if part == "\n" {
                    panic!("shouldn't ever have a single newline direct part");
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn is_indent(&self) -> bool {
        matches!(self, ConcreteLineToken::Indent { .. })
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, Self::Indent { .. })
    }

    pub fn is_in_need_of_a_trailing_blankline(&self) -> bool {
        self.is_conditional_spaced_token() && !self.is_block_closing_token()
    }
}

impl From<ConcreteLineToken> for ConcreteLineTokenAndTargets {
    fn from(clt: ConcreteLineToken) -> ConcreteLineTokenAndTargets {
        ConcreteLineTokenAndTargets::ConcreteLineToken(clt)
    }
}

impl From<ConcreteLineTokenAndTargets> for AbstractLineToken {
    fn from(cltat: ConcreteLineTokenAndTargets) -> AbstractLineToken {
        match cltat {
            ConcreteLineTokenAndTargets::BreakableEntry(be) => {
                AbstractLineToken::BreakableEntry(be)
            }
            ConcreteLineTokenAndTargets::ConcreteLineToken(clt) => {
                AbstractLineToken::ConcreteLineToken(clt)
            }
            ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce) => {
                AbstractLineToken::BreakableCallChainEntry(bcce)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConcreteLineTokenAndTargets {
    ConcreteLineToken(ConcreteLineToken),
    BreakableEntry(BreakableEntry),
    BreakableCallChainEntry(BreakableCallChainEntry),
}

impl ConcreteLineTokenAndTargets {
    pub fn is_newline(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_newline(),
            _ => false,
        }
    }

    pub fn is_comment(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_comment(),
            _ => false,
        }
    }

    pub fn into_ruby(self) -> String {
        match self {
            Self::BreakableEntry(be) => be
                .into_tokens(ConvertType::SingleLine)
                .into_iter()
                .fold("".to_string(), |accum, tok| {
                    format!("{}{}", accum, tok.into_ruby())
                }),
            Self::BreakableCallChainEntry(bcce) => bcce
                .into_tokens(ConvertType::SingleLine)
                .into_iter()
                .fold("".to_string(), |accum, tok| {
                    format!("{}{}", accum, tok.into_ruby())
                }),
            Self::ConcreteLineToken(clt) => clt.into_ruby(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AbstractLineToken {
    // this is all bodil's fault
    ConcreteLineToken(ConcreteLineToken),
    CollapsingNewLine(Option<Vec<HeredocString>>),
    SoftNewline(Option<Vec<HeredocString>>),
    SoftIndent { depth: u32 },
    BreakableEntry(BreakableEntry),
    BreakableCallChainEntry(BreakableCallChainEntry),
}

impl AbstractLineToken {
    pub fn into_single_line(self) -> Vec<ConcreteLineTokenAndTargets> {
        match self {
            Self::CollapsingNewLine(heredoc_strings) => {
                let mut res = Vec::new();
                if heredoc_strings.is_some() {
                    res.push(cltats_hard_newline());
                }
                res.extend(Self::shimmy_and_shake_heredocs(heredoc_strings));
                res
            }
            Self::SoftNewline(heredoc_strings) => {
                let mut res = vec![ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::Space,
                )];
                res.extend(Self::shimmy_and_shake_heredocs(heredoc_strings));
                res
            }
            Self::SoftIndent { .. } => Vec::new(),
            Self::ConcreteLineToken(clt) => {
                vec![ConcreteLineTokenAndTargets::ConcreteLineToken(clt)]
            }
            Self::BreakableEntry(be) => vec![ConcreteLineTokenAndTargets::BreakableEntry(be)],
            Self::BreakableCallChainEntry(bcce) => {
                vec![ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce)]
            }
        }
    }

    pub fn into_multi_line(self) -> Vec<ConcreteLineTokenAndTargets> {
        match self {
            Self::CollapsingNewLine(heredoc_strings) => {
                let mut res = vec![cltats_hard_newline()];
                res.extend(Self::shimmy_and_shake_heredocs(heredoc_strings));
                res
            }
            Self::SoftNewline(heredoc_strings) => {
                let mut res = vec![cltats_hard_newline()];
                res.extend(Self::shimmy_and_shake_heredocs(heredoc_strings));
                res
            }
            Self::SoftIndent { depth } => {
                vec![ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::Indent { depth },
                )]
            }
            Self::ConcreteLineToken(clt) => {
                vec![ConcreteLineTokenAndTargets::ConcreteLineToken(clt)]
            }
            Self::BreakableEntry(be) => vec![ConcreteLineTokenAndTargets::BreakableEntry(be)],
            Self::BreakableCallChainEntry(bcce) => {
                vec![ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce)]
            }
        }
    }

    fn shimmy_and_shake_heredocs(
        heredoc_strings: Option<Vec<HeredocString>>,
    ) -> Vec<ConcreteLineTokenAndTargets> {
        let mut res = vec![];
        if let Some(values) = heredoc_strings {
            for hds in values {
                let indent = hds.indent;
                let kind = hds.kind;
                let symbol = hds.closing_symbol();

                let s = hds.render_as_string();
                if !s.is_empty() {
                    res.push(clats_direct_part(s));
                    res.push(cltats_hard_newline());
                }
                if !kind.is_bare() {
                    res.push(clats_indent(indent));
                }
                res.push(clats_heredoc_close(symbol));
                res.push(cltats_hard_newline());
                let indent_depth = if indent != 0 { indent - 2 } else { indent };
                res.push(clats_indent(indent_depth));
            }
        }
        res
    }

    pub fn is_comment(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_comment(),
            _ => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_newline(),
            Self::SoftNewline(_) => true,
            Self::CollapsingNewLine(_) => true,
            _ => false,
        }
    }
}
