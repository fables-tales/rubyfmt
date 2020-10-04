use crate::comment_block::{CommentBlock, Merge};
use crate::delimiters::BreakableDelims;
use crate::file_comments::FileComments;
use crate::format::{format_inner_string, StringType};
use crate::line_tokens::*;
use crate::render_queue_writer::RenderQueueWriter;
use crate::render_targets::{BaseQueue, BreakableEntry};
use crate::ripper_tree_types::StringContentPart;
use crate::types::{ColNumber, LineNumber};
use log::debug;
use std::io::{self, Cursor, Write};
use std::str;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormattingContext {
    Main,
    Assign,
    Binary,
    ClassOrModule,
    Def,
    CurlyBlock,
    ArgsList,
    IfOp,
}

#[derive(Clone, Copy)]
struct IndentDepth {
    depth: ColNumber,
}

impl IndentDepth {
    fn new() -> Self {
        IndentDepth { depth: 0 }
    }

    fn increment(&mut self) {
        self.depth += 1;
    }

    fn decrement(&mut self) {
        self.depth -= 1;
    }

    fn get(self) -> ColNumber {
        self.depth
    }
}

pub struct HeredocString {
    symbol: String,
    squiggly: bool,
    buf: Vec<u8>,
}

impl HeredocString {
    pub fn new(symbol: String, squiggly: bool, buf: Vec<u8>) -> Self {
        HeredocString {
            symbol,
            squiggly,
            buf,
        }
    }
}
pub struct ParserState {
    depth_stack: Vec<IndentDepth>,
    start_of_line: Vec<bool>,
    surpress_comments_stack: Vec<bool>,
    render_queue: BaseQueue,
    current_orig_line_number: LineNumber,
    comments_hash: FileComments,
    heredoc_strings: Vec<HeredocString>,
    comments_to_insert: Option<CommentBlock>,
    breakable_entry_stack: Vec<BreakableEntry>,
    formatting_context: Vec<FormattingContext>,
    absorbing_indents: i32,
    insert_user_newlines: bool,
    spaces_after_last_newline: ColNumber,
}

impl ParserState {
    pub fn new(fc: FileComments) -> Self {
        ParserState {
            depth_stack: vec![IndentDepth::new()],
            start_of_line: vec![true],
            surpress_comments_stack: vec![false],
            render_queue: BaseQueue::default(),
            current_orig_line_number: 0,
            comments_hash: fc,
            heredoc_strings: vec![],
            comments_to_insert: None,
            breakable_entry_stack: vec![],
            formatting_context: vec![FormattingContext::Main],
            absorbing_indents: 0,
            insert_user_newlines: true,
            spaces_after_last_newline: 0,
        }
    }

    fn consume_to_render_queue(self) -> Vec<ConcreteLineTokenAndTargets> {
        self.render_queue.into_tokens()
    }

    pub fn last_breakable_is_multiline(&self) -> bool {
        self.breakable_entry_stack
            .last()
            .map(|o| o.is_multiline())
            .unwrap_or(false)
    }

    pub fn on_line(&mut self, line_number: LineNumber) {
        if line_number < self.current_orig_line_number {
            return;
        }
        debug!("on_line called: {}", line_number);

        for be in self.breakable_entry_stack.iter_mut().rev() {
            be.push_line_number(line_number);
        }

        let comments = self.comments_hash.extract_comments_to_line(line_number);
        self.push_comments(line_number, comments);
    }

    fn push_comments(&mut self, line_number: LineNumber, comments: Option<CommentBlock>) {
        match comments {
            None => {}
            Some(comments) => {
                if !self
                    .surpress_comments_stack
                    .last()
                    .expect("comments stack is never empty")
                {
                    let len = comments.len();
                    self.insert_comment_collection(comments);
                    self.current_orig_line_number += len as u64;
                }
            }
        }

        if line_number - self.current_orig_line_number >= 2 && self.insert_user_newlines {
            self.insert_extra_newline_at_last_newline();
        }

        self.current_orig_line_number = line_number;
    }

    fn insert_extra_newline_at_last_newline(&mut self) {
        let idx = self.index_of_prev_hard_newline();
        let insert_idx = match idx {
            Some(idx) => idx + 1,
            None => 0,
        };

        self.insert_concrete_tokens(insert_idx, vec![ConcreteLineToken::HardNewLine]);
    }

    pub fn insert_comment_collection(&mut self, comments: CommentBlock) {
        self.comments_to_insert
            .merge(comments.apply_spaces(self.spaces_after_last_newline));
    }

    pub fn emit_indent(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Indent {
                depth: self.current_spaces(),
            },
        ));
    }

    pub fn emit_op(&mut self, op: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Op { op },
        ));
    }

    pub fn emit_double_quote(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::DoubleQuote,
        ));
    }

    pub fn emit_string_content(&mut self, s: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::LTStringContent { content: s },
        ));
    }

    fn current_spaces(&self) -> ColNumber {
        2 * self
            .depth_stack
            .last()
            .expect("depth stack is never empty")
            .get()
    }

    pub fn emit_ident(&mut self, ident: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::DirectPart { part: ident },
        ));
    }

    pub fn emit_keyword(&mut self, kw: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword { keyword: kw },
        ));
    }

    pub fn emit_mod_keyword(&mut self, contents: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::ModKeyword { contents },
        ));
    }

    pub fn emit_conditional_keyword(&mut self, contents: String) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::ConditionalKeyword { contents },
        ));
    }

    pub fn emit_def_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::DefKeyword,
        ));
    }

    pub fn emit_case_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "case".to_string(),
            },
        ));
    }

    pub fn emit_when_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "when".to_string(),
            },
        ));
    }

    pub fn emit_do_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::DoKeyword,
        ));
    }

    pub fn emit_class_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::ClassKeyword,
        ));
    }

    pub fn emit_module_keyword(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::ModuleKeyword,
        ));
    }

    pub fn emit_rescue(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "rescue".to_string(),
            },
        ));
    }

    pub fn emit_ensure(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "ensure".to_string(),
            },
        ));
    }

    pub fn emit_begin(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "begin".to_string(),
            },
        ));
    }

    pub fn emit_begin_block(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "BEGIN".to_string(),
            },
        ));
    }

    pub fn emit_end_block(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Keyword {
                keyword: "END".to_string(),
            },
        ));
    }

    pub fn emit_soft_indent(&mut self) {
        self.push_token(AbstractLineToken::SoftIndent {
            depth: self.current_spaces(),
        });
    }

    pub fn emit_comma(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Comma,
        ));
    }

    pub fn emit_soft_newline(&mut self) {
        self.new_block(|ps| {
            ps.shift_comments();
        });
        self.push_token(AbstractLineToken::SoftNewline);
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub fn emit_collapsing_newline(&mut self) {
        if !self.last_token_is_a_newline() {
            self.push_token(AbstractLineToken::CollapsingNewLine);
        }
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub fn emit_def(&mut self, def_name: String) {
        self.emit_def_keyword();
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::DirectPart {
                part: format!(" {}", def_name),
            },
        ));
    }

    pub fn emit_newline(&mut self) {
        self.shift_comments();
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::HardNewLine,
        ));
        self.render_heredocs(false);
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub fn emit_end(&mut self) {
        if !self.last_token_is_a_newline() {
            self.emit_newline();
        }
        if self.at_start_of_line() {
            self.emit_indent();
        }
        self.push_token(AbstractLineToken::ConcreteLineToken(ConcreteLineToken::End));
    }

    fn last_token_is_a_newline(&self) -> bool {
        match self.breakable_entry_stack.last() {
            Some(be) => be.last_token_is_a_newline(),
            None => self.render_queue.last_token_is_a_newline(),
        }
    }

    pub fn shift_comments(&mut self) {
        let idx_of_prev_hard_newline = self.index_of_prev_hard_newline();

        if let Some(new_comments) = self.comments_to_insert.take() {
            let insert_index = match idx_of_prev_hard_newline {
                Some(idx) => idx + 1,
                None => 0,
            };

            self.insert_concrete_tokens(
                insert_index,
                new_comments.into_line_tokens().into_iter().collect(),
            );
        }
    }

    pub fn index_of_prev_hard_newline(&self) -> Option<usize> {
        match self.breakable_entry_stack.last() {
            Some(be) => be.index_of_prev_hard_newline(),
            None => self.render_queue.index_of_prev_hard_newline(),
        }
    }

    pub fn emit_else(&mut self) {
        self.emit_conditional_keyword("else".to_string());
    }

    pub fn emit_comma_space(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::CommaSpace,
        ))
    }

    pub fn emit_space(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::Space,
        ));
    }

    pub fn emit_dot(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(ConcreteLineToken::Dot));
    }

    pub fn emit_colon_colon(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::ColonColon,
        ));
    }

    pub fn emit_lonely_operator(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::LonelyOperator,
        ));
    }

    pub fn magic_handle_comments_for_mulitiline_arrays<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        let current_line_number = self.current_orig_line_number;
        self.new_block(|ps| {
            ps.shift_comments();
        });
        f(self);
        let new_line_number = self.current_orig_line_number;
        if new_line_number > current_line_number {
            self.wind_line_forward();
            self.shift_comments();
        }
        self.current_orig_line_number = new_line_number;
    }
    pub fn with_surpress_comments<F>(&mut self, surpress: bool, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.surpress_comments_stack.push(surpress);
        f(self);
        self.surpress_comments_stack.pop();
    }

    pub fn with_formatting_context<F>(&mut self, fc: FormattingContext, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.formatting_context.push(fc);
        f(self);
        self.formatting_context.pop();
    }

    pub fn with_absorbing_indent_block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        let was_absorving = self.absorbing_indents != 0;
        self.absorbing_indents += 1;
        if was_absorving {
            f(self);
        } else {
            self.new_block(f);
        }
        self.absorbing_indents -= 1;
    }

    pub fn new_block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
        f(self);
        self.depth_stack[ds_length - 1].decrement();
    }

    pub fn dedent<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].decrement();
        f(self);
        self.depth_stack[ds_length - 1].increment();
    }

    pub fn with_start_of_line<F>(&mut self, start_of_line: bool, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.start_of_line.push(start_of_line);
        f(self);
        self.start_of_line.pop();
    }

    pub fn at_start_of_line(&self) -> bool {
        *self
            .start_of_line
            .last()
            .expect("start of line is never_empty")
    }

    pub fn current_formatting_context(&self) -> FormattingContext {
        *self
            .formatting_context
            .last()
            .expect("formatting context is never empty")
    }

    pub fn current_formatting_context_requires_parens(&self) -> bool {
        self.current_formatting_context() == FormattingContext::Binary
            || self.current_formatting_context() == FormattingContext::IfOp
    }

    pub fn new_with_depth_stack_from(ps: &ParserState) -> Self {
        let mut next_ps = ParserState::new(FileComments::default());
        next_ps.depth_stack = ps.depth_stack.clone();
        next_ps.current_orig_line_number = ps.current_orig_line_number;
        next_ps
    }

    pub fn render_with_blank_state<F>(ps: &mut ParserState, f: F) -> ParserState
    where
        F: FnOnce(&mut ParserState),
    {
        let mut next_ps = ParserState::new_with_depth_stack_from(ps);
        f(&mut next_ps);
        next_ps
    }

    pub fn push_heredoc_content(
        &mut self,
        symbol: String,
        is_squiggly: bool,
        parts: Vec<StringContentPart>,
    ) {
        let mut next_ps = ParserState::render_with_blank_state(self, |n| {
            n.insert_user_newlines = false;
            format_inner_string(n, parts, StringType::Heredoc);
            n.emit_newline();
        });

        for hs in next_ps.heredoc_strings.drain(0..) {
            self.heredoc_strings.push(hs);
        }

        let data = next_ps.render_to_buffer();
        self.heredoc_strings
            .push(HeredocString::new(symbol, is_squiggly, data));
    }

    pub fn will_render_as_multiline<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(&mut ParserState),
    {
        let mut next_ps = ParserState::new_with_depth_stack_from(self);
        f(&mut next_ps);
        let data = next_ps.render_to_buffer();

        let s = str::from_utf8(&data).expect("string is utf8").to_string();
        s.trim().contains('\n')
    }

    fn render_to_buffer(self) -> Vec<u8> {
        let mut bufio = Cursor::new(Vec::new());
        self.write(&mut bufio).expect("in memory io cannot fail");
        bufio.set_position(0);
        bufio.into_inner()
    }

    pub fn render_heredocs(&mut self, skip: bool) {
        while !self.heredoc_strings.is_empty() {
            let mut next_heredoc = self.heredoc_strings.pop().expect("we checked it's there");
            let want_newline = !self.last_token_is_a_newline();
            if want_newline {
                self.push_token(AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::HardNewLine,
                ));
            }

            if let Some(b'\n') = next_heredoc.buf.last() {
                next_heredoc.buf.pop();
            };

            if let Some(b'\n') = next_heredoc.buf.last() {
                next_heredoc.buf.pop();
            };

            self.push_token(AbstractLineToken::ConcreteLineToken(
                ConcreteLineToken::DirectPart {
                    part: String::from_utf8(next_heredoc.buf).expect("hereoc is utf8"),
                },
            ));
            self.emit_newline();
            if next_heredoc.squiggly {
                self.emit_indent();
            } else {
                self.push_token(AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::Indent { depth: 0 },
                ));
            }
            self.emit_ident(next_heredoc.symbol.replace("'", ""));
            if !skip {
                self.emit_newline();
            }
        }
    }

    pub fn breakable_of<F>(&mut self, delims: BreakableDelims, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.shift_comments();
        let mut be = BreakableEntry::new(self.current_spaces(), delims);
        be.push_line_number(self.current_orig_line_number);
        self.breakable_entry_stack.push(be);

        self.new_block(|ps| {
            ps.emit_collapsing_newline();
            f(ps);
        });

        self.emit_soft_indent();

        let insert_be = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed");
        self.push_token(AbstractLineToken::BreakableEntry(insert_be));
    }

    pub fn emit_open_square_bracket(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::OpenSquareBracket,
        ));
    }

    pub fn emit_close_square_bracket(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::CloseSquareBracket,
        ));
    }

    pub fn emit_open_curly_bracket(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::OpenCurlyBracket,
        ));
    }

    pub fn emit_close_curly_bracket(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::CloseCurlyBracket,
        ));
    }

    pub fn emit_slash(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::SingleSlash,
        ));
    }

    pub fn emit_open_paren(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::OpenParen,
        ));
    }

    pub fn emit_close_paren(&mut self) {
        self.push_token(AbstractLineToken::ConcreteLineToken(
            ConcreteLineToken::CloseParen,
        ));
    }

    pub fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let rqw = RenderQueueWriter::new(self.consume_to_render_queue());
        rqw.write(writer)
    }

    fn dangerously_convert(t: AbstractLineToken) -> ConcreteLineTokenAndTargets {
        match t {
            AbstractLineToken::ConcreteLineToken(clt) => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(clt)
            }
            AbstractLineToken::BreakableEntry(be) => {
                ConcreteLineTokenAndTargets::BreakableEntry(be)
            }
            _ => panic!("failed to convert"),
        }
    }

    pub fn is_absorbing_indents(&self) -> bool {
        self.absorbing_indents >= 1
    }

    pub fn wind_line_forward(&mut self) {
        self.on_line(self.current_orig_line_number + 1);
    }

    pub fn flush_start_of_file_comments(&mut self) {
        match self
            .comments_hash
            .take_start_of_file_contiguous_comment_lines()
        {
            None => {
                self.on_line(1);
            }
            Some(comments) => {
                self.push_comments(comments.len() as LineNumber, Some(comments));
                self.shift_comments();
                debug!("rq: {:?}", self.render_queue);
            }
        }
    }

    pub fn wind_dumping_comments(&mut self) {
        self.on_line(self.current_orig_line_number + 1);
        while self
            .comments_hash
            .has_line(self.current_orig_line_number + 1)
        {
            self.on_line(self.current_orig_line_number + 1);
        }
    }

    fn insert_concrete_tokens(&mut self, insert_idx: usize, clts: Vec<ConcreteLineToken>) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.insert_at(
                insert_idx,
                &mut clts
                    .into_iter()
                    .map(AbstractLineToken::ConcreteLineToken)
                    .collect(),
            ),
            None => self.render_queue.insert_at(
                insert_idx,
                &mut clts
                    .into_iter()
                    .map(ConcreteLineTokenAndTargets::ConcreteLineToken)
                    .collect(),
            ),
        }
    }

    pub fn push_token(&mut self, t: AbstractLineToken) {
        if let AbstractLineToken::SoftIndent { .. } = t {
            if self.breakable_entry_stack.is_empty() {
                panic!("should be impossible")
            }
        }

        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(t),
            None => self.render_queue.push(Self::dangerously_convert(t)),
        }
    }
}
