use crate::intermediary::{BlanklineReason, Intermediary};
use crate::line_tokens::*;
use crate::parser_state::FormattingContext;
use crate::render_targets::{AbstractTokenTarget, BreakableEntry, ConvertType};
#[cfg(debug_assertions)]
use log::debug;
use std::io::{self, Write};

pub const MAX_LINE_LENGTH: usize = 120;

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
        use ConcreteLineToken::*;

        for next_token in tokens.into_iter() {
            match next_token {
                ConcreteLineTokenAndTargets::BreakableEntry(be) => {
                    Self::format_breakable_entry(accum, be)
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(x) => accum.push(x),
            }

            if let Some(
                [&ConcreteLineToken::HeredocClose { .. }, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::HardNewLine],
            ) = accum.last::<4>()
            {
                accum.pop_heredoc_mistake();
            }

            if let Some(
                [&ConcreteLineToken::End, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, x],
            ) = accum.last::<4>()
            {
                if x.is_in_need_of_a_trailing_blankline() {
                    accum.insert_trailing_blankline(BlanklineReason::ComesAfterEnd);
                }
            }

            if let Some([HardNewLine, HardNewLine, Comment { contents }, HardNewLine]) =
                accum.last::<4>()
            {
                if contents.is_empty() {
                    accum.pop_require_comment_whitespace();
                }
            }

            if let Some(
                [&ConcreteLineToken::End, &ConcreteLineToken::AfterCallChain, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, x, maybe_space, maybe_def],
            ) = accum.last::<7>()
            {
                match x {
                    ConcreteLineToken::DefKeyword => {}
                    _ => {
                        if x.is_in_need_of_a_trailing_blankline()
                            && !matches!(
                                (maybe_space, maybe_def),
                                (ConcreteLineToken::Space, ConcreteLineToken::DefKeyword)
                            )
                        {
                            // If we're here, the last few tokens must look like this:
                            // | token             | index_from_end |
                            // |  End              | 6              |
                            // |. AfterCallChain   | 5              |
                            // |  HardNewline      | 4              | <-- insert after this token
                            // |. Indent           | 3 .            |
                            // |  (ArbitraryToken) | 2              |
                            // |  (ArbitraryToken) | 1              |
                            // |  (ArbitraryToken) | 0              |
                            const LAST_NEWLINE_INDEX_FROM_END: usize = 4;
                            accum.insert_blankline_from_end(LAST_NEWLINE_INDEX_FROM_END);
                        }
                    }
                }
            }

            if let Some(
                [&ConcreteLineToken::HeredocClose { .. }, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Delim { .. }],
            ) = accum.last::<5>()
            {
                accum.fix_heredoc_delim_indent_mistake();
            }

            if let Some(
                [&ConcreteLineToken::HeredocClose { .. }, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::DirectPart { .. }],
            ) = accum.last::<5>()
            {
                accum.fix_heredoc_direct_part_indent_mistake();
            }

            if let Some(
                [&ConcreteLineToken::HeredocClose { .. }, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Delim { .. }, &ConcreteLineToken::Comma, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::HardNewLine],
            ) = accum.last::<7>()
            {
                accum.fix_heredoc_arg_newline_mistake();
            }
        }
    }

    fn format_breakable_entry(accum: &mut Intermediary, be: BreakableEntry) {
        let length = accum.current_line_length() + be.single_line_string_length();

        if (length > MAX_LINE_LENGTH || be.is_multiline())
            && be.entry_formatting_context() != FormattingContext::StringEmbexpr
        {
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

        loop {
            let len = tokens.len();
            if len < 2 {
                break;
            }

            let delete = match (tokens.get(len - 2), tokens.get(len - 1)) {
                (
                    Some(ConcreteLineToken::Comment { contents }),
                    Some(ConcreteLineToken::HardNewLine),
                ) => contents.is_empty(),
                (Some(ConcreteLineToken::HardNewLine), Some(ConcreteLineToken::HardNewLine)) => {
                    true
                }
                _ => false,
            };

            if delete {
                tokens.pop();
            } else {
                break;
            }
        }

        for line_token in tokens.into_iter() {
            let s = line_token.into_ruby();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
