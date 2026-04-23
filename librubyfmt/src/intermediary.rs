use crate::line_metadata::LineMetadata;
use crate::line_tokens::*;
#[cfg(debug_assertions)]
use log::debug;
use std::mem;

#[derive(Debug)]
pub enum BlanklineReason {
    ComesAfterEnd,
    ClassOrModule,
    EndOfRequireBlock,
}

pub struct Intermediary<'src> {
    tokens: Vec<ConcreteLineToken<'src>>,
    index_of_last_hard_newline: usize,
    current_line_metadata: LineMetadata,
    previous_line_metadata: Option<LineMetadata>,
    pub additional_indent: u32,
}

impl<'src> Intermediary<'src> {
    pub fn new() -> Self {
        Intermediary {
            tokens: vec![],
            current_line_metadata: LineMetadata::new(),
            previous_line_metadata: None,
            index_of_last_hard_newline: 0,
            additional_indent: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn fix_heredoc_duplicate_indent_mistake(&mut self) {
        // Remove duplicate indent
        self.tokens.remove(self.tokens.len() - 3);
    }

    pub fn last<const N: usize>(&self) -> Option<&[ConcreteLineToken<'src>; N]> {
        self.tokens.last_chunk::<N>()
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

        match &lt {
            ConcreteLineToken::HardNewLine => {
                if let Some(prev) = &self.previous_line_metadata
                    && !self.current_line_metadata.has_require()
                    && prev.has_require()
                {
                    self.insert_trailing_blankline(BlanklineReason::EndOfRequireBlock);
                }

                let mut md = LineMetadata::new();
                mem::swap(&mut md, &mut self.current_line_metadata);
                self.previous_line_metadata = Some(md);
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
            ConcreteLineToken::ModuleKeyword | ConcreteLineToken::ClassKeyword => {
                self.handle_class_or_module();
            }
            ConcreteLineToken::Indent { depth } => {
                self.current_line_metadata.observe_indent_level(*depth);

                if let Some(prev) = &mut self.previous_line_metadata
                    && LineMetadata::indent_level_increases_between(
                        prev,
                        &self.current_line_metadata,
                    )
                {
                    prev.set_gets_indented()
                }
            }
            ConcreteLineToken::MethodName { name }
                if *name == b"require" && self.tokens.last().is_some_and(|t| t.is_indent()) =>
            {
                self.current_line_metadata.set_has_require();
            }
            _ => {}
        }

        if do_push {
            self.tokens.push(lt);
        }
        self.debug_assert_newlines();
    }

    fn handle_class_or_module(&mut self) {
        if let Some(prev) = &self.previous_line_metadata
            && !prev.gets_indented()
        {
            self.insert_trailing_blankline(BlanklineReason::ClassOrModule);
        }
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

    pub fn insert_blankline_from_end(&mut self, index_from_end: usize) {
        let insert_position = self.tokens.len() - index_from_end;
        self.tokens
            .insert(insert_position, ConcreteLineToken::HardNewLine);

        // If the insertion was at or before it, we'll have moved the last
        // hard newline up by one place
        if insert_position <= self.index_of_last_hard_newline {
            self.index_of_last_hard_newline += 1;
        }

        self.debug_assert_newlines();
    }

    pub fn insert_trailing_blankline(&mut self, _bl: BlanklineReason) {
        if self.index_of_last_hard_newline <= 2 {
            self.tokens.insert(
                self.index_of_last_hard_newline,
                ConcreteLineToken::HardNewLine,
            );
            self.index_of_last_hard_newline += 1;
            self.debug_assert_newlines();
            return;
        }
        match (
            self.tokens.get(self.index_of_last_hard_newline - 2),
            self.tokens.get(self.index_of_last_hard_newline - 1),
            self.tokens.get(self.index_of_last_hard_newline),
        ) {
            (
                Some(&ConcreteLineToken::HardNewLine),
                Some(&ConcreteLineToken::Indent { .. }),
                Some(&ConcreteLineToken::HardNewLine),
            ) => {}
            (_, Some(&ConcreteLineToken::HardNewLine), Some(&ConcreteLineToken::HardNewLine)) => {}
            (_, _, _) => {
                #[cfg(debug_assertions)]
                {
                    debug!("{:?}", _bl);
                }
                self.tokens.insert(
                    self.index_of_last_hard_newline,
                    ConcreteLineToken::HardNewLine,
                );
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
            Some(&ConcreteLineToken::HardNewLine) => {}
            _ => panic!("newlines are fucked"),
        }
    }

    #[cfg(not(debug_assertions))]
    fn debug_assert_newlines(&self) {}
}
