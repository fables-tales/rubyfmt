use crate::types::ColNumber;

#[derive(Debug, Clone)]
pub struct BreakableEntry {
    spaces: ColNumber,
    tokens: Vec<LineToken>,
}

impl BreakableEntry {
    pub fn new(spaces: ColNumber) -> Self {
        BreakableEntry {
            spaces,
            tokens: vec![],
        }
    }

    pub fn push(&mut self, lt: LineToken) {
        self.tokens.push(lt);
    }

    pub fn as_tokens(self) -> Vec<LineToken> {
        self.tokens
    }

    pub fn single_line_string_length(&self) -> usize {
        self.tokens.iter().map(|tok| {
            if tok.is_indent() {
                tok.clone().as_multi_line()
            } else {
                tok.clone().as_single_line()
            }
        }).map(|tok| tok.to_string().len()).sum()
    }
}

#[derive(Debug, Clone)]
pub enum LineToken {
    // this is all bodil's fault
    CollapsingNewLine,
    HardNewLine,
    SoftNewline,
    Indent { depth: u32 },
    SoftIndent { depth: u32 },
    Keyword { keyword: String },
    DirectPart { part: String },
    CommaSpace,
    Comma,
    Space,
    Dot,
    ColonColon,
    LonelyOperator,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenParen,
    CloseParen,
    BreakableEntry(BreakableEntry),
    Op { op: String },
    DoubleQuote,
    LTStringContent { content: String },
    SingleSlash,
    Comment { contents: String },
}

impl LineToken {
    pub fn as_single_line(self) -> LineToken {
        match self {
            Self::CollapsingNewLine => LineToken::DirectPart {
                part: "".to_string(),
            },
            Self::SoftNewline => LineToken::Space,
            Self::SoftIndent { depth: _ } => LineToken::DirectPart {
                part: "".to_string(),
            },
            x => x,
        }
    }

    pub fn as_multi_line(self) -> LineToken {
        self
    }

    pub fn is_indent(&self) -> bool {
        match self {
            Self::SoftIndent{..} => true,
            Self::Indent{..} => true,
            _ => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::HardNewLine => true,
            Self::SoftNewline => true,
            Self::CollapsingNewLine => true,
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

    pub fn to_string(self) -> String {
        match self {
            Self::CollapsingNewLine => "\n".to_string(),
            Self::HardNewLine => "\n".to_string(),
            Self::SoftNewline => "\n".to_string(),
            Self::Indent { depth } => (0..depth).map(|_| ' ').collect(),
            Self::SoftIndent { depth } => (0..depth).map(|_| ' ').collect(),
            Self::Keyword { keyword } => keyword,
            Self::DirectPart { part } => part,
            Self::CommaSpace => ", ".to_string(),
            Self::Comma => ",".to_string(),
            Self::Space => " ".to_string(),
            Self::Dot => ".".to_string(),
            Self::ColonColon => "::".to_string(),
            Self::LonelyOperator => "&.".to_string(),
            Self::OpenSquareBracket => "[".to_string(),
            Self::CloseSquareBracket => "]".to_string(),
            Self::OpenParen => "(".to_string(),
            Self::CloseParen => ")".to_string(),
            Self::BreakableEntry(BreakableEntry { spaces: _, tokens }) => {
                tokens.into_iter().fold("".to_string(), |accum, tok| {
                    format!("{}{}", accum, tok.to_string()).to_string()
                })
            }
            Self::Op { op } => op,
            Self::DoubleQuote => "\"".to_string(),
            Self::LTStringContent { content } => content,
            Self::SingleSlash => "\\".to_string(),
            Self::Comment { contents } => format!("{}\n", contents),
        }
    }

    pub fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::Comma => true,
            Self::Space => true,
            Self::SoftNewline => true,
            Self::DirectPart{part} => (part == &"".to_string()),
            _ => false,
        }
    }
}
