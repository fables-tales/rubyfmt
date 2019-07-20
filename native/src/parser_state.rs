use crate::comment_block::CommentBlock;
use crate::line_metadata::LineMetadata;
use crate::line_tokens::*;
use crate::types::{FormatStatus, LineNumber};

#[derive(Clone)]
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
    depth: u16,
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

    fn get(&self) -> u16 {
        self.depth
    }
}

pub struct ParserState {
    depth_stack: Vec<IndentDepth>,
    start_of_line: Vec<bool>,
    surpress_comments_stack: Vec<bool>,
    surpress_one_paren: bool,
    render_queue: Vec<Box<LineToken>>,
    current_orig_line_number: LineNumber,
    comments_hash: LineMetadata,
    heredoc_strings: Vec<String>,
    string_concat_position: Vec<i32>,
    comments_to_insert: CommentBlock,
    breakable_state_stack: Vec<BreakableState>,
    formatting_context: Vec<FormattingContext>,
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
            string_concat_position: vec![],
            comments_to_insert: CommentBlock::new(vec![]),
            breakable_state_stack: vec![],
            formatting_context: vec![FormattingContext::Main],
        }
    }

    pub fn on_line(&mut self, line_number: LineNumber) {
        let mut comments = self.comments_hash.extract_comments_to_line(line_number);
        if !self
            .surpress_comments_stack
            .last()
            .expect("comments stack is never empty")
        {
            self.insert_comment_collection(comments)
        }
        self.current_orig_line_number = line_number;
    }

    pub fn insert_comment_collection(&mut self, comments: CommentBlock) {
        self.comments_to_insert.merge(comments);
    }

    pub fn emit_indent(&mut self) {
        self.render_queue.push(Box::new(Indent::new(
            self.depth_stack
                .last()
                .expect("depth stack is never empty")
                .get(),
        )));
    }

    pub fn emit_def_keyword(&mut self) {
        self.render_queue.push(Box::new(Keyword::new("def".into())));
    }

    pub fn emit_def(&mut self, def_name: String) {
        self.emit_def_keyword();
        self.render_queue.push(Box::new(DirectPart::new(def_name)));
    }

    pub fn emit_newline(&mut self) {
        self.render_queue.push(Box::new(HardNewLine::new()));
    }

    pub fn emit_end(&mut self) {
        self.render_queue.push(Box::new(Keyword::new("end".into())));
    }

    pub fn with_formatting_context<F>(
        &mut self,
        fc: FormattingContext,
        f: F,
    ) -> Result<(), FormatStatus>
    where
        F: FnOnce(&mut ParserState) -> Result<(), FormatStatus>,
    {
        self.formatting_context.push(fc);
        let res = f(self);
        self.formatting_context.pop();
        res
    }

    pub fn new_block<F>(&mut self, f: F) -> Result<(), FormatStatus>
    where
        F: FnOnce(&mut ParserState) -> Result<(), FormatStatus>,
    {
        let ds_length = self.depth_stack.len();
        self.depth_stack[ds_length - 1].increment();
        let res = f(self);
        self.depth_stack[ds_length - 1].decrement();
        res
    }

    pub fn with_start_of_line<F>(&mut self, start_of_line: bool, f: F) -> Result<(), FormatStatus>
    where
        F: FnOnce(&mut ParserState) -> Result<(), FormatStatus>,
    {
        self.start_of_line.push(start_of_line);
        let res = f(self);
        self.start_of_line.pop();
        res
    }


    pub fn at_start_of_line(&self) -> bool {
        self.start_of_line.last().expect("start of line is never_empty").clone()
    }

    pub fn current_formatting_context(&self) -> FormattingContext {
        self.formatting_context.last().expect("formatting context is never empty").clone()
    }

}
