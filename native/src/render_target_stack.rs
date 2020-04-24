use crate::breakable_entry::BreakableEntry;
use crate::comment_block::CommentBlock;
use crate::line_token_collection::LineTokenCollection;
use crate::line_tokens::LineToken;
use crate::types::LineNumber;
use crate::parser_state::ParserState;

#[derive(Debug)]
pub enum RenderTarget {
    BaseQueue(LineTokenCollection),
    BreakableEntry(BreakableEntry),
}

impl RenderTarget {
    fn insert_extra_newline_at_last_newline(&mut self) {
        match self {
            Self::BaseQueue(ltc) => ltc.insert_extra_newline_at_last_newline(),
            Self::BreakableEntry(be) => be.insert_extra_newline_at_last_newline(),
        }
    }

    fn last_token_is_a_newline(&self) -> bool {
        match self {
            Self::BaseQueue(ltc) => ltc.last_token_is_a_newline(),
            Self::BreakableEntry(be) => be.last_token_is_a_newline(),
        }
    }

    fn insert_comments_at_last_hard_newline(&mut self, comments: CommentBlock) {
        match self {
            RenderTarget::BaseQueue(rq) => rq.insert_comments_at_last_hard_newline(comments),
            RenderTarget::BreakableEntry(be) => be.insert_comments_at_last_hard_newline(comments),
        }
    }

    fn push_token(&mut self, lt: LineToken) {
        match self {
            RenderTarget::BaseQueue(rq) => rq.push(lt),
            RenderTarget::BreakableEntry(be) => be.push(lt),
        }
    }
}

pub struct RenderTargetStack {
    target_stack: Vec<RenderTarget>,
}

impl RenderTargetStack {
    pub fn new() -> Self {
        RenderTargetStack {
            target_stack: vec![RenderTarget::BaseQueue(LineTokenCollection::new())],
        }
    }

    pub fn push_token(&mut self, lt: LineToken) {
        self.current_target_mut().push_token(lt);
    }

    pub fn insert_comments_at_last_hard_newline(&mut self, comments: CommentBlock) {
        self.current_target_mut()
            .insert_comments_at_last_hard_newline(comments);
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.current_target().last_token_is_a_newline()
    }

    pub fn insert_extra_newline_at_last_newline(&mut self) {
        self.current_target_mut()
            .insert_extra_newline_at_last_newline();
    }

    pub fn push_breakable_entry(&mut self, be: BreakableEntry) {
        self.target_stack.push(RenderTarget::BreakableEntry(be));
    }

    pub fn with_breakable<F>(&mut self, ps: &mut ParserState, be: BreakableEntry, f: F)
    where F: FnOnce(&mut ParserState) {
        self.push_breakable_entry(be);
        f(ps);
        let be = self.pop_expecting_breakable_entry();
        self.push_token(LineToken::BreakableEntry(be));
    }
    pub fn pop_expecting_breakable_entry(&mut self) -> BreakableEntry {
        if let RenderTarget::BreakableEntry(be) =
            self.target_stack.pop().expect("should be present")
        {
            return be;
        }
        panic!("should be impossible")
    }

    pub fn record_line_number(&mut self, ln: LineNumber) {
        for item in self.target_stack.iter_mut() {
            if let RenderTarget::BreakableEntry(be) = item {
                be.push_line_number(ln)
            }
        }
    }

    pub fn current_breakable_is_multiline(&self) -> Option<bool> {
        match self.current_target() {
            RenderTarget::BreakableEntry(be) => Some(be.is_multiline()),
            _ => None,
        }
    }

    pub fn into_render_queue(self) -> LineTokenCollection {
        if self.target_stack.len() != 1 {
            panic!("target stack was consumed when not length 1, indicates programming error");
        }

        let mut ts = self.target_stack;

        match ts.pop().expect("we checked it exists") {
            RenderTarget::BaseQueue(x) => x,
            RenderTarget::BreakableEntry(_) => {
                panic!("should not have breakable entry at bottom of stack")
            }
        }
    }

    pub fn render_queue(&self) -> &LineTokenCollection {
        match self.target_stack.get(0).expect("it's never empty") {
            RenderTarget::BaseQueue(rq) => rq,
            RenderTarget::BreakableEntry(..) => panic!("prgoramming error on get render queue"),
        }
    }

    fn current_target(&self) -> &RenderTarget {
        self.target_stack
            .last()
            .expect("target stack is never empty")
    }

    fn current_target_mut(&mut self) -> &mut RenderTarget {
        self.target_stack
            .last_mut()
            .expect("target stack is never empty")
    }
}
