use crate::breakable_entry::BreakableEntry;
use crate::comment_block::CommentBlock;
use crate::line_token_collection::LineTokenCollection;
use crate::line_tokens::LineToken;

#[derive(Debug)]
pub enum RenderTarget {
    BaseQueue(LineTokenCollection),
    #[allow(dead_code)]
    BreakableEntry(BreakableEntry),
}

impl RenderTarget {
    fn insert_extra_newline_at_last_newline(&mut self) {
        match self {
            Self::BaseQueue(ltc) => ltc.insert_extra_newline_at_last_newline(),
            Self::BreakableEntry(_) => unimplemented!(),
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
            RenderTarget::BreakableEntry(..) => unimplemented!(),
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

    #[allow(dead_code)]
    pub fn current_breakable_is_multiline(&mut self) -> Option<bool> {
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
