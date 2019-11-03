use crate::comment_block::CommentBlock;
use crate::line_metadata::LineMetadata;
use crate::line_tokens::*;
use crate::types::{ColNumber, FormatStatus, LineNumber};

#[derive(Clone, Debug, PartialEq)]
pub enum FormattingContext {
    Main,
    Assign,
    Binary,
    ClassOrModule,
    Def,
    CurlyBlock,
    ArgsList,
}

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

    fn get(&self) -> ColNumber {
        self.depth
    }
}

pub struct ParserState {
    depth_stack: Vec<IndentDepth>,
    start_of_line: Vec<bool>,
    surpress_comments_stack: Vec<bool>,
    surpress_one_paren: bool,
    render_queue: Vec<Box<dyn LineToken>>,
    current_orig_line_number: LineNumber,
    comments_hash: LineMetadata,
    heredoc_strings: Vec<String>,
    comments_to_insert: CommentBlock,
    breakable_entry_stack: Vec<BreakableEntry>,
    next_breakable_entry_id: u32,
    formatting_context: Vec<FormattingContext>,
    absorbing_indents: bool,
}

impl ParserState {
    pub fn new(lm: LineMetadata) -> Self {
        ParserState {
            depth_stack: vec![IndentDepth::new()],
            start_of_line: vec![true],
            surpress_comments_stack: vec![false],
            surpress_one_paren: false,
            render_queue: vec![],
            current_orig_line_number: 0,
            comments_hash: lm,
            heredoc_strings: vec![],
            comments_to_insert: CommentBlock::new(vec![]),
            breakable_entry_stack: vec![],
            next_breakable_entry_id: 0,
            formatting_context: vec![FormattingContext::Main],
            absorbing_indents: false,
        }
    }

    pub fn consume_to_render_queue(self) -> Vec<Box<dyn LineToken>> {
        self.render_queue
    }

    pub fn on_line(&mut self, line_number: LineNumber) {
        let comments = self.comments_hash.extract_comments_to_line(line_number);
        if comments.is_none() {
            return;
        }

        if !self
            .surpress_comments_stack
            .last()
            .expect("comments stack is never empty")
        {
            self.insert_comment_collection(
                comments.expect("we checked it was none at the top of the function"),
            )
        }
        self.current_orig_line_number = line_number;
    }

    pub fn insert_comment_collection(&mut self, comments: CommentBlock) {
        self.comments_to_insert.merge(comments);
    }

    pub fn emit_indent(&mut self) {
        self.push_token(Indent::new(self.current_spaces()));
    }

    pub fn emit_op(&mut self, op: String) {
        self.push_token(Op::new(op));
    }

    pub fn emit_double_quote(&mut self) {
        self.push_token(DoubleQuote::new());
    }

    pub fn emit_string_content(&mut self, s: String) {
        self.push_token(LTStringContent::new(s));
    }

    fn current_spaces(&self) -> ColNumber {
        2 * self
            .depth_stack
            .last()
            .expect("depth stack is never empty")
            .get()
    }

    pub fn emit_ident(&mut self, ident: String) {
        self.push_token(DirectPart::new(ident));
    }

    pub fn emit_def_keyword(&mut self) {
        self.push_token(Keyword::new("def".to_string()));
    }

    pub fn emit_rescue(&mut self) {
        self.push_token(Keyword::new("rescue".to_string()));
    }

    pub fn emit_begin(&mut self) {
        self.push_token(Keyword::new("begin".to_string()));
    }

    pub fn emit_soft_indent(&mut self) {
        self.push_token(SoftIndent::new(self.current_spaces()));
    }

    pub fn emit_comma(&mut self) {
        self.push_token(Comma::new());
    }

    pub fn emit_soft_newline(&mut self) {
        self.push_token(SoftNewline::new());
    }

    pub fn emit_collapsing_newline(&mut self) {
        self.push_token(CollapsingNewLine::new());
    }

    pub fn emit_def(&mut self, def_name: String) {
        self.emit_def_keyword();
        self.push_token(DirectPart::new(format!(" {}", def_name)));
    }

    pub fn emit_int(&mut self, int: String) {
        self.push_token(DirectPart::new(int));
    }

    pub fn emit_newline(&mut self) {
        self.push_token(HardNewLine::new());
    }

    pub fn emit_end(&mut self) {
        self.emit_newline();
        if self.at_start_of_line() {
            self.emit_indent();
        }
        self.push_token(Keyword::new("end".into()));
    }

    pub fn emit_comma_space(&mut self) {
        self.push_token(CommaSpace::new())
    }

    pub fn emit_space(&mut self) {
        self.push_token(Space::new());
    }

    pub fn emit_dot(&mut self) {
        self.push_token(Dot::new());
    }

    pub fn emit_colon_colon(&mut self) {
        self.push_token(ColonColon::new());
    }

    pub fn emit_lonely_operator(&mut self) {
        self.push_token(LonelyOperator::new());
    }

    pub fn with_formatting_context<F>(&mut self, fc: FormattingContext, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.formatting_context.push(fc);
        let res = f(self);
        self.formatting_context.pop();
        res
    }

    pub fn with_absorbing_indent_block<F>(&mut self, f: F)
        where F: FnOnce(&mut ParserState),
    {
        let was_absorbing = self.absorbing_indents;
        self.absorbing_indents = true;
        if was_absorbing {
            f(self);
        } else {
            self.new_block(f);
        }
        self.absorbing_indents = false;
    }

    pub fn new_block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
        let res = f(self);
        self.depth_stack[ds_length - 1].decrement();
        res
    }

    pub fn with_start_of_line<F>(&mut self, start_of_line: bool, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.start_of_line.push(start_of_line);
        let res = f(self);
        self.start_of_line.pop();
        res
    }

    pub fn at_start_of_line(&self) -> bool {
        self.start_of_line
            .last()
            .expect("start of line is never_empty")
            .clone()
    }

    pub fn current_formatting_context(&self) -> FormattingContext {
        self.formatting_context
            .last()
            .expect("formatting context is never empty")
            .clone()
    }

    pub fn breakable_of<F>(&mut self, start_delim: String, end_delim: String, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.emit_ident(start_delim);
        let breakable_state = BreakableState::new(self.current_spaces());
        self.push_token(breakable_state);

        self.new_block(|ps| f(ps));

        self.emit_soft_indent();
        self.push_token(breakable_state);
        self.emit_ident(end_delim);
    }

    pub fn breakable_entry<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ParserState),
    {
        self.next_breakable_entry_id += 1;
        let be = BreakableEntry::new(self.next_breakable_entry_id);
        self.breakable_entry_stack.push(be);
        f(self);
        let insert_be = self
            .breakable_entry_stack
            .pop()
            .expect("cannot have empty here because we just pushed");
        self.push_token(insert_be);
    }

    pub fn emit_open_square_bracket(&mut self) {
        self.push_token(OpenSquareBracket::new());
    }

    pub fn emit_close_square_bracket(&mut self) {
        self.push_token(CloseSquareBracket::new());
    }

    pub fn emit_slash(&mut self) {
        self.push_token(SingleSlash::new());
    }

    pub fn emit_open_paren(&mut self) {
        self.push_token(OpenParen::new());
    }

    pub fn emit_close_paren(&mut self) {
        self.push_token(CloseParen::new());
    }

    pub fn push_token<T: 'static + LineToken>(&mut self, t: T) {
        if self.breakable_entry_stack.is_empty() {
            self.render_queue.push(Box::new(t));
        } else {
            self.breakable_entry_stack
                .last_mut()
                .expect("we checked it wasn't empty")
                .push(t);
        }
    }
}
