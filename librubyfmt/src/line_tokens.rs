use crate::render_targets::{BreakableEntry, ConvertType};

// represents something that will actually end up as a ruby token, as opposed to
// something that has to be transformd to become a ruby token
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
            Self::Comment { contents } => format!("{}\n", contents),
            Self::Delim { contents } => contents,
            Self::End => "end".to_string(),
        }
    }

    fn is_block_closing_token(&self) -> bool {
        eprintln!("{:?}", self);

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

    fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::DirectPart { part } => (part == &"".to_string()),
            Self::Comma => true,
            Self::Space => true,
            _ => false,
        }
    }

    fn into_line_token(self) -> LineToken {
        LineToken::ConcreteLineToken(self)
    }
}

#[derive(Debug, Clone)]
pub enum LineToken {
    // this is all bodil's fault
    ConcreteLineToken(ConcreteLineToken),
    CollapsingNewLine,
    SoftNewline,
    SoftIndent { depth: u32 },
    BreakableEntry(BreakableEntry),
}

impl LineToken {
    pub fn into_single_line(self) -> LineToken {
        match self {
            Self::CollapsingNewLine => ConcreteLineToken::DirectPart {
                part: "".to_string(),
            }
            .into_line_token(),
            Self::SoftNewline => ConcreteLineToken::Space.into_line_token(),
            Self::SoftIndent { .. } => ConcreteLineToken::DirectPart {
                part: "".to_string(),
            }
            .into_line_token(),
            Self::ConcreteLineToken(clt) => Self::ConcreteLineToken(clt),
            Self::BreakableEntry(be) => Self::BreakableEntry(be),
        }
    }

    pub fn into_multi_line(self) -> LineToken {
        match self {
            Self::CollapsingNewLine => ConcreteLineToken::HardNewLine.into_line_token(),
            Self::SoftNewline => ConcreteLineToken::HardNewLine.into_line_token(),
            Self::SoftIndent { depth } => ConcreteLineToken::Indent { depth }.into_line_token(),
            Self::ConcreteLineToken(clt) => Self::ConcreteLineToken(clt),
            Self::BreakableEntry(be) => Self::BreakableEntry(be),
        }
    }

    pub fn is_indent(&self) -> bool {
        matches!(
            self,
            Self::ConcreteLineToken(ConcreteLineToken::Indent { .. })
        )
    }

    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            Self::ConcreteLineToken(ConcreteLineToken::Comment { .. })
        )
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::ConcreteLineToken(ConcreteLineToken::HardNewLine) => true,
            Self::SoftNewline => true,
            Self::CollapsingNewLine => true,
            Self::ConcreteLineToken(ConcreteLineToken::DirectPart { part }) => {
                if part == "\n" {
                    panic!("shouldn't ever have a single newline direct part");
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn into_ruby(self) -> String {
        match self {
            Self::CollapsingNewLine => "\n".to_string(),
            Self::SoftNewline => "\n".to_string(),
            Self::SoftIndent { depth } => (0..depth).map(|_| ' ').collect(),
            Self::BreakableEntry(be) => be
                .into_tokens(ConvertType::SingleLine)
                .into_iter()
                .fold("".to_string(), |accum, tok| {
                    format!("{}{}", accum, tok.into_ruby())
                }),
            Self::ConcreteLineToken(clt) => clt.into_ruby(),
        }
    }

    pub fn is_in_need_of_a_trailing_blankline(&self) -> bool {
        self.is_conditional_spaced_token() && !self.is_block_closing_token()
    }

    pub fn is_block_closing_token(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_block_closing_token(),
            _ => false,
        }
    }

    pub fn is_conditional_spaced_token(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_conditional_spaced_token(),
            _ => true,
        }
    }

    pub fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::SoftNewline => true,
            Self::ConcreteLineToken(clt) => clt.is_single_line_breakable_garbage(),
            _ => false,
        }
    }
}
