use crate::delimiters::BreakableDelims;
use crate::line_tokens::{AbstractLineToken, ConcreteLineTokenAndTargets};
use crate::types::{ColNumber, LineNumber};
use std::collections::HashSet;

fn insert_at<T>(idx: usize, target: &mut Vec<T>, input: &mut Vec<T>) {
    let mut tail = target.split_off(idx);
    target.append(input);
    target.append(&mut tail);
}

#[derive(Copy, Clone, Debug)]
pub enum ConvertType {
    MultiLine,
    SingleLine,
}

#[derive(Debug, Default, Clone)]
pub struct BaseQueue {
    tokens: Vec<ConcreteLineTokenAndTargets>,
}

impl BaseQueue {
    pub fn push(&mut self, lt: ConcreteLineTokenAndTargets) {
        self.tokens.push(lt)
    }

    pub fn insert_at(&mut self, idx: usize, tokens: &mut Vec<ConcreteLineTokenAndTargets>) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn into_tokens(self) -> Vec<ConcreteLineTokenAndTargets> {
        self.tokens
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.tokens.last().map(|x| x.is_newline()).unwrap_or(false)
    }

    pub fn index_of_prev_hard_newline(&self) -> Option<usize> {
        self.tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment())
    }
}

#[derive(Debug, Clone)]
pub struct BreakableEntry {
    spaces: ColNumber,
    tokens: Vec<AbstractLineToken>,
    line_numbers: HashSet<LineNumber>,
    delims: BreakableDelims,
}

impl BreakableEntry {
    pub fn new(spaces: ColNumber, delims: BreakableDelims) -> Self {
        BreakableEntry {
            spaces,
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            delims,
        }
    }

    pub fn push(&mut self, lt: AbstractLineToken) {
        self.tokens.push(lt);
    }

    pub fn insert_at(&mut self, idx: usize, tokens: &mut Vec<AbstractLineToken>) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn into_tokens(self, ct: ConvertType) -> Vec<ConcreteLineTokenAndTargets> {
        match ct {
            ConvertType::MultiLine => {
                let mut new_tokens: Vec<_> = self
                    .tokens
                    .into_iter()
                    .map(|t| t.into_multi_line())
                    .collect();
                new_tokens.insert(0, self.delims.multi_line_open().into());
                new_tokens.push(self.delims.multi_line_close().into());
                new_tokens
            }
            ConvertType::SingleLine => {
                let mut new_tokens: Vec<_> = self
                    .tokens
                    .into_iter()
                    .map(|t| t.into_single_line())
                    .collect();
                new_tokens.insert(0, self.delims.single_line_open().into());
                new_tokens.push(self.delims.single_line_close().into());
                new_tokens
            }
        }
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        match self.tokens.last() {
            Some(x) => x.is_newline(),
            _ => false,
        }
    }

    pub fn index_of_prev_hard_newline(&self) -> Option<usize> {
        self.tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment())
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
}
