use crate::line_metadata::LineMetadata;
use crate::line_tokens::*;
use std::mem;

pub struct Intermediary {
    tokens: Vec<LineToken>,
    index_of_last_hard_newline: usize,
    current_line_metadata: LineMetadata,
    previous_line_metadata: Option<LineMetadata>,
}

impl Intermediary {
    pub fn new() -> Self {
        Intermediary {
            tokens: vec![],
            current_line_metadata: LineMetadata::new(),
            previous_line_metadata: None,
            index_of_last_hard_newline: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn last_4(&self) -> Option<(&LineToken, &LineToken, &LineToken, &LineToken)> {
        if self.len() < 4 {
            return None;
        }

        Some((
            &self.tokens[self.len() - 4],
            &self.tokens[self.len() - 3],
            &self.tokens[self.len() - 2],
            &self.tokens[self.len() - 1],
        ))
    }

    pub fn into_tokens(self) -> Vec<LineToken> {
        self.tokens
    }

    pub fn push(&mut self, lt: LineToken) {
        self.debug_assert_newlines();
        match lt {
            LineToken::ModuleKeyword | LineToken::ClassKeyword => {
                self.handle_class_or_module();
            }
            LineToken::HardNewLine => {
                let mut md = LineMetadata::new();
                mem::swap(&mut md, &mut self.current_line_metadata);
                self.previous_line_metadata = Some(md);
                self.index_of_last_hard_newline = self.tokens.len();
            }
            _ => {}
        }
        self.tokens.push(lt);
        self.debug_assert_newlines();
    }

    fn handle_class_or_module(&mut self) {
        self.current_line_metadata.set_defines_class_or_module();
        if let Some(prev) = &self.previous_line_metadata {
            if !prev.has_class_or_module_definition() {
                self.insert_trailing_blankline();
            }
        }
    }

    pub fn clear_breakable_garbage(&mut self) {
        // after running self.tokens looks like this (or some variant):
        // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
        // so we remove items at positions length-2 until there is nothing
        // in that position that is garbage.
        while self.tokens[self.len() - 2].is_single_line_breakable_garbage() {
            self.tokens.remove(self.len() - 2);
        }
    }

    pub fn insert_trailing_blankline(&mut self) {
        match (
            self.tokens.get(self.index_of_last_hard_newline - 1),
            self.tokens.get(self.index_of_last_hard_newline),
        ) {
            (Some(&LineToken::HardNewLine), Some(&LineToken::HardNewLine)) => {}
            _ => {
                self.tokens
                    .insert(self.index_of_last_hard_newline, LineToken::HardNewLine);
                self.index_of_last_hard_newline += 1;
                self.debug_assert_newlines();
            }
        }
    }

    #[cfg(debug_assertions)]
    fn debug_assert_newlines(&self) {
        if self.index_of_last_hard_newline == 0 {
            return;
        }
        match self.tokens.get(self.index_of_last_hard_newline) {
            Some(&LineToken::HardNewLine) => {}
            _ => panic!("newlines are fucked"),
        }
    }

    #[cfg(not(debug_assertions))]
    fn debug_assert_newlines(&self) {}
}
