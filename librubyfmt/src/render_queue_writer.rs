use crate::intermediary::{BlanklineReason, Intermediary};
use crate::line_tokens::*;
use crate::render_targets::{AbstractTokenTarget, BreakableEntry, ConvertType, HeredocString};
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
        let mut heredocs = vec![];
        #[cfg(debug_assertions)]
        {
            debug!("first tokens {:?}", self.tokens);
        }
        Self::render_as(&mut accum, self.tokens, &mut heredocs);
        Self::write_final_tokens(writer, accum.into_tokens())
    }

    fn render_as(accum: &mut Intermediary, tokens: Vec<ConcreteLineTokenAndTargets>, heredocs: &mut Vec<HeredocString>) {
        for next_token in tokens.into_iter() {
            match next_token {
                ConcreteLineTokenAndTargets::BreakableEntry(be) => {
                    Self::format_breakable_entry(accum, be, heredocs)
                }
                ConcreteLineTokenAndTargets::HeredocString(hs) => {
                    heredocs.push(hs);
                    #[cfg(debug_assertions)]
                    {
                        debug!("render_as: got heredoc {}", heredocs.len())
                    }
                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::LTStringContent {content}) => {
                    let parts: Vec<&str> = content.splitn(2, '\n').collect();

                    if !parts[0].is_empty() {
                        accum.push(ConcreteLineToken::LTStringContent {
                            content: parts[0].to_string()
                        });
                    }
                    if parts.len() == 2 {
                        accum.push(ConcreteLineToken::HardNewLine);
                        Self::format_heredocs(accum, heredocs);

                        if !parts[1].is_empty() {
                            accum.push(ConcreteLineToken::LTStringContent {
                                content: parts[1].to_string()
                            });
                        }
                    }

                }
                ConcreteLineTokenAndTargets::ConcreteLineToken(x) => {
                    let is_newline = x.is_newline();
                    accum.push(x);
                    if is_newline {
                        #[cfg(debug_assertions)]
                        {
                            debug!("render_as: got newline")
                        }
                        Self::format_heredocs(accum, heredocs);
                    }
                }
            }

            if accum.len() >= 4 {
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
        }
    }

    fn format_breakable_entry(accum: &mut Intermediary, be: BreakableEntry, heredocs: &mut Vec<HeredocString>) {
        let length = be.single_line_string_length();

        if length > MAX_LINE_LENGTH || be.is_multiline() {
            Self::render_as(accum, be.into_tokens(ConvertType::MultiLine), heredocs);
        } else {
            Self::render_as(accum, be.into_tokens(ConvertType::SingleLine), heredocs);
            // after running accum looks like this (or some variant):
            // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
            // so we remove items at positions length-2 until there is nothing
            // in that position that is garbage.
            accum.clear_breakable_garbage();
        }
    }

    fn format_heredocs(accum: &mut Intermediary, heredocs: &mut Vec<HeredocString>) {
        #[cfg(debug_assertions)]
        {
            debug!("format_heredocs {}", heredocs.len())
        }
        for hs in heredocs.drain(0..) {
            accum.push(ConcreteLineToken::LTStringContent {
                content: hs.content
            });
            accum.push(ConcreteLineToken::Indent {
                depth: hs.indent
            });
            accum.push(ConcreteLineToken::DirectPart {
                part: hs.symbol
            });
            accum.push(ConcreteLineToken::HardNewLine);
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
