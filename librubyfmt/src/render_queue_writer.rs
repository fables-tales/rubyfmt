use crate::intermediary::{BlanklineReason, Intermediary};
use crate::line_tokens::*;
use crate::render_targets::{AbstractTokenTarget, BreakableEntry, ConvertType};
#[cfg(debug_assertions)]
use log::debug;
use std::io::{self, Write};

const MAX_LINE_LENGTH: usize = 120;

pub struct RenderQueueWriter {
    tokens: Vec<ConcreteLineTokenAndTargets>,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<ConcreteLineTokenAndTargets>) -> Self {
        RenderQueueWriter { tokens }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let mut accum = Intermediary::new();
        #[cfg(debug_assertions)]
        {
            debug!("first tokens {:?}", self.tokens);
        }
        Self::render_as(&mut accum, self.tokens);
        Self::write_final_tokens(writer, accum.into_tokens())
    }

    fn render_as(accum: &mut Intermediary, tokens: Vec<ConcreteLineTokenAndTargets>) {
        for next_token in tokens.into_iter() {
            match next_token {
                ConcreteLineTokenAndTargets::BreakableEntry(be) => {
                    Self::format_breakable_entry(accum, be)
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(x) => accum.push(x),
            }

            if accum.len() >= 4 {
                if let (
                    &ConcreteLineToken::HeredocClose { .. },
                    &ConcreteLineToken::HardNewLine,
                    &ConcreteLineToken::Indent { .. },
                    &ConcreteLineToken::HardNewLine,
                ) = accum.last_4().expect("we checked length")
                {
                    accum.pop_heredoc_mistake();
                }

                if let (
                    &ConcreteLineToken::End,
                    &ConcreteLineToken::HardNewLine,
                    &ConcreteLineToken::Indent { .. },
                    x,
                ) = accum.last_4().expect("we checked length")
                {
                    if x.is_in_need_of_a_trailing_blankline() {
                        accum.insert_trailing_blankline(BlanklineReason::ComesAfterEnd);
                    }
                }
            }

            if accum.len() >= 5 {
                if let (
                    &ConcreteLineToken::End,
                    &ConcreteLineToken::AfterCallChain,
                    &ConcreteLineToken::HardNewLine,
                    &ConcreteLineToken::Indent { .. },
                    x,
                ) = accum.last_5().expect("we checked length")
                {
                    match x {
                        ConcreteLineToken::DefKeyword => {}
                        _ => {
                            if x.is_in_need_of_a_trailing_blankline() {
                                accum.insert_trailing_blankline(BlanklineReason::ComesAfterEnd);
                            }
                        }
                    }
                }
            }
        }
    }

    fn format_breakable_entry(accum: &mut Intermediary, be: BreakableEntry) {
        let length = be.single_line_string_length();

        if length > MAX_LINE_LENGTH || be.is_multiline() {
            Self::render_as(accum, be.into_tokens(ConvertType::MultiLine));
        } else {
            Self::render_as(accum, be.into_tokens(ConvertType::SingleLine));
            // after running accum looks like this (or some variant):
            // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
            // so we remove items at positions length-2 until there is nothing
            // in that position that is garbage.
            accum.clear_breakable_garbage();
        }
    }

    fn write_final_tokens<W: Write>(
        writer: &mut W,
        mut tokens: Vec<ConcreteLineToken>,
    ) -> io::Result<()> {
        #[cfg(debug_assertions)]
        {
            debug!("final tokens: {:?}", tokens);
        }

        let len = tokens.len();
        if len > 2 {
            let delete = matches!(
                (tokens.get(len - 2), tokens.get(len - 1)),
                (
                    Some(ConcreteLineToken::HardNewLine),
                    Some(ConcreteLineToken::HardNewLine)
                )
            );
            if delete {
                tokens.pop();
            }
        }

        for line_token in tokens.into_iter() {
            let s = line_token.into_ruby();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
