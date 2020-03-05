use crate::breakable_entry::{BreakableEntry, ConvertType};
use crate::intermediary::Intermediary;
use crate::line_tokens::*;
use std::io::{self, Write};

const MAX_LINE_LENGTH: usize = 120;

pub struct RenderQueueWriter {
    tokens: Vec<LineToken>,
}

impl RenderQueueWriter {
    pub fn new(tokens: Vec<LineToken>) -> Self {
        RenderQueueWriter { tokens }
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let mut accum = Intermediary::new();
        eprintln!("first tokens");
        eprintln!("{:?}", self.tokens.clone());
        Self::render_as(
            &mut accum,
            self.tokens.into_iter().map(|t| t.into_multi_line()).collect(),
        );
        Self::write_final_tokens(writer, accum.into_tokens())
    }

    fn render_as(accum: &mut Intermediary, tokens: Vec<LineToken>) {
        for next_token in tokens.into_iter() {
            match next_token {
                LineToken::BreakableEntry(be) => Self::format_breakable_entry(accum, be),
                x => accum.push(x),
            }

            if accum.len() >= 4 {
                match accum.last_4().expect("we checked length") {
                    // do nothing in the case we're in an end cascade
                    (
                        &LineToken::End,
                        &LineToken::HardNewLine,
                        &LineToken::Indent { .. },
                        &LineToken::End,
                    ) => {}
                    // in this case we have an end followed by something that isn't
                    // an end, so insert an extra blankline
                    (&LineToken::End, &LineToken::HardNewLine, &LineToken::Indent { .. }, x) => {
                        eprintln!(">>>>>>>>>>>>>>>>>>>> {:?}", x);
                        eprintln!("coming from cleanup");
                        accum.insert_trailing_blankline();
                    }
                    _ => {}
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

    fn write_final_tokens<W: Write>(writer: &mut W, tokens: Vec<LineToken>) -> io::Result<()> {
        eprintln!("last tokens");
        eprintln!("{:?}", tokens);
        for line_token in tokens.into_iter() {
            let s = line_token.into_ruby();
            write!(writer, "{}", s)?
        }
        Ok(())
    }
}
