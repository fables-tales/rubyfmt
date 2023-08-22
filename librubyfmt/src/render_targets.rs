use crate::delimiters::BreakableDelims;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::FormattingContext;
use crate::ripper_tree_types::{CallChainElement, Expression, StringLiteral};
use crate::types::LineNumber;
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
    fn single_line_string_length(&self, current_line_length: usize) -> usize;
    fn to_breakable_entry(self: Box<Self>) -> Option<BreakableEntry>;
    fn to_breakable_call_chain(self: Box<Self>) -> Option<BreakableCallChainEntry>;
    fn tokens(&self) -> &Vec<AbstractLineToken>;
    fn any_collapsing_newline_has_heredoc_content(&self) -> bool;

    fn len(&self) -> usize {
        self.tokens().len()
    }

    fn index_of_prev_newline(&self) -> Option<usize> {
        let first_idx = self
            .tokens()
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment());
        match first_idx {
            Some(x) => {
                if matches!(self.tokens()[x], AbstractLineToken::CollapsingNewLine(_))
                    || matches!(self.tokens()[x], AbstractLineToken::SoftNewline(_))
                {
                    Some(x + 1)
                } else {
                    Some(x)
                }
            }
            None => None,
        }
    }

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
    context: Vec<FormattingContext>,
}

impl AbstractTokenTarget for BreakableEntry {
    fn to_breakable_entry(self: Box<Self>) -> Option<BreakableEntry> {
        Some(*self)
    }

    fn to_breakable_call_chain(self: Box<Self>) -> Option<BreakableCallChainEntry> {
        None
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

    fn single_line_string_length(&self, current_line_length: usize) -> usize {
        self.tokens
            .iter()
            .flat_map(|tok| tok.clone().into_single_line())
            .map(|tok| tok.into_ruby().len())
            .sum::<usize>()
            + self.delims.single_line_len()
            + current_line_length
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    fn is_multiline(&self) -> bool {
        self.line_numbers.len() > 1
            || self.any_collapsing_newline_has_heredoc_content()
            || self.contains_hard_newline()
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
    pub fn new(delims: BreakableDelims, context: Vec<FormattingContext>) -> Self {
        BreakableEntry {
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            delims,
            context,
        }
    }

    pub fn in_string_embexpr(&self) -> bool {
        self.context
            .iter()
            .any(|fc| fc == &FormattingContext::StringEmbexpr)
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
    context: Vec<FormattingContext>,
}

impl AbstractTokenTarget for BreakableCallChainEntry {
    fn to_breakable_entry(self: Box<Self>) -> Option<BreakableEntry> {
        None
    }

    fn tokens(&self) -> &Vec<AbstractLineToken> {
        &self.tokens
    }

    fn to_breakable_call_chain(self: Box<Self>) -> Option<BreakableCallChainEntry> {
        Some(*self)
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

    fn single_line_string_length(&self, current_line_length: usize) -> usize {
        // Render all tokens to strings, but since these are call chains, they may
        // have multiline blocks (which will often be quite long vertically, even if
        // they're under 120 characters horizontally). In this case, look for the longest
        // individual line and get _that_ max length.
        let mut tokens = self.tokens.clone();
        if tokens.len() > 2 {
            if let Some(AbstractLineToken::ConcreteLineToken(ConcreteLineToken::End)) =
                tokens.get(tokens.len() - 2)
            {
                // Pop off all tokens that make up the `do`/`end` block (but not `do`!),
                // since we assume that the block contents will handle their own line
                // length appropriately.
                while let Some(token) = tokens.last() {
                    if matches!(
                        token,
                        AbstractLineToken::ConcreteLineToken(ConcreteLineToken::DoKeyword)
                    ) {
                        break;
                    }
                    tokens.pop();
                }
            }
        }

        if let Some(AbstractLineToken::BreakableEntry(_)) = tokens.first() {
            tokens.remove(0);
        }
        // EndCallChainIndent, which we don't care about
        tokens.pop();
        // If the last breakable extends beyond the line length but the call chain doesn't,
        // the breakable will break itself, e.g.
        // ```ruby
        // #                                              ↓ if the break is here, we'll break the parens instead of the call chain
        // AssumeThisIs.one_hundred_twenty_characters(breaks_here)
        // ```
        if let Some(AbstractLineToken::BreakableEntry(_)) = tokens.last() {
            tokens.pop();
        }
        tokens.insert(
            0,
            AbstractLineToken::ConcreteLineToken(
                // Push the starting indentation for the first line -- other
                // lines will already have the appropriate indentation
                ConcreteLineToken::Indent {
                    depth: current_line_length as u32,
                },
            ),
        );

        tokens
            .into_iter()
            .flat_map(|t| t.into_single_line())
            .map(|t| t.into_ruby())
            .collect::<String>()
            .len()
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.line_numbers.insert(number);
    }

    fn is_multiline(&self) -> bool {
        if self.begins_with_heredoc() {
            return true;
        }

        let mut call_chain_to_check = self.call_chain.as_slice();
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
            call_chain_to_check = &call_chain_to_check[..call_chain_to_check.len() - 1];
        }

        let has_leading_expression = match call_chain_to_check.first() {
            Some(CallChainElement::Expression(expr)) => !expr.is_constant_reference(),
            _ => false,
        };
        let has_comments = self.tokens.iter().any(|t| {
            matches!(
                t,
                AbstractLineToken::ConcreteLineToken(ConcreteLineToken::Comment { .. })
            )
        });

        // If the first item in the chain is a multiline expression (like a hash or array),
        // ignore it when checking line length.
        // Don't ignore this if there are comments in the call chain though; this check may
        // cause it to single-lined, which breaks comment rendering.
        if has_leading_expression && !has_comments {
            call_chain_to_check = &call_chain_to_check[1..];
        }

        let chain_is_user_multilined = call_chain_to_check
            .iter()
            .filter_map(|cc_elem| cc_elem.start_line())
            .collect::<HashSet<_>>()
            .len()
            > 1;

        if chain_is_user_multilined {
            return true;
        }

        false
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
            // In cases where the heredoc is the first item in the call chain,
            // it won't get stored in an abstract token; instead, it'll be directly
            // in the call chain as a concrete token.
            CallChainElement::Expression(expr) => {
                matches!(
                    expr.as_ref(),
                    Expression::StringLiteral(StringLiteral::Heredoc(..))
                )
            }
            _ => false,
        })
    }
}

impl BreakableCallChainEntry {
    pub fn new(context: Vec<FormattingContext>, call_chain: Vec<CallChainElement>) -> Self {
        BreakableCallChainEntry {
            tokens: Vec::new(),
            line_numbers: HashSet::new(),
            context,
            call_chain,
        }
    }

    /// Removes `BeginCallChainIndent` and `EndCallChainIndent`, which is only really
    /// necessary when rendering a call chain as single-line. This prevents unnecessariliy
    /// increasing the indentation for a trailing block in e.g. `thing.each do; /* block */; end`
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

    pub fn in_string_embexpr(&self) -> bool {
        self.context
            .iter()
            .any(|fc| fc == &FormattingContext::StringEmbexpr)
    }

    fn begins_with_heredoc(&self) -> bool {
        if let Some(CallChainElement::Expression(expr)) = self.call_chain.first() {
            if let Expression::StringLiteral(string_literal) = &**expr {
                return matches!(string_literal, StringLiteral::Heredoc(..));
            }
        }

        false
    }
}
