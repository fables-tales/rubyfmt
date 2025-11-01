use crate::comment_block::{CommentBlock, Merge};
use crate::delimiters::BreakableDelims;
use crate::file_comments::FileComments;
use crate::heredoc_string::{HeredocKind, HeredocString};
use crate::line_tokens::*;
use crate::render_queue_writer::{MAX_LINE_LENGTH, RenderQueueWriter};
use crate::render_targets::{
    AbstractTokenTarget, BaseQueue, BreakableCallChainEntry, BreakableEntry, MultilineHandling,
};
use crate::types::{ColNumber, LineNumber, SourceOffset};
use log::debug;
use std::io::{self, Cursor, Write};
use std::str;

pub type RenderFunc<'a> = Box<dyn FnOnce(&mut BaseParserState) + 'a>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormattingContext {
    Main,
    Assign,
    Binary,
    ClassOrModule,
    Def,
    ArgsList,
    IfOp,
    StringEmbexpr,
    HashType(HashType),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashType {
    SymbolKey,
    HashRocket,
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

    fn get(self) -> u32 {
        self.depth
    }
}

#[derive(Debug)]
pub struct BaseParserState {
    depth_stack: Vec<IndentDepth>,
    start_of_line: Vec<bool>,
    suppress_comments_stack: Vec<bool>,
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
    scopes: Vec<Vec<String>>,
}

impl BaseParserState {
    pub(crate) fn scope_has_variable(&self, s: &str) -> bool {
        self.scopes
            .last()
            .expect("it's never empty")
            .contains(&s.to_string())
    }
    pub(crate) fn new_scope<'a>(&mut self, f: RenderFunc) {
        self.scopes.push(vec![]);
        f(self);
        self.scopes.pop();
    }
    pub(crate) fn bind_variable(&mut self, s: String) {
        self.scopes.last_mut().expect("it's never empty").push(s);
    }
    pub(crate) fn push_heredoc_content<'a>(
        &mut self,
        symbol: String,
        kind: HeredocKind,
        end_line: LineNumber,
        formatting_func: Box<dyn FnOnce(&mut BaseParserState) + 'a>,
    ) {
        let mut next_ps = BaseParserState::render_with_blank_state(self, formatting_func);

        for hs in next_ps.heredoc_strings.drain(0..) {
            self.heredoc_strings.push(hs);
        }

        // Update line number and clear out any comments we might have rendered in e.g. an embexpr
        //
        // (Ignore this comment extraction, we've already rendered them elsewhere)
        let _ = self
            .comments_hash
            .extract_comments_to_line(self.current_orig_line_number, end_line);
        self.current_orig_line_number = end_line;

        let data = next_ps.render_to_buffer();
        self.heredoc_strings.push(HeredocString::new(
            symbol,
            kind,
            data,
            self.current_spaces(),
        ));
    }

    pub(crate) fn emit_heredoc_start(&mut self, symbol: String, kind: HeredocKind) {
        self.push_concrete_token(ConcreteLineToken::HeredocStart { kind, symbol });
    }

    pub(crate) fn emit_heredoc_close(&mut self, symbol: String) {
        self.push_concrete_token(ConcreteLineToken::HeredocClose { symbol });
    }

    pub(crate) fn magic_handle_comments_for_multiline_arrays<'a>(
        &mut self,
        end_line: Option<LineNumber>,
        f: RenderFunc,
    ) {
        let current_line_number = self.current_orig_line_number;
        self.new_block(Box::new(|ps| {
            ps.shift_comments();
        }));
        f(self);
        // Reset here -- this resets when we emit newlines, but this may be out of date
        // if the most recent array didn't emit a newline
        self.reset_space_count();
        let new_line_number = self.current_orig_line_number;
        if new_line_number > current_line_number {
            // Only wind forward if the next line is empty or a comment
            if self.comments_hash.is_empty_line(new_line_number + 1) {
                if let Some(end_line) = end_line {
                    self.wind_dumping_comments_until_line(end_line);
                }
                if self
                    .comments_to_insert
                    .as_ref()
                    .map(|comments| comments.has_comments())
                    .unwrap_or(false)
                    .to_owned()
                {
                    self.push_concrete_token(ConcreteLineToken::HardNewLine);
                }
                let index = if let Some(be) = self.breakable_entry_stack.last() {
                    be.len()
                } else {
                    self.index_of_prev_hard_newline().unwrap_or(0)
                };
                self.shift_comments_at_index(index);
            } else {
                self.shift_comments();
            }
        }
    }

    pub(crate) fn will_render_as_multiline<'a>(&mut self, f: RenderFunc) -> bool {
        let mut next_ps = BaseParserState::new_with_depth_stack_from(self);
        // Ignore commments when determining line length
        next_ps.with_suppress_comments(true, f);
        let data = next_ps.render_to_buffer();

        let s = str::from_utf8(&data).expect("string is utf8").to_string();
        s.trim().contains('\n') || s.len() > MAX_LINE_LENGTH
    }

    pub(crate) fn has_comment_in_offset_span(
        &self,
        start_offset: SourceOffset,
        end_offset: SourceOffset,
    ) -> bool {
        self.comments_hash
            .has_comment_in_offsets(start_offset, end_offset)
    }

    pub(crate) fn will_render_beyond_max_line_length<'a>(&mut self, f: RenderFunc) -> bool {
        let mut next_ps = BaseParserState::new_with_depth_stack_from(self);
        // Ignore commments when determining line length
        next_ps.with_suppress_comments(true, f);
        let data = next_ps.render_to_buffer();

        let s = str::from_utf8(&data).expect("string is utf8").to_string();

        // Add current spaces to account for current indentation level
        (s.split_whitespace().collect::<String>().len() + (self.current_spaces() as usize))
            > MAX_LINE_LENGTH
    }

    pub(crate) fn reset_space_count(&mut self) {
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub(crate) fn dedent<'a>(&mut self, f: RenderFunc) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].decrement();
        f(self);
        self.depth_stack[ds_length - 1].increment();
    }

    pub(crate) fn start_indent(&mut self) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
    }

    pub(crate) fn start_indent_for_call_chain(&mut self) {
        self.push_concrete_token(ConcreteLineToken::BeginCallChainIndent)
    }

    pub(crate) fn end_indent_for_call_chain(&mut self) {
        self.push_concrete_token(ConcreteLineToken::EndCallChainIndent)
    }

    pub(crate) fn end_indent(&mut self) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].decrement();
    }

    pub(crate) fn with_start_of_line<'a>(&mut self, start_of_line: bool, f: RenderFunc) {
        self.start_of_line.push(start_of_line);
        f(self);
        self.start_of_line.pop();
    }

    pub(crate) fn breakable_of<'a>(&mut self, delims: BreakableDelims, f: RenderFunc) {
        self.shift_comments();
        let mut be = BreakableEntry::new(delims, self.formatting_context.clone());
        be.push_line_number(self.current_orig_line_number);
        self.breakable_entry_stack.push(Box::new(be));

        self.new_block(Box::new(|ps| {
            ps.emit_collapsing_newline();
            f(ps);
            ps.emit_collapsing_newline();
        }));

        // The last newline is in the old block, so we need
        // to reset to ensure that any comments between now and the
        // next newline are at the right indentation level
        self.reset_space_count();
        // Make sure we shift comments before ending the breakable.
        // This ensures that comments inside breakables never end up rendering
        // outside their delimiters.
        self.shift_comments();
        self.emit_soft_indent();

        let insert_be = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed")
            .to_breakable_entry()
            .expect("This should be the BreakableEntry we just pushed");
        self.push_target(ConcreteLineTokenAndTargets::BreakableEntry(insert_be));
    }

    /// A version of `breakable_of` for list-like things that use whitespace delimiters.
    /// At the moment, this is only for conditions in a `when` clause
    pub(crate) fn inline_breakable_of<'a>(&mut self, delims: BreakableDelims, f: RenderFunc) {
        self.shift_comments();
        let mut be = BreakableEntry::new(delims, self.formatting_context.clone());
        be.push_line_number(self.current_orig_line_number);
        self.breakable_entry_stack.push(Box::new(be));

        self.new_block(Box::new(|ps| {
            f(ps);
        }));

        // The last newline is in the old block, so we need
        // to reset to ensure that any comments between now and the
        // next newline are at the right indentation level
        self.reset_space_count();
        self.shift_comments();

        let insert_be = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed")
            .to_breakable_entry()
            .expect("This should be the BreakableEntry we just pushed");
        self.push_target(ConcreteLineTokenAndTargets::BreakableEntry(insert_be));
    }

    pub(crate) fn breakable_call_chain_of<'a>(
        &mut self,
        mulitiline_handling: MultilineHandling,
        f: RenderFunc,
    ) {
        self.shift_comments();
        let mut be =
            BreakableCallChainEntry::new(self.formatting_context.clone(), mulitiline_handling);
        be.push_line_number(self.current_orig_line_number);
        self.breakable_entry_stack.push(Box::new(be));

        f(self);

        let insert_bcce = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed")
            .to_breakable_call_chain()
            .expect("This should be the BreakableCallChainEntry we just pushed");
        self.push_target(ConcreteLineTokenAndTargets::BreakableCallChainEntry(
            insert_bcce,
        ));
    }

    pub(crate) fn with_suppress_comments<'a>(&mut self, suppress: bool, f: RenderFunc) {
        self.suppress_comments_stack.push(suppress);
        f(self);
        self.suppress_comments_stack.pop();
    }

    pub(crate) fn with_absorbing_indent_block<'a>(&mut self, f: RenderFunc) {
        let was_absorbing = self.absorbing_indents != 0;
        self.absorbing_indents += 1;
        if was_absorbing {
            f(self);
        } else {
            self.new_block(f);
        }
        self.absorbing_indents -= 1;
    }

    pub(crate) fn new_block<'a>(&mut self, f: RenderFunc) {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
        f(self);
        self.depth_stack[ds_length - 1].decrement();
    }

    pub(crate) fn with_formatting_context<'a>(&mut self, fc: FormattingContext, f: RenderFunc) {
        self.formatting_context.push(fc);
        f(self);
        self.formatting_context.pop();
    }

    pub(crate) fn on_line(&mut self, line_number: LineNumber) {
        if line_number < self.current_orig_line_number {
            return;
        }
        debug!("on_line called: {}", line_number);

        for be in self.breakable_entry_stack.iter_mut().rev() {
            be.push_line_number(line_number);
        }

        if let Some((comments, last_comment_line)) = self
            .comments_hash
            .extract_comments_to_line(self.current_orig_line_number, line_number)
        {
            self.push_comments(comments);
            self.current_orig_line_number =
                std::cmp::max(self.current_orig_line_number, last_comment_line);
        }

        debug!("lns: {} {}", line_number, self.current_orig_line_number);
        if line_difference_requires_newline(line_number, self.current_orig_line_number)
            && self.insert_user_newlines
        {
            self.insert_extra_newline_at_last_newline();
        }

        self.current_orig_line_number = line_number;
    }

    pub(crate) fn at_offset(&mut self, source_offset: SourceOffset) {
        self.on_line(self.get_line_number_for_offset(source_offset));
    }

    pub(crate) fn emit_indent(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Indent {
            depth: self.current_spaces(),
        });
    }

    pub(crate) fn at_start_of_line(&self) -> bool {
        *self
            .start_of_line
            .last()
            .expect("start of line is never_empty")
    }

    pub(crate) fn has_comments_in_line(&self, start_line: LineNumber, end_line: LineNumber) -> bool {
        self.comments_hash
            .has_comments_in_lines(start_line, end_line)
    }

    pub(crate) fn current_line_number(&self) -> u64 {
        self.current_orig_line_number
    }

    pub(crate) fn emit_def(&mut self, def_name: String) {
        self.emit_def_keyword();
        self.push_concrete_token(ConcreteLineToken::DirectPart {
            part: format!(" {}", def_name),
        });
    }

    pub(crate) fn insert_comment_collection(&mut self, comments: CommentBlock) {
        self.comments_to_insert
            .merge(comments.apply_spaces(self.spaces_after_last_newline));
    }

    pub(crate) fn emit_op(&mut self, op: String) {
        self.push_concrete_token(ConcreteLineToken::Op { op });
    }

    pub(crate) fn emit_double_quote(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DoubleQuote);
    }

    pub(crate) fn emit_string_content(&mut self, s: String) {
        let newline_count = s.matches('\n').count() as u64;
        self.current_orig_line_number += newline_count;

        self.push_concrete_token(ConcreteLineToken::LTStringContent { content: s });
    }

    pub(crate) fn emit_ident(&mut self, ident: String) {
        self.push_concrete_token(ConcreteLineToken::DirectPart { part: ident });
    }

    pub(crate) fn emit_newline(&mut self) {
        self.shift_comments();
        self.push_concrete_token(ConcreteLineToken::HardNewLine);
        self.render_heredocs(false);
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub(crate) fn wind_dumping_comments_until_line(&mut self, line_number: LineNumber) {
        self.wind_dumping_comments(Some(line_number))
    }

    pub(crate) fn wind_dumping_comments_until_offset(&mut self, source_offset: SourceOffset) {
        self.wind_dumping_comments_until_line(self.get_line_number_for_offset(source_offset))
    }

    pub(crate) fn wind_dumping_comments(&mut self, maybe_max_line_number: Option<LineNumber>) {
        // Return early if we're already at/past
        // the max line number
        if maybe_max_line_number
            .map(|ln| ln <= self.current_orig_line_number)
            .unwrap_or(false)
        {
            return;
        }

        self.on_line(self.current_orig_line_number + 1);
        let should_iter = |ps: &BaseParserState, ln| {
            // If we have a max line number, it will be the last token
            // of an expression (e.g. the `end` of a `do`/`end` block), so it's
            // fine if we wind forward to that line
            if maybe_max_line_number
                .map(|max| ln + 1 == max)
                .unwrap_or(false)
            {
                return true;
            }

            ps.comments_hash.still_in_file(ln + 1)
                && (ps.comments_hash.has_line(ln + 1) || ps.comments_hash.is_empty_line(ln + 1))
                && maybe_max_line_number
                    .map(|max_line| ln + 1 < max_line)
                    .unwrap_or(true)
        };
        while should_iter(self, self.current_orig_line_number) {
            if !self
                .comments_hash
                .has_line(self.current_orig_line_number + 1)
                && self
                    .comments_hash
                    .is_empty_line(self.current_orig_line_number + 1)
                && self.comments_to_insert.is_some()
            {
                let mr = self.comments_to_insert.as_mut().expect("it's not nil");
                if mr.len() == 0 {
                    break;
                }
                mr.add_line("".to_string());
            }
            self.on_line(self.current_orig_line_number + 1);
        }
    }

    pub(crate) fn emit_end(&mut self) {
        if !self.last_token_is_a_newline() {
            self.emit_newline();
        }
        if self.at_start_of_line() {
            self.emit_indent();
        }
        self.push_concrete_token(ConcreteLineToken::End);
    }

    pub(crate) fn emit_comma(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Comma);
    }

    pub(crate) fn shift_comments(&mut self) {
        let idx_of_prev_hard_newline = self.index_of_prev_hard_newline();
        self.shift_comments_at_index(idx_of_prev_hard_newline.unwrap_or(0));
    }

    pub(crate) fn shift_comments_at_index(&mut self, index: usize) {
        if let Some(new_comments) = self.comments_to_insert.take() {
            self.insert_concrete_tokens(index, new_comments.into_line_tokens());
        }
    }

    pub(crate) fn emit_soft_newline(&mut self) {
        self.new_block(Box::new(|ps| {
            ps.shift_comments();
        }));
        let hd = self.gather_heredocs();
        self.push_abstract_token(AbstractLineToken::SoftNewline(hd));
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub(crate) fn emit_soft_indent(&mut self) {
        self.push_abstract_token(AbstractLineToken::SoftIndent {
            depth: self.current_spaces(),
        });
    }

    pub(crate) fn emit_collapsing_newline(&mut self) {
        if !self.last_token_is_a_newline() {
            let hd = self.gather_heredocs();
            self.push_abstract_token(AbstractLineToken::CollapsingNewLine(hd));
        }
        self.spaces_after_last_newline = self.current_spaces();
    }

    pub(crate) fn emit_after_call_chain(&mut self) {
        self.push_concrete_token(ConcreteLineToken::AfterCallChain)
    }

    pub(crate) fn emit_space(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Space);
    }

    pub(crate) fn emit_open_paren(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenParen);
    }

    pub(crate) fn emit_single_line_delims(&mut self, delims: BreakableDelims) {
        self.push_concrete_token(delims.single_line_open());
        self.push_concrete_token(delims.single_line_close());
    }

    pub(crate) fn emit_comma_space(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CommaSpace)
    }

    pub(crate) fn emit_close_paren(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseParen);
    }

    pub(crate) fn emit_slash(&mut self) {
        self.push_concrete_token(ConcreteLineToken::SingleSlash);
    }

    pub(crate) fn emit_close_curly_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseCurlyBracket);
    }

    pub(crate) fn emit_open_curly_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenCurlyBracket);
    }

    pub(crate) fn emit_close_square_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::CloseSquareBracket);
    }

    pub(crate) fn emit_open_square_bracket(&mut self) {
        self.push_concrete_token(ConcreteLineToken::OpenSquareBracket);
    }

    pub(crate) fn emit_rescue(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "rescue".to_string(),
        });
    }

    pub(crate) fn emit_case_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "case".to_string(),
        });
    }

    pub(crate) fn emit_when_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "when".to_string(),
        });
    }

    pub(crate) fn emit_do_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DoKeyword);
    }

    pub(crate) fn emit_class_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ClassKeyword);
    }

    pub(crate) fn emit_module_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ModuleKeyword);
    }

    pub(crate) fn emit_ensure(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "ensure".to_string(),
        });
    }

    pub(crate) fn emit_begin(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "begin".to_string(),
        });
    }

    pub(crate) fn emit_begin_block(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "BEGIN".to_string(),
        });
    }

    pub(crate) fn emit_else(&mut self) {
        self.emit_conditional_keyword("else".to_string());
    }

    pub(crate) fn emit_data_end(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DataEnd);
    }

    pub(crate) fn emit_data(&mut self, data: &str) {
        self.push_concrete_token(ConcreteLineToken::DirectPart {
            part: data.to_string(),
        })
    }

    pub(crate) fn wind_line_forward(&mut self) {
        self.on_line(self.current_orig_line_number + 1);
    }

    pub(crate) fn current_formatting_context_requires_parens(&self) -> bool {
        self.current_formatting_context() == FormattingContext::Binary
            || self.current_formatting_context() == FormattingContext::IfOp
    }

    pub(crate) fn hash_type_from_formatting_context(&self) -> Option<&HashType> {
        self.formatting_context
            .iter()
            .filter_map(|fc| match fc {
                FormattingContext::HashType(hash_type) => Some(hash_type),
                _ => None,
            })
            .last()
    }

    pub(crate) fn emit_dot(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Dot);
    }

    pub(crate) fn emit_ellipsis(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Ellipsis)
    }

    pub(crate) fn emit_lonely_operator(&mut self) {
        self.push_concrete_token(ConcreteLineToken::LonelyOperator);
    }

    pub(crate) fn emit_colon_colon(&mut self) {
        self.push_concrete_token(ConcreteLineToken::ColonColon);
    }

    pub(crate) fn current_formatting_context(&self) -> FormattingContext {
        *self
            .formatting_context
            .last()
            .expect("formatting context is never empty")
    }

    pub(crate) fn get_line_number_for_offset(&self, source_offset: SourceOffset) -> LineNumber {
        self.comments_hash.get_line_number_for_offset(source_offset)
    }

    pub(crate) fn emit_end_block(&mut self) {
        self.push_concrete_token(ConcreteLineToken::Keyword {
            keyword: "END".to_string(),
        });
    }

    pub(crate) fn render_heredocs(&mut self, skip: bool) {
        while let Some(next_heredoc) = self.heredoc_strings.pop() {
            let want_newline = !self.last_token_is_a_newline();
            if want_newline {
                self.push_concrete_token(ConcreteLineToken::HardNewLine);
            }

            let kind = next_heredoc.kind;
            let symbol = next_heredoc.closing_symbol();
            let space_count = next_heredoc.indent;
            let string_contents = next_heredoc.render_as_string();

            if !string_contents.is_empty() {
                self.push_concrete_token(ConcreteLineToken::DirectPart {
                    part: string_contents,
                });
                self.emit_newline();
            }
            if !kind.is_bare() {
                self.push_concrete_token(ConcreteLineToken::Indent { depth: space_count })
            } else {
                self.push_concrete_token(ConcreteLineToken::Indent { depth: 0 });
            }
            self.emit_heredoc_close(symbol.replace('\'', ""));
            if !skip {
                self.emit_newline();
            }
        }
    }

    pub(crate) fn is_absorbing_indents(&self) -> bool {
        self.absorbing_indents >= 1
    }

    pub(crate) fn emit_def_keyword(&mut self) {
        self.push_concrete_token(ConcreteLineToken::DefKeyword);
    }

    pub(crate) fn emit_keyword(&mut self, kw: String) {
        self.push_concrete_token(ConcreteLineToken::Keyword { keyword: kw });
    }

    pub(crate) fn emit_mod_keyword(&mut self, contents: String) {
        self.push_concrete_token(ConcreteLineToken::ModKeyword { contents });
    }

    pub(crate) fn emit_conditional_keyword(&mut self, contents: String) {
        self.push_concrete_token(ConcreteLineToken::ConditionalKeyword { contents });
    }
}

impl BaseParserState {
    pub(crate) fn new(fc: FileComments) -> Self {
        BaseParserState {
            depth_stack: vec![IndentDepth::new()],
            start_of_line: vec![true],
            suppress_comments_stack: vec![false],
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
            scopes: vec![vec![]],
        }
    }

    pub(crate) fn consume_to_render_queue(self) -> Vec<ConcreteLineTokenAndTargets> {
        self.render_queue.into_tokens()
    }

    pub(crate) fn gather_heredocs(&mut self) -> Option<Vec<HeredocString>> {
        if self.heredoc_strings.is_empty() {
            None
        } else {
            Some(self.heredoc_strings.drain(..).rev().collect())
        }
    }

    pub(crate) fn push_comments(&mut self, comments: CommentBlock) {
        if !self
            .suppress_comments_stack
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

    pub(crate) fn insert_extra_newline_at_last_newline(&mut self) {
        let idx = self.index_of_prev_hard_newline();
        let insert_idx = idx.unwrap_or(0);

        if self.breakable_entry_stack.last().is_some() {
            self.breakable_entry_stack.last_mut().unwrap().insert_at(
                insert_idx,
                &mut vec![AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::HardNewLine,
                )],
            );
        } else {
            self.insert_concrete_tokens(insert_idx, vec![ConcreteLineToken::HardNewLine]);
        }
    }

    pub(crate) fn current_spaces(&self) -> ColNumber {
        2 * self
            .depth_stack
            .last()
            .expect("depth stack is never empty")
            .get()
    }

    pub(crate) fn disable_user_newlines(&mut self) {
        self.insert_user_newlines = false;
    }

    pub(crate) fn last_token_is_a_newline(&self) -> bool {
        match self.breakable_entry_stack.last() {
            Some(be) => be.last_token_is_a_newline(),
            None => self.render_queue.last_token_is_a_newline(),
        }
    }

    pub (crate) fn index_of_prev_hard_newline(&self) -> Option<usize> {
        match self.breakable_entry_stack.last() {
            Some(be) => be.index_of_prev_newline(),
            None => self.render_queue.index_of_prev_newline(),
        }
    }

    pub(crate) fn new_with_depth_stack_from(ps: &BaseParserState) -> Self {
        let mut next_ps = BaseParserState::new_with_reset_depth_stack(ps);
        next_ps.depth_stack = ps.depth_stack.clone();
        next_ps
    }

    // Creates a copy of the parser state *with the depth_stack reset*.
    // This is used for heredocs, where we explicitly want to ignore current indentation.
    pub(crate) fn new_with_reset_depth_stack(ps: &BaseParserState) -> Self {
        let mut next_ps = BaseParserState::new(FileComments::default());
        next_ps.comments_hash = ps.comments_hash.clone();
        next_ps.start_of_line = ps.start_of_line.clone();
        next_ps.current_orig_line_number = ps.current_orig_line_number;
        next_ps
    }

    pub(crate) fn render_to_buffer(self) -> Vec<u8> {
        let mut bufio = Cursor::new(Vec::new());
        self.write(&mut bufio).expect("in memory io cannot fail");
        bufio.set_position(0);
        bufio.into_inner()
    }

    pub (crate) fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let rqw = RenderQueueWriter::new(self.consume_to_render_queue());
        rqw.write(writer)
    }

    pub(crate) fn dangerously_convert(t: AbstractLineToken) -> ConcreteLineTokenAndTargets {
        match t {
            AbstractLineToken::ConcreteLineToken(clt) => {
                ConcreteLineTokenAndTargets::ConcreteLineToken(clt)
            }
            AbstractLineToken::BreakableEntry(be) => {
                ConcreteLineTokenAndTargets::BreakableEntry(be)
            }
            AbstractLineToken::BreakableCallChainEntry(bcce) => {
                ConcreteLineTokenAndTargets::BreakableCallChainEntry(bcce)
            }
            _ => panic!("failed to convert"),
        }
    }

    pub (crate) fn flush_start_of_file_comments(&mut self) {
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
            }
        }
    }

    pub(crate) fn insert_concrete_tokens(&mut self, insert_idx: usize, clts: Vec<ConcreteLineToken>) {
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

    pub(crate) fn push_concrete_token(&mut self, t: ConcreteLineToken) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(AbstractLineToken::ConcreteLineToken(t)),
            None => self
                .render_queue
                .push(ConcreteLineTokenAndTargets::ConcreteLineToken(t)),
        }
    }

    pub(crate) fn push_target(&mut self, t: ConcreteLineTokenAndTargets) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(t.into()),
            None => self.render_queue.push(t),
        }
    }

    pub(crate) fn push_abstract_token(&mut self, t: AbstractLineToken) {
        match self.breakable_entry_stack.last_mut() {
            Some(be) => be.push(t),
            None => self.render_queue.push(Self::dangerously_convert(t)),
        }
    }

    pub(crate) fn render_with_blank_state<F>(ps: &mut BaseParserState, f: F) -> BaseParserState
    where
        F: FnOnce(&mut BaseParserState),
    {
        let mut next_ps = BaseParserState::new_with_reset_depth_stack(ps);
        f(&mut next_ps);
        next_ps
    }
}

pub fn line_difference_requires_newline(to_line: LineNumber, from_line: LineNumber) -> bool {
    (to_line > from_line) && (to_line - from_line >= 2)
}
