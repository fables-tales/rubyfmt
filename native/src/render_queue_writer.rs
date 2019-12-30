use crate::line_tokens::*;
use std::io::{self, Write};

const MAX_LINE_LENGTH: usize = 80;

pub struct RenderQueueWriter {
    tokens: Vec<LineToken>,
}

enum ConvertType {
    MultiLine,
    SingleLine,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<LineToken>) -> Self {
        RenderQueueWriter { tokens: tokens }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        eprintln!("{:?}", self.tokens);
        let mut accum = vec!();
        Self::render_as(&mut accum, self.tokens, ConvertType::MultiLine);
        Self::write_final_tokens(writer, accum)
    }

    fn render_as(accum: &mut Vec<LineToken>, tokens: Vec<LineToken>, convert_type: ConvertType) {
        let mut token_iter = tokens.into_iter();

        while let Some(next_token) = token_iter.next() {
            match next_token {
                LineToken::BreakableEntry(be) => Self::format_breakable_entry(accum, be),
                x => accum.push(match convert_type {
                    ConvertType::MultiLine => x.as_multi_line(),
                    ConvertType::SingleLine => x.as_single_line(),
                }),
            }
        }
    }

    fn format_breakable_entry(accum: &mut Vec<LineToken>, be: BreakableEntry) {
        let length = be.single_line_string_length();

        if length > MAX_LINE_LENGTH {
            Self::render_as(
                accum,
                be.tr_delims().as_tokens(),
                ConvertType::MultiLine,
            );
        } else {
            Self::render_as(accum, be.as_tokens(), ConvertType::SingleLine);
            // after running accum looks like this (or some variant):
            // [.., Comma, Space, DirectPart {part: ""}, <close_delimiter>]
            // so we remove items at positions length-2 until there is nothing
            // in that position that is garbage.
            while accum[accum.len()-2].is_single_line_breakable_garbage() {
                accum.remove(accum.len()-2);
            }
        }
    }

    fn write_final_tokens<W: Write>(writer: &mut W, tokens: Vec<LineToken>) -> io::Result<()> {
        for line_token in tokens.into_iter() {
            let s = line_token.to_string();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
