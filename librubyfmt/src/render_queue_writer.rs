use crate::heredoc_string::HeredocKind;
use crate::intermediary::{BlanklineReason, Intermediary};
use crate::line_tokens::*;
use crate::render_targets::{
    AbstractTokenTarget, BreakableCallChainEntry, BreakableEntry, ConditionalLayoutEntry,
    ConvertType,
};
use crate::util::get_indent;
#[cfg(debug_assertions)]
use log::debug;
use std::io::{self, Write};

pub const MAX_LINE_LENGTH: usize = 120;

pub struct RenderQueueWriter<'src> {
    tokens: Vec<ConcreteLineTokenAndTargets<'src>>,
}

impl<'src> RenderQueueWriter<'src> {
    pub fn new(tokens: Vec<ConcreteLineTokenAndTargets<'src>>) -> Self {
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

    /// Convert the render queue to final tokens without writing them.
    /// This is used for extracting heredoc segments.
    pub fn into_tokens(self) -> Vec<ConcreteLineToken<'src>> {
        let mut accum = Intermediary::new();
        Self::render_as(&mut accum, self.tokens);
        accum.into_tokens()
    }

    fn render_as(accum: &mut Intermediary<'src>, tokens: Vec<ConcreteLineTokenAndTargets<'src>>) {
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
                }) if !contents.is_empty() => {
                    let indent = get_indent(accum.additional_indent as usize * 2);
                    let mut new_contents = indent.into_owned();
                    new_contents.extend_from_slice(contents);
                    next_token =
                        ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::Comment {
                            contents: new_contents.into(),
                        })
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::DirectPart {
                    part,
                }) if current_heredoc_kind.is_some_and(|k| k.is_squiggly()) => {
                    let indent = get_indent(accum.additional_indent as usize * 2);
                    let indent_bytes = indent.as_ref();
                    let mut new_contents = Vec::new();
                    let parts = part.split(|&b| b == b'\n');

                    for (i, p) in parts.enumerate() {
                        if i > 0 {
                            new_contents.push(b'\n');
                        }
                        if !p.is_empty() {
                            new_contents.extend_from_slice(indent_bytes);
                        }
                        new_contents.extend_from_slice(p);
                    }
                    next_token = clats_direct_part(new_contents)
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::HeredocStart { kind, .. },
                ) => current_heredoc_kind = Some(*kind),
                ConcreteLineTokenAndTargets::ConcreteLineToken(
                    ConcreteLineToken::HeredocClose { symbol },
                ) => {
                    // Bare heredocs (e.g. <<FOO) must have the closing ident completely unindented, so
                    // ignore them in this case
                    if current_heredoc_kind.is_some_and(|k| !k.is_bare()) {
                        let indent = get_indent(accum.additional_indent as usize * 2);
                        let mut new_contents = indent.into_owned();
                        new_contents.extend_from_slice(symbol);
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
                ConcreteLineTokenAndTargets::ConditionalLayoutEntry(cle) => {
                    Self::format_conditional_layout_entry(accum, cle)
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(x) => match x {
                    BeginCallChainIndent => accum.additional_indent += 1,
                    EndCallChainIndent => accum.additional_indent -= 1,
                    _ => accum.push(x),
                },
            }

            if let Some(
                [
                    ConcreteLineToken::End,
                    ConcreteLineToken::HardNewLine,
                    ConcreteLineToken::Indent { .. },
                    x,
                ],
            ) = accum.last::<4>()
                && x.is_in_need_of_a_trailing_blankline()
            {
                accum.insert_trailing_blankline(BlanklineReason::ComesAfterEnd);
            }

            if let Some(
                [
                    ConcreteLineToken::End,
                    ConcreteLineToken::AfterCallChain,
                    ConcreteLineToken::HardNewLine,
                    ConcreteLineToken::Indent { .. },
                    x,
                    maybe_space,
                    maybe_def,
                ],
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
                [
                    ConcreteLineToken::HeredocClose { .. },
                    ConcreteLineToken::HardNewLine,
                    ConcreteLineToken::Indent { .. },
                    ConcreteLineToken::Indent { .. },
                    ConcreteLineToken::Delim { .. }
                    | ConcreteLineToken::Dot
                    | ConcreteLineToken::DirectPart { .. }
                    | ConcreteLineToken::MethodName { .. },
                ],
            ) = accum.last::<5>()
            {
                accum.fix_heredoc_duplicate_indent_mistake();
            }
        }
    }

    fn format_breakable_entry(accum: &mut Intermediary<'src>, be: BreakableEntry<'src>) {
        // We generally will force expressions embedded in strings to be on a single line,
        // but if that expression has a heredoc nested in it, we should let it render across lines
        // so that the collapsing newlines render properly. Similarly, if the entry already
        // contains hard newlines (e.g. a multi-statement brace block), forcing single-line
        // would leave dangling newlines and squash statements together.
        let force_single_line = be.in_string_embexpr()
            && !be.any_collapsing_newline_has_heredoc_content()
            && !be.contains_hard_newline();

        let mut tokens = Vec::new();
        if !force_single_line
            && (be.is_multiline() || Self::renders_over_max_line_length(accum, &be))
        {
            be.write_tokens(ConvertType::MultiLine, &mut tokens);
            Self::render_as(accum, tokens);
        } else {
            be.write_tokens(ConvertType::SingleLine, &mut tokens);
            Self::render_as(accum, tokens);
            // after running accum looks like this (or some variant):
            // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
            // so we remove items at positions length-2 until there is nothing
            // in that position that is garbage.
            accum.clear_breakable_garbage();
        }
    }

    fn format_breakable_call_chain_entry(
        accum: &mut Intermediary<'src>,
        mut bcce: BreakableCallChainEntry<'src>,
    ) {
        let must_multiline =
            bcce.any_collapsing_newline_has_heredoc_content() && bcce.in_string_embexpr();
        let mut tokens = Vec::new();
        if must_multiline
            || ((bcce.is_multiline() || Self::renders_over_max_line_length(accum, &bcce))
                && !bcce.in_string_embexpr())
        {
            bcce.write_tokens(ConvertType::MultiLine, &mut tokens);
            Self::render_as(accum, tokens);
        } else {
            bcce.remove_call_chain_magic_tokens();
            bcce.write_tokens(ConvertType::SingleLine, &mut tokens);
            Self::render_as(accum, tokens);
        }
    }

    fn format_conditional_layout_entry(
        accum: &mut Intermediary<'src>,
        cle: ConditionalLayoutEntry<'src>,
    ) {
        if cle.should_use_block_form(accum.current_line_length()) {
            Self::render_as(accum, cle.into_block_tokens());
        } else {
            Self::render_as(accum, cle.into_inline_tokens());
        }
    }

    fn renders_over_max_line_length(
        accum: &Intermediary<'src>,
        breakable: &dyn AbstractTokenTarget<'src>,
    ) -> bool {
        breakable.single_line_string_length(accum.current_line_length()) > MAX_LINE_LENGTH
    }

    fn write_final_tokens<W: Write>(
        writer: &mut W,
        mut tokens: Vec<ConcreteLineToken<'src>>,
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
            writer.write_all(&s)?;
        }
        Ok(())
    }
}
