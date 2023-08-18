use crate::delimiters::BreakableDelims;
use crate::file_comments::FileComments;
use crate::format::format_expression;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::{will_render_as_multiline, BaseParserState, FormattingContext};
use crate::ripper_tree_types::{Block, CallChainElement, Expression, StringLiteral};
use crate::types::{ColNumber, LineNumber};
use std::collections::HashSet;
use std::iter;

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
    fn to_breakable_entry(self: Box<Self>) -> BreakableEntry;
    fn to_breakable_call_chain(self: Box<Self>) -> BreakableCallChainEntry;
    fn len(&self) -> usize;
    fn tokens(&self) -> &Vec<AbstractLineToken>;
    fn any_collapsing_newline_has_heredoc_content(&self) -> bool;

    fn last_token_is_a_newline(&self) -> bool {
        match self.tokens().last() {
            Some(x) => x.is_newline(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BreakableEntry {
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
        unreachable!()
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
                    .flat_map(|t| t.into_single_line())
                    .collect();
                new_tokens.insert(0, self.delims.single_line_open().into());
                new_tokens.push(self.delims.single_line_close().into());
                new_tokens
            }
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
            .flat_map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby(ConvertType::SingleLine).len())
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

    fn tokens(&self) -> &Vec<AbstractLineToken> {
        &self.tokens
    }

    fn any_collapsing_newline_has_heredoc_content(&self) -> bool {
        self.tokens().iter().any(|t| match t {
            AbstractLineToken::CollapsingNewLine(Some(..)) => true,
            AbstractLineToken::SoftNewline(Some(..)) => true,
            AbstractLineToken::BreakableEntry(be) => {
                be.any_collapsing_newline_has_heredoc_content()
            }
            _ => false,
        })
    }
}

impl BreakableEntry {
    pub fn new(delims: BreakableDelims, context: FormattingContext) -> Self {
        BreakableEntry {
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            delims,
            context,
        }
    }

    pub fn entry_formatting_context(&self) -> FormattingContext {
        self.context
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
    pub starting_indentation_depth: u32,
    tokens: Vec<AbstractLineToken>,
    line_numbers: HashSet<LineNumber>,
    call_chain: Vec<CallChainElement>,
    context: FormattingContext,
}

impl AbstractTokenTarget for BreakableCallChainEntry {
    fn to_breakable_entry(self: Box<Self>) -> BreakableEntry {
        unreachable!()
    }

    fn tokens(&self) -> &Vec<AbstractLineToken> {
        &self.tokens
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
                .flat_map(|t| t.into_single_line())
                .collect(),
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
        // Render all tokens to strings, but since these are call chains, they may
        // have multiline blocks (which will often be quite long vertically, even if
        // they're under 120 characters horizontally). In this case, look for the longest
        // individual line and get _that_ max length
        self.tokens
            .iter()
            .flat_map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby(ConvertType::SingleLine).len())
            .sum::<usize>()
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    fn is_multiline(&self) -> bool {
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

        if self.is_heredoc_call_chain_with_breakables(&call_chain_to_check) {
            return true;
        }

        // If the first item in the chain is a multiline expression (like a hash or array),
        // ignore it when checking line length
        if let Some(CallChainElement::Expression(expr)) = call_chain_to_check.first() {
            let is_multiline_expression = will_render_as_multiline(
                &BaseParserState::new(FileComments::default()),
                Box::new(|ps| {
                    format_expression(ps, expr.as_ref().clone());
                }),
            );
            if is_multiline_expression {
                call_chain_to_check.remove(0);
            }
        }

        let all_element_locations = call_chain_to_check
            .iter()
            .filter_map(|cc_elem| cc_elem.start_line())
            .collect::<Vec<u64>>();

        // Multiline the chain if all the call chain elements are not on the same line
        if let Some(first_op_start_end) = all_element_locations.first() {
            let chain_is_user_multilined = !all_element_locations
                .iter()
                .all(|op_start_end| op_start_end == first_op_start_end);
            if chain_is_user_multilined {
                return true;
            }
        }

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

    fn any_collapsing_newline_has_heredoc_content(&self) -> bool {
        self.tokens().iter().any(|t| match t {
            AbstractLineToken::CollapsingNewLine(Some(..)) => true,
            AbstractLineToken::SoftNewline(Some(..)) => true,
            AbstractLineToken::BreakableEntry(be) => {
                be.any_collapsing_newline_has_heredoc_content()
            }
            _ => false,
        }) || self.call_chain.iter().any(|cce| match cce {
            CallChainElement::Expression(expr) => {
                matches!(
                    expr.as_ref(),
                    Expression::StringLiteral(StringLiteral::Heredoc(..))
                )
            }
            _ => false,
        })
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }
}

impl BreakableCallChainEntry {
    pub fn new(
        context: FormattingContext,
        call_chain: Vec<CallChainElement>,
        starting_indentation_depth: u32,
    ) -> Self {
        BreakableCallChainEntry {
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            starting_indentation_depth,
            context,
            call_chain,
        }
    }

    pub fn remove_call_chain_magic_tokens(&mut self) {
        self.tokens.retain(|t| {
            !matches!(
                t,
                AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::BeginCallChainIndent | ConcreteLineToken::EndCallChainIndent
                )
            )
        });
    }

    pub fn entry_formatting_context(&self) -> FormattingContext {
        self.context
    }

    pub fn longest_multiline_string_length(&self) -> usize {
        // Render all tokens to strings, but since these are call chains, they may
        // have multiline blocks (which will often be quite long vertically, even if
        // they're under 120 characters horizontally). In this case, look for the longest
        // individual line and get _that_ max length
        iter::once(&AbstractLineToken::ConcreteLineToken(
            // Push the starting indentation for the first line -- other
            // lines will already have this indentation
            ConcreteLineToken::Indent {
                depth: self.starting_indentation_depth,
            },
        ))
        .chain(&self.tokens)
        .map(|tok| {
            let forced_multiline = match tok {
                AbstractLineToken::BreakableCallChainEntry(bcce) => bcce.is_multiline(),
                AbstractLineToken::BreakableEntry(be) => be.is_multiline(),
                _ => false,
            };
            if forced_multiline {
                RenderItem {
                    tokens: tok.clone().into_multi_line(),
                    convert_type: ConvertType::MultiLine,
                }
            } else {
                RenderItem {
                    tokens: tok.clone().into_single_line(),
                    convert_type: ConvertType::SingleLine,
                }
            }
        })
        .flat_map(
            |RenderItem {
                 tokens,
                 convert_type,
             }| {
                tokens
                    .into_iter()
                    .map(move |tok| tok.into_ruby(convert_type))
            },
        )
        .collect::<String>()
        .split('\n')
        .map(|st| st.len())
        .max()
        .unwrap()
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
                return matches!(string_literal, StringLiteral::Heredoc(..));
            }
        }

        false
    }

    pub fn must_single_line(&self) -> bool {
        // Ignore chains that are basically only method calls, e.g.
        // ````ruby
        // Thing.foo(args)
        // Thing.foo(args) { block! }
        // ```
        match self.call_chain.as_slice() {
            [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
            | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] =>
            {
                return true;
            }
            [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
            | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] => {
                if matches!(maybe_const_ref.as_ref(), Expression::ConstPathRef(..)) {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

struct RenderItem {
    tokens: Vec<ConcreteLineTokenAndTargets>,
    convert_type: ConvertType,
}
