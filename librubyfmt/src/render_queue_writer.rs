use crate::heredoc_string::HeredocKind;
use crate::intermediary::{BlanklineReason, Intermediary};
use crate::line_tokens::*;
use crate::render_targets::{
    AbstractTokenTarget, BreakableCallChainEntry, BreakableEntry, ConvertType,
};
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
        let token_len = tokens.len();
        let mut peekable = tokens.into_iter().enumerate().peekable();
        let mut current_heredoc_kind: Option<HeredocKind> = None;

        while let Some((index, mut next_token)) = peekable.next() {
            // Do any additional indentation changes caused by call chain rendering
            match &next_token {
                ConcreteLineTokenAndTargets::ConcreteLineToken(Indent { depth }) => {
                    let is_ending_heredoc_token = token_len > index
                        && matches!(
                            peekable.peek(),
                            Some((
                                _,
                                ConcreteLineTokenAndTargets::ConcreteLineToken(HeredocClose { .. })
                            ))
                        );
                    if !is_ending_heredoc_token {
                        next_token = clats_indent(depth + (accum.additional_indent * 2))
                    }
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Comment {
                    contents,
                }) => {
                    if !contents.is_empty() {
                        let mut new_contents: String =
                            (0..(accum.additional_indent * 2)).map(|_| ' ').collect();
                        new_contents.push_str(contents.as_str());
                        next_token = ConcreteLineTokenAndTargets::ConcreteLineToken(
                            ConcreteLineToken::Comment {
                                contents: new_contents,
                            },
                        )
                    }
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart {
                    part,
                }) => {
                    if current_heredoc_kind
                        .map(|k| k.is_squiggly())
                        .unwrap_or(false)
                    {
                        let indent: String =
                            (0..(accum.additional_indent * 2)).map(|_| ' ').collect();
                        let new_contents = part
                            .split('\n')
                            .map(|p| {
                                if p.is_empty() {
                                    return p.to_string();
                                }
                                let mut line = indent.clone();
                                line.push_str(p);
                                line
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        next_token = clats_direct_part(new_contents)
                    }
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::HeredocStart { kind, .. },
                ) => current_heredoc_kind = Some(*kind),
                ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::HeredocClose { symbol },
                ) => {
                    // Bare heredocs (e.g. <<FOO) must have the closing ident completely unindented, so
                    // ignore them in this case
                    if current_heredoc_kind.map(|k| !k.is_bare()).unwrap_or(false) {
                        let mut new_contents: String =
                            (0..(accum.additional_indent * 2)).map(|_| ' ').collect();
                        new_contents.push_str(symbol.as_str());
                        next_token = clats_heredoc_close(new_contents);
                    }
                    current_heredoc_kind = None;
                }
                _ => {}
            }

            match next_token {
                ConcreteLineTokenAndTargets::BreakableEntry(be) => {
                    Self::format_breakable_entry(accum, be)
                }
                ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce) => {
                    Self::format_breakable_call_chain_entry(accum, bcce)
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(x) => match x {
                    BeginCallChainIndent => accum.additional_indent += 1,
                    EndCallChainIndent => accum.additional_indent -= 1,
                    _ => accum.push(x),
                },
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
                [&ConcreteLineToken::HeredocClose { .. }, &ConcreteLineToken::HardNewLine, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Indent { .. }, &ConcreteLineToken::Delim { .. }
                | &ConcreteLineToken::Dot
                | &ConcreteLineToken::DirectPart { .. }],
            ) = accum.last::<5>()
            {
                accum.fix_heredoc_duplicate_indent_mistake();
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
        let length = be.single_line_string_length(accum.current_line_length());
        // We generally will force expressions embedded in strings to be on a single line,
        // but if that expression has a heredoc nested in it, we should let it render across lines
        // so that the collapsing newlines render properly.
        let force_single_line =
            !be.any_collapsing_newline_has_heredoc_content() && be.in_string_embexpr();

        if !force_single_line && (length > MAX_LINE_LENGTH || be.is_multiline()) {
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

    fn format_breakable_call_chain_entry(
        accum: &mut Intermediary,
        mut bcce: BreakableCallChainEntry,
    ) {
        let length = bcce.single_line_string_length(accum.current_line_length());
        let must_multiline =
            bcce.any_collapsing_newline_has_heredoc_content() && bcce.in_string_embexpr();
        if must_multiline
            || ((length > MAX_LINE_LENGTH || bcce.is_multiline()) && !bcce.in_string_embexpr())
        {
            let tokens = bcce.into_tokens(ConvertType::MultiLine);
            Self::render_as(accum, tokens);
        } else {
            bcce.remove_call_chain_magic_tokens();
            Self::render_as(accum, bcce.into_tokens(ConvertType::SingleLine));
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
