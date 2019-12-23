use crate::line_tokens::*;
use std::io::{self, Write};

pub struct RenderQueueWriter {
    tokens: Vec<Box<dyn LineToken>>,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<Box<dyn LineToken>>) -> Self {
        RenderQueueWriter {
            tokens: tokens,
        }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let mut idx = 0;
        let mut accum = vec!();
        while idx < self.tokens.len() {
            let token = self.tokens[idx];

            if token.is_breakable_state() {
            } else {
                accum.append(vec
            }

        }

        for line_token in self.tokens.into_iter() {
            let s = line_token.consume_to_string();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
