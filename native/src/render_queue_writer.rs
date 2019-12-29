use crate::line_tokens::*;
use std::io::{self, Write};

pub struct RenderQueueWriter {
    tokens: Vec<LineToken>,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<LineToken>) -> Self {
        RenderQueueWriter { tokens: tokens }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        //let mut accum = vec!();
        //let mut token_drain = self.tokens.into_iter();
        let mut final_tokens = self.tokens;

        //while let Some(next_token) = token_drain.next() {
        //    if next_token.is_breakable_entry() {
        //        Self::format_breakable_entry(&mut accum, next_token.to_breakable_entry());
        //    } else {
        //        accum.push(next_token);
        //    }
        //}

        Self::write_final_tokens(writer, final_tokens)
    }

    fn format_breakable_entry<I>(accum: &mut Vec<LineToken>, be: BreakableEntry) {}

    fn write_final_tokens<W: Write>(writer: &mut W, tokens: Vec<LineToken>) -> io::Result<()> {
        for line_token in tokens.into_iter() {
            let s = line_token.to_string();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
