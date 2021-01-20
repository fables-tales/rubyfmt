use crate::comment_block::{CommentBlock, Merge};
use crate::delimiters::BreakableDelims;
use crate::file_comments::FileComments;
use crate::format::{format_inner_string, StringType};
use crate::line_tokens::*;
use crate::render_queue_writer::RenderQueueWriter;
use crate::render_targets::{AbstractTokenTarget, BaseQueue, BreakableEntry};
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
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
pub trait ConcreteParserState {
    // token emitters
    fn emit_conditional_keyword(&mut self, contents: String);
    fn emit_mod_keyword(&mut self, contents: String);
    fn emit_keyword(&mut self, kw: String);
    fn emit_def_keyword(&mut self);
    fn emit_end_block(&mut self);
    fn emit_colon_colon(&mut self);
    fn emit_lonely_operator(&mut self);
    fn emit_dot(&mut self);
    fn emit_else(&mut self);
    fn emit_begin_block(&mut self);
    fn emit_begin(&mut self);
    fn emit_ensure(&mut self);
    fn emit_module_keyword(&mut self);
    fn emit_class_keyword(&mut self);
    fn emit_do_keyword(&mut self);
    fn emit_when_keyword(&mut self);
    fn emit_case_keyword(&mut self);
    fn emit_rescue(&mut self);
    fn emit_open_square_bracket(&mut self);
    fn emit_close_square_bracket(&mut self);
    fn emit_open_curly_bracket(&mut self);
    fn emit_close_curly_bracket(&mut self);
    fn emit_slash(&mut self);
    fn emit_close_paren(&mut self);
    fn emit_comma_space(&mut self);
    fn emit_open_paren(&mut self);
    fn emit_space(&mut self);
    fn emit_comma(&mut self);
    fn emit_end(&mut self);
    fn emit_newline(&mut self);
    fn emit_ident(&mut self, ident: String);
    fn emit_string_content(&mut self, s: String);
    fn emit_double_quote(&mut self);
    fn emit_op(&mut self, op: String);
    fn emit_def(&mut self, def_name: String);
    fn emit_indent(&mut self);

    // other state changers
    fn insert_comment_collection(&mut self, comments: CommentBlock);
    fn wind_line_if_needed_for_array(&mut self);
    fn on_line(&mut self, line_number: LineNumber);
    fn wind_dumping_comments(&mut self);
    fn shift_comments(&mut self);
    fn wind_line_forward(&mut self);
    fn render_heredocs(&mut self, skip: bool);
    fn push_heredoc_content(
        &mut self,
        symbol: String,
        is_squiggly: bool,
        parts: Vec<StringContentPart>,
    );

    // queries
    fn at_start_of_line(&self) -> bool;
    fn current_formatting_context_requires_parens(&self) -> bool;
    fn current_formatting_context(&self) -> FormattingContext;
    fn is_absorbing_indents(&self) -> bool;

    // blocks
    fn with_formatting_context<'a>(
        &mut self,
        fc: FormattingContext,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn new_block<'a>(&mut self, f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>);
    fn with_start_of_line<'a>(
        &mut self,
        start_of_line: bool,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn breakable_of<'a>(
        &mut self,
        delims: BreakableDelims,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn dedent<'a>(&mut self, f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>);
    fn with_absorbing_indent_block<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn magic_handle_comments_for_mulitiline_arrays<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn with_surpress_comments<'a>(
        &mut self,
        surpress: bool,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    );
    fn will_render_as_multiline<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) -> bool;

    // stuff to remove from this enum
    fn emit_soft_newline(&mut self);
    fn emit_soft_indent(&mut self);
    fn emit_collapsing_newline(&mut self);
}

#[derive(Debug)]
pub struct BaseParserState {
    depth_stack: Vec<IndentDepth>,
    start_of_line: Vec<bool>,
    surpress_comments_stack: Vec<bool>,
    render_queue: BaseQueue,
    current_orig_line_number: LineNumber,
    comments_hash: FileComments,
    heredoc_strings: Vec<HeredocString>,
    comments_to_insert: Option<CommentBlock>,
    breakable_entry_stack: Vec<Box<dyn AbstractTokenTarget>>,
    formatting_context: Vec<FormattingContext>,
    absorbing_indents: i32,
    insert_user_newlines: bool,
    spaces_after_last_newline: ColNumber,
}

impl ConcreteParserState for BaseParserState {
    fn push_heredoc_content(
        &mut self,
        symbol: String,
        is_squiggly: bool,
        parts: Vec<StringContentPart>,
    ) {
        let mut next_ps = BaseParserState::render_with_blank_state(self, |n| {
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

    fn magic_handle_comments_for_mulitiline_arrays<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        let current_line_number = self.current_orig_line_number;
        self.new_block(Box::new(|ps| {
            ps.shift_comments();
        }));
        f(self);
        let new_line_number = self.current_orig_line_number;
        if new_line_number > current_line_number {
            self.wind_line_forward();
            self.shift_comments();
        }
        self.current_orig_line_number = new_line_number;
    }

    fn will_render_as_multiline<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) -> bool {
        let mut next_ps = BaseParserState::new_with_depth_stack_from(self);
        f(&mut next_ps);
        let data = next_ps.render_to_buffer();

        let s = str::from_utf8(&data).expect("string is utf8").to_string();
        s.trim().contains('\n')
    }

    fn dedent<'a>(&mut self, f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].decrement();
        f(self);
        self.depth_stack[ds_length - 1].increment();
    }

    fn with_start_of_line<'a>(
        &mut self,
        start_of_line: bool,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        self.start_of_line.push(start_of_line);
        f(self);
        self.start_of_line.pop();
    }

    fn breakable_of<'a>(
        &mut self,
        delims: BreakableDelims,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        self.shift_comments();
        let mut be = BreakableEntry::new(self.current_spaces(), delims);
        be.push_line_number(self.current_orig_line_number);
        self.breakable_entry_stack.push(Box::new(be));

        self.new_block(Box::new(|ps| {
            ps.emit_collapsing_newline();
            f(ps);
        }));

        self.emit_soft_indent();

        let insert_be = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed")
            .to_breakable_entry();
        self.push_target(ConcreteLineTokenAndTargets::BreakableEntry(insert_be));
    }

    fn with_surpress_comments<'a>(
        &mut self,
        surpress: bool,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        self.surpress_comments_stack.push(surpress);
        f(self);
        self.surpress_comments_stack.pop();
    }

    fn with_absorbing_indent_block<'a>(
        &mut self,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        let was_absorving = self.absorbing_indents != 0;
        self.absorbing_indents += 1;
        if was_absorving {
            f(self);
        } else {
            self.new_block(f);
        }
        self.absorbing_indents -= 1;
    }

    fn new_block<'a>(&mut self, f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
        f(self);
        self.depth_stack[ds_length - 1].decrement();
    }

    fn with_formatting_context<'a>(
        &mut self,
        fc: FormattingContext,
        f: Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>,
    ) {
        self.formatting_context.push(fc);
        f(self);
        self.formatting_context.pop();
    }

    fn on_line(&mut self, line_number: LineNumber) {
        if line_number < self.current_orig_line_number {
            return;
        }
        debug!("on_line called: {}", line_number);

        for be in self.breakable_entry_stack.iter_mut().rev() {
            be.push_line_number(line_number);
        }

        let comments = self.comments_hash.extract_comments_to_line(line_number);
        self.push_comments(comments);

        if line_number - self.current_orig_line_number >= 2 && self.insert_user_newlines {
            self.insert_extra_newline_at_last_newline();
        }

        self.current_orig_line_number = line_number;
    }

    fn emit_indent(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Indent {
            depth: self.current_spaces(),
        });
    }

    fn at_start_of_line(&self) -> bool {
        *self
            .start_of_line
            .last()
            .expect("start of line is never_empty")
    }

    fn emit_def(&mut self, def_name: String) {
        self.emit_def_keyword();
        self.push_concrete_token(ConcreteLineToken::DirectPart {
            part: format!(" {}", def_name),
        });
    }

    fn wind_line_if_needed_for_array(&mut self) {
        if self.last_breakable_is_multiline() {
            self.wind_line_forward()
        }
    }

    fn insert_comment_collection(&mut self, comments: CommentBlock) {
        self.comments_to_insert
            .merge(comments.apply_spaces(self.spaces_after_last_newline));
    }

    fn emit_op(&mut self, op: String) {
        self.push_concrete_token(ConcreteLineToken::Op { op });
    }

    fn emit_double_quote(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DoubleQuote);
    }

    fn emit_string_content(&mut self, s: String) {
        self.push_concrete_token(ConcreteLineToken::LTStringContent { content: s });
    }

    fn emit_ident(&mut self, ident: String) {
        if ident == "example" {
            debug!("--------- boogaloo ------------");
            debug!("ps: {:?}", self);
            debug!("---------------------");
        }
        self.push_concrete_token(ConcreteLineToken::DirectPart { part: ident });
    }

    fn emit_newline(&mut self) {
        debug!("---------------------");
        debug!("ps: {:?}", self);
        debug!("---------------------");

        self.shift_comments();
        self.push_concrete_token(ConcreteLineToken::HardNewLine);
        self.render_heredocs(false);
        self.spaces_after_last_newline = self.current_spaces();
    }

    fn wind_dumping_comments(&mut self) {
        self.on_line(self.current_orig_line_number + 1);
        let mut did_wind = false;
        while self
            .comments_hash
            .has_line(self.current_orig_line_number + 1)
        {
            self.on_line(self.current_orig_line_number + 1);
            did_wind = true;
        }
        if did_wind {
            self.on_line(self.current_orig_line_number + 1);
        }
    }

    fn emit_end(&mut self) {
        if !self.last_token_is_a_newline() {
            self.emit_newline();
        }
        if self.at_start_of_line() {
            self.emit_indent();
        }
        self.push_concrete_token(ConcreteLineToken::End);
    }

    fn emit_comma(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Comma);
    }

    fn shift_comments(&mut self) {
        let idx_of_prev_hard_newline = self.index_of_prev_hard_newline();
        if let Some(new_comments) = self.comments_to_insert.take() {
            let insert_index = match idx_of_prev_hard_newline {
                Some(idx) => idx,
                None => 0,
            };

            self.insert_concrete_tokens(
                insert_index,
                new_comments.into_line_tokens().into_iter().collect(),
            );
        }
    }

    fn emit_soft_newline(&mut self) {
        self.new_block(Box::new(|ps| {
            ps.shift_comments();
        }));
        self.push_abstract_token(AbstractLineToken::SoftNewline);
        self.spaces_after_last_newline = self.current_spaces();
    }

    fn emit_soft_indent(&mut self) {
        self.push_abstract_token(AbstractLineToken::SoftIndent {
            depth: self.current_spaces(),
        });
    }

    fn emit_collapsing_newline(&mut self) {
        if !self.last_token_is_a_newline() {
            self.push_abstract_token(AbstractLineToken::CollapsingNewLine);
        }
        self.spaces_after_last_newline = self.current_spaces();
    }

    fn emit_space(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Space);
    }

    fn emit_open_paren(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenParen);
    }

    fn emit_comma_space(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CommaSpace)
    }

    fn emit_close_paren(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseParen);
    }

    fn emit_slash(&mut self) {
        self.push_concrete_token(ConcreteLineToken::SingleSlash);
    }

    fn emit_close_curly_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseCurlyBracket);
    }

    fn emit_open_curly_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenCurlyBracket);
    }

    fn emit_close_square_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseSquareBracket);
    }

    fn emit_open_square_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenSquareBracket);
    }

    fn emit_rescue(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "rescue".to_string(),
        });
    }

    fn emit_case_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "case".to_string(),
        });
    }

    fn emit_when_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "when".to_string(),
        });
    }

    fn emit_do_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DoKeyword);
    }

    fn emit_class_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ClassKeyword);
    }

    fn emit_module_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ModuleKeyword);
    }

    fn emit_ensure(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "ensure".to_string(),
        });
    }

    fn emit_begin(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "begin".to_string(),
        });
    }

    fn emit_begin_block(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "BEGIN".to_string(),
        });
    }

    fn emit_else(&mut self) {
        self.emit_conditional_keyword("else".to_string());
    }

    fn wind_line_forward(&mut self) {
        self.on_line(self.current_orig_line_number + 1);
    }

    fn current_formatting_context_requires_parens(&self) -> bool {
        self.current_formatting_context() == FormattingContext::Binary
            || self.current_formatting_context() == FormattingContext::IfOp
    }

    fn emit_dot(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Dot);
    }

    fn emit_lonely_operator(&mut self) {
        self.push_concrete_token(ConcreteLineToken::LonelyOperator);
    }

    fn emit_colon_colon(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ColonColon);
    }

    fn current_formatting_context(&self) -> FormattingContext {
        *self
            .formatting_context
            .last()
            .expect("formatting context is never empty")
    }

    fn emit_end_block(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "END".to_string(),
        });
    }

    fn render_heredocs(&mut self, skip: bool) {
        while !self.heredoc_strings.is_empty() {
            let mut next_heredoc = self.heredoc_strings.pop().expect("we checked it's there");
            let want_newline = !self.last_token_is_a_newline();
            if want_newline {
                self.push_concrete_token(ConcreteLineToken::HardNewLine);
            }

            if let Some(b'\n') = next_heredoc.buf.last() {
                next_heredoc.buf.pop();
            };

            if let Some(b'\n') = next_heredoc.buf.last() {
                next_heredoc.buf.pop();
            };

            self.push_concrete_token(ConcreteLineToken::DirectPart {
                part: String::from_utf8(next_heredoc.buf).expect("hereoc is utf8"),
            });
            self.emit_newline();
            if next_heredoc.squiggly {
                self.emit_indent();
            } else {
                self.push_concrete_token(ConcreteLineToken::Indent { depth: 0 });
            }
            self.emit_ident(next_heredoc.symbol.replace("'", ""));
            if !skip {
                self.emit_newline();
            }
        }
    }

    fn is_absorbing_indents(&self) -> bool {
        self.absorbing_indents >= 1
    }

    fn emit_def_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DefKeyword);
    }

    fn emit_keyword(&mut self, kw: String) {
        self.push_concrete_token(ConcreteLineToken::Keyword { keyword: kw });
    }

    fn emit_mod_keyword(&mut self, contents: String) {
        self.push_concrete_token(ConcreteLineToken::ModKeyword { contents });
    }

    fn emit_conditional_keyword(&mut self, contents: String) {
        self.push_concrete_token(ConcreteLineToken::ConditionalKeyword { contents });
    }
}

impl BaseParserState {
    pub fn new(fc: FileComments) -> Self {
        BaseParserState {
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

    fn last_breakable_is_multiline(&self) -> bool {
        self.breakable_entry_stack
            .last()
            .map(|o| o.is_multiline())
            .unwrap_or(false)
    }

    fn push_comments(&mut self, comments: Option<CommentBlock>) {
        match comments {
            None => {}
            Some(comments) => {
                if !self
                    .surpress_comments_stack
                    .last()
                    .expect("comments stack is never empty")
                {
                    let len = comments.len();
                    let trailing_comment = comments.is_trailing();
                    self.insert_comment_collection(comments);
                    if !trailing_comment {
                        self.current_orig_line_number += len as u64;
                    }
                }
            }
        }
    }

    fn insert_extra_newline_at_last_newline(&mut self) {
        let idx = self.index_of_prev_hard_newline();
        let insert_idx = match idx {
            Some(idx) => idx,
            None => 0,
        };

        self.insert_concrete_tokens(insert_idx, vec![ConcreteLineToken::HardNewLine]);
    }

    fn current_spaces(&self) -> ColNumber {
        2 * self
            .depth_stack
            .last()
            .expect("depth stack is never empty")
            .get()
    }

    fn last_token_is_a_newline(&self) -> bool {
        match self.breakable_entry_stack.last() {
            Some(be) => be.last_token_is_a_newline(),
            None => self.render_queue.last_token_is_a_newline(),
        }
    }

    pub fn index_of_prev_hard_newline(&self) -> Option<usize> {
        match self.breakable_entry_stack.last() {
            Some(be) => be.index_of_prev_newline(),
            None => self.render_queue.index_of_prev_newline(),
        }
    }

    fn new_with_depth_stack_from(ps: &BaseParserState) -> Self {
        let mut next_ps = BaseParserState::new(FileComments::default());
        next_ps.depth_stack = ps.depth_stack.clone();
        next_ps.current_orig_line_number = ps.current_orig_line_number;
        next_ps
    }

    fn render_to_buffer(self) -> Vec<u8> {
        let mut bufio = Cursor::new(Vec::new());
        self.write(&mut bufio).expect("in memory io cannot fail");
        bufio.set_position(0);
        bufio.into_inner()
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
                let len = comments.len();
                let lts = comments.into_line_tokens();
                for comment in lts.into_iter() {
                    self.push_concrete_token(comment);
                }
                self.current_orig_line_number = len as LineNumber;
                debug!("rq: {:?}", self.render_queue);
            }
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

    fn push_concrete_token(&mut self, t: ConcreteLineToken) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(AbstractLineToken::ConcreteLineToken(t)),
            None => self
                .render_queue
                .push(ConcreteLineTokenAndTargets::ConcreteLineToken(t)),
        }
    }

    fn push_target(&mut self, t: ConcreteLineTokenAndTargets) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(t.into()),
            None => self.render_queue.push(t),
        }
    }

    fn push_abstract_token(&mut self, t: AbstractLineToken) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(t),
            None => self.render_queue.push(Self::dangerously_convert(t)),
        }
    }

    fn render_with_blank_state<F>(ps: &mut BaseParserState, f: F) -> BaseParserState
    where
        F: FnOnce(&mut BaseParserState),
    {
        let mut next_ps = BaseParserState::new_with_depth_stack_from(ps);
        f(&mut next_ps);
        next_ps
    }
}
