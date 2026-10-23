use crate::line_tokens::ConcreteLineToken;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DelimiterKind {
    MethodCall,
    ReturnKw,
    Kw,
    BlockParams,
    Array,
    When,
    Hash,
    BraceBlock,
    BinaryOp,
    Parens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakableDelims {
    kind: DelimiterKind,
}

impl BreakableDelims {
    const fn new(kind: DelimiterKind) -> Self {
        BreakableDelims { kind }
    }

    pub fn for_method_call() -> Self {
        Self::new(DelimiterKind::MethodCall)
    }

    pub fn for_return_kw() -> Self {
        Self::new(DelimiterKind::ReturnKw)
    }

    pub fn for_kw() -> Self {
        Self::new(DelimiterKind::Kw)
    }

    pub fn for_block_params() -> Self {
        Self::new(DelimiterKind::BlockParams)
    }

    pub fn for_array() -> Self {
        Self::new(DelimiterKind::Array)
    }

    pub fn for_when() -> Self {
        Self::new(DelimiterKind::When)
    }

    pub fn for_hash() -> Self {
        Self::new(DelimiterKind::Hash)
    }

    pub fn for_brace_block() -> Self {
        Self::new(DelimiterKind::BraceBlock)
    }

    pub fn for_binary_op() -> Self {
        Self::new(DelimiterKind::BinaryOp)
    }

    pub fn for_parens() -> Self {
        Self::new(DelimiterKind::Parens)
    }

    fn single_line_pair(self) -> (&'static [u8], &'static [u8]) {
        match self.kind {
            DelimiterKind::MethodCall => (b"(", b")"),
            DelimiterKind::ReturnKw => (b" ", b""),
            DelimiterKind::Kw => (b" ", b""),
            DelimiterKind::BlockParams => (b" |", b"|"),
            DelimiterKind::Array => (b"[", b"]"),
            DelimiterKind::When => (b" ", b""),
            DelimiterKind::Hash => (b"{", b"}"),
            DelimiterKind::BraceBlock => (b"{", b" }"),
            DelimiterKind::BinaryOp => (b"", b""),
            DelimiterKind::Parens => (b"(", b")"),
        }
    }

    fn multi_line_pair(self) -> (&'static [u8], &'static [u8]) {
        match self.kind {
            DelimiterKind::MethodCall => (b"(", b")"),
            DelimiterKind::ReturnKw => (b" [", b"]"),
            DelimiterKind::Kw => (b"(", b")"),
            DelimiterKind::BlockParams => (b" |", b"|"),
            DelimiterKind::Array => (b"[", b"]"),
            DelimiterKind::When => (b"", b""),
            DelimiterKind::Hash => (b"{", b"}"),
            DelimiterKind::BraceBlock => (b"{", b"}"),
            DelimiterKind::BinaryOp => (b"", b""),
            DelimiterKind::Parens => (b"(", b")"),
        }
    }

    pub fn single_line_open<'src>(self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.single_line_pair().0,
        }
    }

    pub fn single_line_close<'src>(self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.single_line_pair().1,
        }
    }

    pub fn multi_line_open<'src>(self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.multi_line_pair().0,
        }
    }

    pub fn multi_line_close<'src>(self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.multi_line_pair().1,
        }
    }

    pub fn single_line_len(self) -> usize {
        let (open, close) = self.single_line_pair();
        open.len() + close.len()
    }
}
