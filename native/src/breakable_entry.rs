use crate::delimiters::BreakableDelims;
use crate::line_tokens::LineToken;
use crate::types::{ColNumber, LineNumber};
use std::collections::HashSet;

pub enum ConvertType {
    MultiLine,
    SingleLine,
}

#[derive(Debug, Clone)]
pub struct BreakableEntry {
    spaces: ColNumber,
    tokens: Vec<LineToken>,
    line_numbers: HashSet<LineNumber>,
    delims: BreakableDelims,
}

impl BreakableEntry {
    pub fn new(spaces: ColNumber, delims: BreakableDelims) -> Self {
        BreakableEntry {
            spaces,
            tokens: vec![],
            line_numbers: HashSet::new(),
            delims,
        }
    }

    pub fn push(&mut self, lt: LineToken) {
        self.tokens.push(lt);
    }

    pub fn into_tokens(self, ct: ConvertType) -> Vec<LineToken> {
        let mut tokens = self.tokens;
        match ct {
            ConvertType::MultiLine => {
                tokens = tokens.into_iter().map(|t| t.into_multi_line()).collect();
                tokens.insert(0, self.delims.multi_line_open());
                tokens.push(self.delims.multi_line_close());
            }
            ConvertType::SingleLine => {
                tokens = tokens.into_iter().map(|t| t.into_single_line()).collect();
                tokens.insert(0, self.delims.single_line_open());
                tokens.push(self.delims.single_line_close());
            }
        }
        tokens
    }

    pub fn single_line_string_length(&self) -> usize {
        self.tokens
            .iter()
            .map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby().len())
            .sum()
    }

    pub fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    pub fn is_multiline(&self) -> bool {
        self.line_numbers.len() > 1
    }

    pub fn last_token_is_a_hard_newline(&self) -> bool {
        match self.tokens.last() {
            Some(LineToken::HardNewLine) => true,
            _ => false,
        }
    }
}
