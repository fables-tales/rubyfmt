use crate::line_tokens::*;

pub struct Intermediary {
    tokens: Vec<LineToken>,
}

impl Intermediary {
    pub fn new() -> Self {
        Intermediary { tokens: vec![] }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn last_4(&self) -> Option<(&LineToken, &LineToken, &LineToken, &LineToken)> {
        if self.len() < 4 {
            return None
        }

        Some((
            &self.tokens[self.len()-4],
            &self.tokens[self.len()-3],
            &self.tokens[self.len()-2],
            &self.tokens[self.len()-1],
        ))
    }

    pub fn to_tokens(self) -> Vec<LineToken> {
        self.tokens
    }

    pub fn insert(&mut self, idx: usize, lt: LineToken) {
        self.tokens.insert(idx, lt);
    }

    pub fn push(&mut self, lt: LineToken) {
        self.tokens.push(lt);
    }

    pub fn clear_breakable_garbage(&mut self) {
        // after running self.tokens looks like this (or some variant):
        // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
        // so we remove items at positions length-2 until there is nothing
        // in that position that is garbage.
        while self.tokens[self.len()-2].is_single_line_breakable_garbage() {
            self.tokens.remove(self.len()-2);
        }
    }
}
