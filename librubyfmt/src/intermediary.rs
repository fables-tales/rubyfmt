use crate::line_tokens::*;

pub struct Intermediary<'src> {
    tokens: Vec<ConcreteLineToken<'src>>,
    index_of_last_hard_newline: usize,
    pub additional_indent: u32,
}

impl<'src> Intermediary<'src> {
    pub fn new() -> Self {
        Intermediary {
            tokens: vec![],
            index_of_last_hard_newline: 0,
            additional_indent: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn into_tokens(self) -> Vec<ConcreteLineToken<'src>> {
        self.tokens
    }

    pub fn current_line_length(&self) -> usize {
        if self.tokens.is_empty() {
            return 0;
        }

        let line_start_index = self.index_of_last_hard_newline + 1;
        let tokens_on_current_line = &self.tokens[line_start_index..];

        tokens_on_current_line.iter().map(|t| t.len()).sum()
    }

    pub fn push(&mut self, lt: ConcreteLineToken<'src>) {
        self.debug_assert_newlines();
        let mut do_push = true;

        if matches!(lt, ConcreteLineToken::HardNewLine) {
            self.index_of_last_hard_newline = self.tokens.len();

            if self.tokens.len() >= 2
                && let (
                    Some(&ConcreteLineToken::HardNewLine),
                    Some(&ConcreteLineToken::HardNewLine),
                ) = (
                    self.tokens.get(self.index_of_last_hard_newline - 2),
                    self.tokens.get(self.index_of_last_hard_newline - 1),
                )
            {
                do_push = false;
                self.index_of_last_hard_newline = self.tokens.len() - 1;
            }
        }

        if do_push {
            self.tokens.push(lt);
        }
        self.debug_assert_newlines();
    }

    pub fn clear_breakable_garbage(&mut self) {
        // after running self.tokens looks like this (or some variant):
        // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
        // so we remove items at positions length-2 until there is nothing
        // in that position that is garbage.
        while self.tokens.len() > 2
            && self.tokens[self.len() - 2].is_single_line_breakable_garbage()
        {
            self.tokens.remove(self.len() - 2);
        }
    }

    #[cfg(debug_assertions)]
    fn debug_assert_newlines(&self) {
        if self.index_of_last_hard_newline == 0 {
            return;
        }
        match self.tokens.get(self.index_of_last_hard_newline) {
            Some(&ConcreteLineToken::HardNewLine) => {}
            _ => panic!("newlines are fucked"),
        }
    }

    #[cfg(not(debug_assertions))]
    fn debug_assert_newlines(&self) {}
}
