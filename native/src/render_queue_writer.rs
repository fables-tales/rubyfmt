use crate::line_tokens::*;
use crate::breakable_entry::{BreakableEntry, ConvertType};
use std::io::{self, Write};

const MAX_LINE_LENGTH: usize = 120;

pub struct RenderQueueWriter {
    tokens: Vec<LineToken>,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<LineToken>) -> Self {
        RenderQueueWriter { tokens: tokens }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let mut accum = vec!();
        Self::render_as(
            &mut accum,
            self.tokens.into_iter().map(|t| t.as_multi_line()).collect(),
        );
        Self::write_final_tokens(writer, accum)
    }

    fn render_as(accum: &mut Vec<LineToken>, tokens: Vec<LineToken>) {
        let mut token_iter = tokens.into_iter();

        while let Some(next_token) = token_iter.next() {
            match next_token {
                LineToken::BreakableEntry(be) => Self::format_breakable_entry(accum, be),
                x => accum.push(x),
            }
        }
    }

    fn format_breakable_entry(accum: &mut Vec<LineToken>, be: BreakableEntry) {
        let length = be.single_line_string_length();
        eprintln!("----------------");
        eprintln!("{:?}", be);
        eprintln!("----------------");

        if length > MAX_LINE_LENGTH  || be.is_multiline() {
            Self::render_as(
                accum,
                be.as_tokens(ConvertType::MultiLine),
            );
        } else {
            Self::render_as(accum, be.as_tokens(ConvertType::SingleLine));
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
