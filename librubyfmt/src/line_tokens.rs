use crate::render_targets::{AbstractTokenTarget, BreakableEntry, ConvertType};

// represents something that will actually end up as a ruby token, as opposed to
// something that has to be transformed to become a ruby token
#[derive(Debug, Clone)]
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
            _ => true,
        }
    }

    pub fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::DirectPart { part } => (part == &"".to_string()),
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConcreteLineTokenAndTargets {
    ConcreteLineToken(ConcreteLineToken),
    BreakableEntry(BreakableEntry),
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
            Self::ConcreteLineToken(clt) => clt.into_ruby(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AbstractLineToken {
    // this is all bodil's fault
    ConcreteLineToken(ConcreteLineToken),
    CollapsingNewLine,
    SoftNewline,
    SoftIndent { depth: u32 },
    BreakableEntry(BreakableEntry),
}

impl AbstractLineToken {
    pub fn into_single_line(self) -> ConcreteLineTokenAndTargets {
        match self {
            Self::CollapsingNewLine => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart {
                    part: "".to_string(),
                })
            }
            Self::SoftNewline => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Space)
            }
            Self::SoftIndent { .. } => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart {
                    part: "".to_string(),
                })
            }
            Self::ConcreteLineToken(clt) => ConcreteLineTokenAndTargets::ConcreteLineToken(clt),
            Self::BreakableEntry(be) => ConcreteLineTokenAndTargets::BreakableEntry(be),
        }
    }

    pub fn into_multi_line(self) -> ConcreteLineTokenAndTargets {
        match self {
            Self::CollapsingNewLine => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HardNewLine)
            }
            Self::SoftNewline => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HardNewLine)
            }
            Self::SoftIndent { depth } => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Indent { depth })
            }
            Self::ConcreteLineToken(clt) => ConcreteLineTokenAndTargets::ConcreteLineToken(clt),
            Self::BreakableEntry(be) => ConcreteLineTokenAndTargets::BreakableEntry(be),
        }
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
            Self::SoftNewline => true,
            Self::CollapsingNewLine => true,
            _ => false,
        }
    }
}
