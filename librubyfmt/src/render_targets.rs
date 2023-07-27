use crate::delimiters::BreakableDelims;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::FormattingContext;
use crate::ripper_tree_types::{
    ArgsAddStarOrExpressionListOrArgsForward, Block, CallChainElement, Dot, DotType, DotTypeOrOp,
    Expression, LonelyOperator, Op, Period, StartEnd, StringLiteral,
};
use crate::types::{ColNumber, LineNumber};
use std::collections::HashSet;

fn insert_at<T>(idx: usize, target: &mut Vec<T>, input: &mut Vec<T>) {
    let mut tail = target.split_off(idx);
    target.append(input);
    target.append(&mut tail);
}

#[derive(Copy, Clone, Debug)]
pub enum ConvertType {
    MultiLine,
    SingleLine,
}

#[derive(Debug, Default, Clone)]
pub struct BaseQueue {
    tokens: Vec<ConcreteLineTokenAndTargets>,
}

impl BaseQueue {
    pub fn push(&mut self, lt: ConcreteLineTokenAndTargets) {
        self.tokens.push(lt)
    }

    pub fn insert_at(&mut self, idx: usize, tokens: &mut Vec<ConcreteLineTokenAndTargets>) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn into_tokens(self) -> Vec<ConcreteLineTokenAndTargets> {
        self.tokens
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.tokens.last().map(|x| x.is_newline()).unwrap_or(false)
    }

    pub fn index_of_prev_newline(&self) -> Option<usize> {
        self.tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment())
    }
}

pub trait AbstractTokenTarget: std::fmt::Debug {
    fn push(&mut self, lt: AbstractLineToken);
    fn insert_at(&mut self, idx: usize, tokens: &mut Vec<AbstractLineToken>);
    fn into_tokens(self, ct: ConvertType) -> Vec<ConcreteLineTokenAndTargets>;
    fn is_multiline(&self) -> bool;
    fn push_line_number(&mut self, number: LineNumber);
    fn single_line_string_length(&self) -> usize;
    fn index_of_prev_newline(&self) -> Option<usize>;
    fn last_token_is_a_newline(&self) -> bool;
    fn to_breakable_entry(self: Box<Self>) -> BreakableEntry;
    fn to_breakable_call_chain(self: Box<Self>) -> BreakableCallChainEntry;
    fn len(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct BreakableEntry {
    #[allow(dead_code)]
    spaces: ColNumber,
    tokens: Vec<AbstractLineToken>,
    line_numbers: HashSet<LineNumber>,
    delims: BreakableDelims,
    context: FormattingContext,
}

impl AbstractTokenTarget for BreakableEntry {
    fn to_breakable_entry(self: Box<Self>) -> BreakableEntry {
        *self
    }

    fn to_breakable_call_chain(self: Box<Self>) -> BreakableCallChainEntry {
        unimplemented!()
    }

    fn push(&mut self, lt: AbstractLineToken) {
        self.tokens.push(lt);
    }

    fn insert_at(&mut self, idx: usize, tokens: &mut Vec<AbstractLineToken>) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    fn into_tokens(self, ct: ConvertType) -> Vec<ConcreteLineTokenAndTargets> {
        match ct {
            ConvertType::MultiLine => {
                let mut new_tokens: Vec<_> = self
                    .tokens
                    .into_iter()
                    .flat_map(|t| t.into_multi_line())
                    .collect();
                new_tokens.insert(0, self.delims.multi_line_open().into());
                new_tokens.push(self.delims.multi_line_close().into());
                new_tokens
            }
            ConvertType::SingleLine => {
                let mut new_tokens: Vec<_> = self
                    .tokens
                    .into_iter()
                    .map(|t| t.into_single_line())
                    .collect();
                new_tokens.insert(0, self.delims.single_line_open().into());
                new_tokens.push(self.delims.single_line_close().into());
                new_tokens
            }
        }
    }

    fn last_token_is_a_newline(&self) -> bool {
        match self.tokens.last() {
            Some(x) => x.is_newline(),
            _ => false,
        }
    }

    fn index_of_prev_newline(&self) -> Option<usize> {
        let first_idx = self
            .tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment());
        match first_idx {
            Some(x) => {
                if matches!(self.tokens[x], AbstractLineToken::CollapsingNewLine(_))
                    || matches!(self.tokens[x], AbstractLineToken::SoftNewline(_))
                {
                    Some(x + 1)
                } else {
                    Some(x)
                }
            }
            None => None,
        }
    }

    fn single_line_string_length(&self) -> usize {
        self.tokens
            .iter()
            .map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby().len())
            .sum::<usize>()
            + self.delims.single_line_len()
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    fn is_multiline(&self) -> bool {
        self.line_numbers.len() > 1
            || self.any_collapsing_newline_has_heredoc_content()
            || self.contains_hard_newline()
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }
}

impl BreakableEntry {
    pub fn new(spaces: ColNumber, delims: BreakableDelims, context: FormattingContext) -> Self {
        BreakableEntry {
            spaces,
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            delims,
            context,
        }
    }

    pub fn entry_formatting_context(&self) -> FormattingContext {
        self.context
    }

    fn any_collapsing_newline_has_heredoc_content(&self) -> bool {
        self.tokens.iter().any(|t| match t {
            AbstractLineToken::CollapsingNewLine(Some(..)) => true,
            AbstractLineToken::SoftNewline(Some(..)) => true,
            AbstractLineToken::BreakableEntry(be) => {
                be.any_collapsing_newline_has_heredoc_content()
            }
            _ => false,
        })
    }

    fn contains_hard_newline(&self) -> bool {
        self.tokens.iter().any(|t| {
            matches!(
                t,
                AbstractLineToken::ConcreteLineToken(ConcreteLineToken::HardNewLine)
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct BreakableCallChainEntry {
    tokens: Vec<AbstractLineToken>,
    line_numbers: HashSet<LineNumber>,
    call_chain: Vec<CallChainElement>,
    context: FormattingContext,
}

impl AbstractTokenTarget for BreakableCallChainEntry {
    fn to_breakable_entry(self: Box<Self>) -> BreakableEntry {
        unimplemented!()
    }

    fn to_breakable_call_chain(self: Box<Self>) -> BreakableCallChainEntry {
        *self
    }

    fn push(&mut self, lt: AbstractLineToken) {
        self.tokens.push(lt);
    }

    fn insert_at(&mut self, idx: usize, tokens: &mut Vec<AbstractLineToken>) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    fn into_tokens(self, ct: ConvertType) -> Vec<ConcreteLineTokenAndTargets> {
        match ct {
            ConvertType::MultiLine => self
                .tokens
                .into_iter()
                .flat_map(|t| t.into_multi_line())
                .collect(),
            ConvertType::SingleLine => self
                .tokens
                .into_iter()
                .map(|t| t.into_single_line())
                .collect(),
        }
    }

    fn last_token_is_a_newline(&self) -> bool {
        match self.tokens.last() {
            Some(x) => x.is_newline(),
            _ => false,
        }
    }

    fn index_of_prev_newline(&self) -> Option<usize> {
        let first_idx = self
            .tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment());
        match first_idx {
            Some(x) => {
                if matches!(self.tokens[x], AbstractLineToken::CollapsingNewLine(_))
                    || matches!(self.tokens[x], AbstractLineToken::SoftNewline(_))
                {
                    Some(x + 1)
                } else {
                    Some(x)
                }
            }
            None => None,
        }
    }

    fn single_line_string_length(&self) -> usize {
        self.tokens
            .iter()
            .map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby().len())
            .sum::<usize>()
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    fn is_multiline(&self) -> bool {
        // Never multiline if we're in an embedded expression
        if self.context == FormattingContext::StringEmbexpr {
            return false;
        }

        let parens_are_multilined = self.tokens.iter().any(|token| match token {
            AbstractLineToken::BreakableEntry(be) => be.is_multiline(),
            _ => false,
        });
        if parens_are_multilined {
            dbg!(parens_are_multilined);
            return true;
        }

        let has_newline_contents =
            self.any_collapsing_newline_has_heredoc_content();
        if has_newline_contents {
            return true;
        }

        let mut call_chain_to_check = self.call_chain.clone();

        // We don't always want to multiline blocks if their only usage
        // is at the end of a chain, since it's common to have chains
        // that end with long blocks, but those blocks don't mean we should
        // multiline the rest of the chain.
        //
        // example:
        // ```
        // items.get_all.each do
        // end
        // ```
        if let Some(CallChainElement::Block(..)) = call_chain_to_check.last() {
            call_chain_to_check.pop();
        }

        let all_op_locations = self
            .call_chain
            .iter()
            .filter_map(|cc_elem| match cc_elem {
                CallChainElement::DotTypeOrOp(dot_type_or_op) => {
                    match dot_type_or_op {
                        // ColonColon is specially represented in the parser, and
                        // it can't be properly multilined anyways, so we ignore it here
                        DotTypeOrOp::ColonColon(..) => None,
                        DotTypeOrOp::StringDot(..) => None,
                        DotTypeOrOp::Op(Op(.., start_end))
                        | DotTypeOrOp::DotType(
                            DotType::LonelyOperator(LonelyOperator(_, start_end))
                            | DotType::Dot(Dot(_, start_end)),
                        ) => Some(start_end.clone()),
                        DotTypeOrOp::Period(Period(.., linecol)) => {
                            Some(StartEnd(linecol.0, linecol.0))
                        }
                    }
                }
                _ => None,
            })
            .collect::<Vec<StartEnd>>();

        // don't multiline if there's only one call in the chain
        if all_op_locations.len() < 2 {
            return false;
        }

        // Multiline the chain if all the operators (dots, double colons, etc.) are not on the same line
        if let Some(first_op_start_end) = all_op_locations.first() {
            let chain_is_user_multilined = !all_op_locations
                .iter()
                .all(|op_start_end| op_start_end == first_op_start_end);
            if chain_is_user_multilined {
                return true;
            }
        }

        // Ignore chains that are basically only method calls, e.g.
        // ````ruby
        // Thing.foo(args)
        // Thing.foo(args) { block! }
        // ```
        // These should always stay inline
        match call_chain_to_check.as_slice() {
            [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] =>
            {
                return false;
            }
            [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] => {
                if matches!(maybe_const_ref.as_ref(), Expression::ConstPathRef(..)) {
                    return false;
                }
            }
            _ => {}
        }

        if self.is_heredoc_call_chain_with_breakables(&call_chain_to_check) {
            return true;
        }

        // If the first item in the chain is a multiline expression (like a hash or array),
        // ignore it when checking line length
        // if let Some(CallChainElement::Expression(expr)) = call_chain_to_check.first() {
        //     let is_multiline_expression = ps.will_render_as_multiline(Box::new(|ps| {
        //         format_expression(ps, expr.as_ref().clone());
        //     }));

        //     if is_multiline_expression {
        //         call_chain_to_check.remove(0);
        //     }
        // }

        let chain_blocks_are_multilined = call_chain_to_check
            .iter()
            .filter_map(|elem| match elem {
                CallChainElement::Block(block) => Some(block.clone()),
                _ => None,
            })
            .any(|block| match block {
                Block::BraceBlock(brace_block) => brace_block.3.is_multiline(),
                Block::DoBlock(_) => true,
            });

        chain_blocks_are_multilined
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }
}

impl BreakableCallChainEntry {
    pub fn new(context: FormattingContext, call_chain: Vec<CallChainElement>) -> Self {
        BreakableCallChainEntry {
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            context: context,
            call_chain: call_chain,
        }
    }
    fn any_collapsing_newline_has_heredoc_content(&self) -> bool {
        self.tokens.iter().any(|t| match t {
            AbstractLineToken::CollapsingNewLine(Some(..)) => true,
            AbstractLineToken::SoftNewline(Some(..)) => true,
            AbstractLineToken::BreakableEntry(be) => {
                be.any_collapsing_newline_has_heredoc_content()
            }
            _ => false,
        })
    }

    pub fn entry_formatting_context(&self) -> FormattingContext {
        self.context
    }

    /// In practice, this generally means something like the call chain having something
    /// like a method call with args or a block, e.g.
    ///
    /// ```ruby
    /// # `|line|` here is the breakable
    /// <<~FOO.lines.map { |line| p(line) }
    /// FOO
    /// ```
    ///
    /// Breakables don't play very nicely with heredoc rendering in call chains,
    /// and it would likely be a pretty hefty refactor to properly support this.
    fn is_heredoc_call_chain_with_breakables(&self, cc_elements: &[CallChainElement]) -> bool {
        if let Some(CallChainElement::Expression(expr)) = cc_elements.first() {
            if let Expression::StringLiteral(string_literal) = &**expr {
                if matches!(string_literal, StringLiteral::Heredoc(..)) {
                    let contains_breakables = cc_elements.iter().any(|cc_elem| match cc_elem {
                        CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(
                            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(list),
                            ..,
                        ) => !list.is_empty(),
                        CallChainElement::Block(..) => true,
                        _ => false,
                    });
                    return contains_breakables;
                }
            }
        }

        false
    }
}
