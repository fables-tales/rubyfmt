use crate::line_tokens::ConcreteLineToken;

#[derive(Debug, Clone, Eq, PartialEq)]
struct DelimiterPair {
    open: &'static [u8],
    close: &'static [u8],
}

impl DelimiterPair {
    fn new(open: &'static [u8], close: &'static [u8]) -> Self {
        DelimiterPair { open, close }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakableDelims {
    single_line: DelimiterPair,
    multi_line: DelimiterPair,
}

impl BreakableDelims {
    pub fn for_method_call() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"(", b")"),
            multi_line: DelimiterPair::new(b"(", b")"),
        }
    }

    pub fn for_return_kw() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b" ", b""),
            multi_line: DelimiterPair::new(b" [", b"]"),
        }
    }

    pub fn for_kw() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b" ", b""),
            multi_line: DelimiterPair::new(b"(", b")"),
        }
    }

    pub fn for_block_params() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b" |", b"|"),
            multi_line: DelimiterPair::new(b" |", b"|"),
        }
    }

    pub fn for_array() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"[", b"]"),
            multi_line: DelimiterPair::new(b"[", b"]"),
        }
    }

    pub fn for_when() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b" ", b""),
            multi_line: DelimiterPair::new(b"", b""),
        }
    }

    pub fn for_hash() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"{", b"}"),
            multi_line: DelimiterPair::new(b"{", b"}"),
        }
    }

    pub fn for_brace_block() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"{", b" }"),
            multi_line: DelimiterPair::new(b"{", b"}"),
        }
    }

    pub fn for_binary_op() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"", b""),
            multi_line: DelimiterPair::new(b"", b""),
        }
    }

    pub fn for_parens() -> Self {
        BreakableDelims {
            single_line: DelimiterPair::new(b"(", b")"),
            multi_line: DelimiterPair::new(b"(", b")"),
        }
    }

    pub fn single_line_open<'src>(&self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.single_line.open,
        }
    }

    pub fn single_line_close<'src>(&self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.single_line.close,
        }
    }

    pub fn multi_line_open<'src>(&self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.multi_line.open,
        }
    }

    pub fn multi_line_close<'src>(&self) -> ConcreteLineToken<'src> {
        ConcreteLineToken::Delim {
            contents: self.multi_line.close,
        }
    }

    pub fn single_line_len(&self) -> usize {
        self.single_line.open.len() + self.single_line.close.len()
    }
}
