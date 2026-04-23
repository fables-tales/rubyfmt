use crate::heredoc_string::{HeredocKind, HeredocString};
use crate::render_targets::{BreakableCallChainEntry, BreakableEntry, ConditionalLayoutEntry};
use crate::types::ColNumber;
use crate::util::get_indent;
use std::borrow::Cow;

pub fn cltats_hard_newline<'src>() -> ConcreteLineTokenAndTargets<'src> {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HardNewLine)
}

pub fn clats_direct_part<'src>(
    part: impl Into<Cow<'src, [u8]>>,
) -> ConcreteLineTokenAndTargets<'src> {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart {
        part: part.into(),
    })
}

pub fn clats_heredoc_close<'src>(symbol: Vec<u8>) -> ConcreteLineTokenAndTargets<'src> {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::HeredocClose { symbol })
}

pub fn clats_indent<'src>(depth: ColNumber) -> ConcreteLineTokenAndTargets<'src> {
    ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Indent { depth })
}

// represents something that will actually end up as a ruby token, as opposed to
// something that has to be transformed to become a ruby token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcreteLineToken<'src> {
    HardNewLine,
    Indent {
        depth: u32,
    },
    Keyword {
        keyword: &'static [u8],
    },
    DefKeyword,
    ClassKeyword,
    ModuleKeyword,
    DoKeyword,
    ConditionalKeyword {
        contents: &'static [u8],
    },
    DirectPart {
        part: Cow<'src, [u8]>,
    },
    MethodName {
        name: &'src [u8],
    },
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
    Op {
        op: &'src [u8],
    },
    DoubleQuote,
    LTStringContent {
        content: Cow<'src, [u8]>,
    },
    SingleSlash,
    Comment {
        contents: Cow<'src, [u8]>,
    },
    Delim {
        contents: &'static [u8],
    },
    End,
    HeredocClose {
        symbol: Vec<u8>,
    },
    // These are "magic" tokens. They have no concrete representation,
    // but they're meaningful inside of the render queue
    AfterCallChain,
    BeginCallChainIndent,
    EndCallChainIndent,
    HeredocStart {
        kind: HeredocKind,
        symbol: &'src [u8],
    },
    RawHeredocContent {
        content: Vec<u8>,
    },
}

impl<'src> ConcreteLineToken<'src> {
    pub fn into_ruby(self) -> Cow<'src, [u8]> {
        match self {
            Self::HardNewLine => Cow::Borrowed(b"\n"),
            Self::Indent { depth } => get_indent(depth as usize),
            Self::Keyword { keyword } => Cow::Borrowed(keyword),
            Self::ConditionalKeyword { contents } => Cow::Borrowed(contents),
            Self::DoKeyword => Cow::Borrowed(b"do"),
            Self::ClassKeyword => Cow::Borrowed(b"class"),
            Self::DefKeyword => Cow::Borrowed(b"def"),
            Self::ModuleKeyword => Cow::Borrowed(b"module"),
            Self::DirectPart { part } => part,
            Self::MethodName { name } => Cow::Borrowed(name),
            Self::CommaSpace => Cow::Borrowed(b", "),
            Self::Comma => Cow::Borrowed(b","),
            Self::Space => Cow::Borrowed(b" "),
            Self::Dot => Cow::Borrowed(b"."),
            Self::ColonColon => Cow::Borrowed(b"::"),
            Self::LonelyOperator => Cow::Borrowed(b"&."),
            Self::OpenSquareBracket => Cow::Borrowed(b"["),
            Self::CloseSquareBracket => Cow::Borrowed(b"]"),
            Self::OpenCurlyBracket => Cow::Borrowed(b"{"),
            Self::CloseCurlyBracket => Cow::Borrowed(b"}"),
            Self::OpenParen => Cow::Borrowed(b"("),
            Self::CloseParen => Cow::Borrowed(b")"),
            Self::Op { op } => Cow::Borrowed(op),
            Self::DoubleQuote => Cow::Borrowed(b"\""),
            Self::LTStringContent { content } => content,
            Self::SingleSlash => Cow::Borrowed(b"\\"),
            Self::Comment { contents } => contents,
            Self::Delim { contents } => Cow::Borrowed(contents),
            Self::End => Cow::Borrowed(b"end"),
            Self::HeredocClose { symbol } => Cow::Owned(symbol),
            Self::HeredocStart { symbol, .. } => Cow::Borrowed(symbol),
            Self::RawHeredocContent { content } => Cow::Owned(content),
            // no-op, this is purely semantic information
            // for the render queue
            Self::AfterCallChain | Self::BeginCallChainIndent | Self::EndCallChainIndent => {
                Cow::Borrowed(b"")
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
            HeredocStart { symbol, .. } => symbol.len(),
            Delim { contents } => contents.len(),
            Indent { depth } => *depth as usize,
            Keyword { keyword: contents } | ConditionalKeyword { contents } => contents.len(),
            Op { op } => op.len(),
            MethodName { name: op } => op.len(),
            DirectPart { part } => part.len(),
            LTStringContent { content } => content.len(),
            Comment { contents } => contents.len(),
            HeredocClose { symbol: contents } | RawHeredocContent { content: contents } => {
                contents.len()
            }

            HardNewLine | Comma | Space | Dot | OpenSquareBracket | CloseSquareBracket
            | OpenCurlyBracket | CloseCurlyBracket | OpenParen | CloseParen | SingleSlash
            | DoubleQuote => 1,
            DoKeyword | CommaSpace | LonelyOperator | ColonColon => 2,
            DefKeyword | End => 3, // "def"/"..."/"end"
            ClassKeyword => 5,     // "class"
            ModuleKeyword => 6,    // "module"
        }
    }

    fn is_block_closing_token(&self) -> bool {
        match self {
            Self::End => true,
            Self::DirectPart { part } => {
                part.as_ref() == b"}" || part.as_ref() == b"]" || part.as_ref() == b")"
            }
            Self::Delim { contents } => *contents == b"}" || *contents == b"]" || *contents == b")",
            _ => false,
        }
    }

    fn is_conditional_spaced_token(&self) -> bool {
        match self {
            Self::ConditionalKeyword { contents } => {
                !(*contents == b"else" || *contents == b"elsif")
            }
            Self::Dot | Self::LonelyOperator => false,
            Self::DirectPart { part } => part.as_ref() != b"&.",
            _ => true,
        }
    }

    pub fn is_single_line_breakable_garbage(&self) -> bool {
        match self {
            Self::DirectPart { part } => part.is_empty(),
            Self::Space => true,
            _ => false,
        }
    }

    pub fn is_newline(&self) -> bool {
        match self {
            Self::HardNewLine => true,
            Self::DirectPart { part } if part.as_ref() == b"\n" => {
                panic!("shouldn't ever have a single newline direct part");
            }
            _ => false,
        }
    }

    pub fn is_indent(&self) -> bool {
        matches!(self, ConcreteLineToken::Indent { .. })
    }

    pub fn is_in_need_of_a_trailing_blankline(&self) -> bool {
        self.is_conditional_spaced_token() && !self.is_block_closing_token()
    }
}

impl<'src> From<ConcreteLineToken<'src>> for ConcreteLineTokenAndTargets<'src> {
    fn from(clt: ConcreteLineToken<'src>) -> ConcreteLineTokenAndTargets<'src> {
        ConcreteLineTokenAndTargets::ConcreteLineToken(clt)
    }
}

impl<'src> From<ConcreteLineTokenAndTargets<'src>> for AbstractLineToken<'src> {
    fn from(cltat: ConcreteLineTokenAndTargets<'src>) -> AbstractLineToken<'src> {
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
            ConcreteLineTokenAndTargets::ConditionalLayoutEntry(cle) => {
                AbstractLineToken::ConditionalLayoutEntry(cle)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConcreteLineTokenAndTargets<'src> {
    ConcreteLineToken(ConcreteLineToken<'src>),
    BreakableEntry(BreakableEntry<'src>),
    BreakableCallChainEntry(BreakableCallChainEntry<'src>),
    ConditionalLayoutEntry(ConditionalLayoutEntry<'src>),
}

impl ConcreteLineTokenAndTargets<'_> {
    pub fn is_newline(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_newline(),
            _ => false,
        }
    }

    pub fn is_indent(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_indent(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AbstractLineToken<'src> {
    // this is all bodil's fault
    ConcreteLineToken(ConcreteLineToken<'src>),
    CollapsingNewLine(Option<Vec<HeredocString<'src>>>),
    SoftNewline(Option<Vec<HeredocString<'src>>>),
    SoftIndent { depth: u32 },
    BreakableEntry(BreakableEntry<'src>),
    BreakableCallChainEntry(BreakableCallChainEntry<'src>),
    ConditionalLayoutEntry(ConditionalLayoutEntry<'src>),
}

impl<'src> AbstractLineToken<'src> {
    pub fn write_single_line(self, out: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        match self {
            Self::CollapsingNewLine(heredoc_strings) => {
                if heredoc_strings.is_some() {
                    out.push(cltats_hard_newline());
                }
                Self::write_heredocs(heredoc_strings, out);
            }
            Self::SoftNewline(heredoc_strings) => {
                out.push(ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::Space,
                ));
                Self::write_heredocs(heredoc_strings, out);
            }
            Self::SoftIndent { .. } => {}
            Self::ConcreteLineToken(clt) => {
                out.push(ConcreteLineTokenAndTargets::ConcreteLineToken(clt));
            }
            Self::BreakableEntry(be) => {
                out.push(ConcreteLineTokenAndTargets::BreakableEntry(be));
            }
            Self::BreakableCallChainEntry(bcce) => {
                out.push(ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce));
            }
            Self::ConditionalLayoutEntry(cle) => {
                out.push(ConcreteLineTokenAndTargets::ConditionalLayoutEntry(cle))
            }
        }
    }

    pub fn write_multi_line(self, out: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        match self {
            Self::CollapsingNewLine(heredoc_strings) => {
                out.push(cltats_hard_newline());
                Self::write_heredocs(heredoc_strings, out);
            }
            Self::SoftNewline(heredoc_strings) => {
                out.push(cltats_hard_newline());
                Self::write_heredocs(heredoc_strings, out);
            }
            Self::SoftIndent { depth } => {
                out.push(ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::Indent { depth },
                ));
            }
            Self::ConcreteLineToken(clt) => {
                out.push(ConcreteLineTokenAndTargets::ConcreteLineToken(clt));
            }
            Self::BreakableEntry(be) => {
                out.push(ConcreteLineTokenAndTargets::BreakableEntry(be));
            }
            Self::BreakableCallChainEntry(bcce) => {
                out.push(ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce));
            }
            Self::ConditionalLayoutEntry(cle) => {
                out.push(ConcreteLineTokenAndTargets::ConditionalLayoutEntry(cle))
            }
        }
    }

    fn write_heredocs(
        heredoc_strings: Option<Vec<HeredocString>>,
        out: &mut Vec<ConcreteLineTokenAndTargets<'src>>,
    ) {
        if let Some(values) = heredoc_strings {
            for hds in values {
                let indent = hds.indent;
                let kind = hds.kind;
                let symbol = hds.closing_symbol();

                let s = hds.render_as_bytes();
                if !s.is_empty() {
                    out.push(clats_direct_part(s));
                    out.push(cltats_hard_newline());
                }
                if !kind.is_bare() {
                    out.push(clats_indent(indent));
                }
                out.push(clats_heredoc_close(symbol));
                out.push(cltats_hard_newline());
                let indent_depth = if indent != 0 { indent - 2 } else { indent };
                out.push(clats_indent(indent_depth));
            }
        }
    }

    pub fn is_indent(&self) -> bool {
        match self {
            Self::ConcreteLineToken(clt) => clt.is_indent(),
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

    /// Returns the length of this token when rendered as single-line,
    /// without cloning or allocating intermediate structures. This assumes
    /// that its caller is purely checking if the line is over MAX_LINE_LENGTH,
    /// so heredocs are not actually calculated and instead just return `MAX_LINE_LENGTH` + 1
    pub fn single_line_len(&self) -> usize {
        use crate::render_queue_writer::MAX_LINE_LENGTH;

        match self {
            Self::CollapsingNewLine(heredoc_strings) => {
                if heredoc_strings.is_some() {
                    MAX_LINE_LENGTH + 1
                } else {
                    0
                }
            }
            Self::SoftNewline(heredoc_strings) => {
                if heredoc_strings.is_some() {
                    MAX_LINE_LENGTH + 1
                } else {
                    1
                }
            }
            Self::SoftIndent { .. } => 0,
            Self::ConcreteLineToken(clt) => clt.len(),
            Self::BreakableEntry(be) => be.single_line_len(),
            Self::BreakableCallChainEntry(bcce) => bcce.single_line_len(),
            Self::ConditionalLayoutEntry(cle) => cle.inline_single_line_len(),
        }
    }
}
