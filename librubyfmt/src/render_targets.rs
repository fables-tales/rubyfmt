use crate::delimiters::BreakableDelims;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::FormattingContext;
use crate::ripper_tree_types::CallChainElement;
use crate::types::LineNumber;

fn insert_at<T>(idx: usize, target: &mut Vec<T>, input: impl IntoIterator<Item = T>) {
    target.splice(idx..idx, input);
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

    pub fn insert_at(
        &mut self,
        idx: usize,
        tokens: impl IntoIterator<Item = ConcreteLineTokenAndTargets>,
    ) {
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
    fn insert_at(&mut self, idx: usize, tokens: Box<dyn Iterator<Item = AbstractLineToken> + '_>);
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
        self.tokens()
            .iter()
            .rposition(|v| v.is_newline() || v.is_comment())
            .map(|x| {
                let token = &self.tokens()[x];
                if matches!(token, AbstractLineToken::CollapsingNewLine(_))
                    || matches!(token, AbstractLineToken::SoftNewline(_))
                {
                    x + 1
                } else {
                    x
                }
            })
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
    multiline_tracker: MultilineTracker,
    delims: BreakableDelims,
    in_string_embexpr: bool,
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

    fn insert_at(&mut self, idx: usize, tokens: Box<dyn Iterator<Item = AbstractLineToken> + '_>) {
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
        self.single_line_len() + current_line_length
    }

    fn push_line_number(&mut self, number: LineNumber) {
        self.multiline_tracker.on_line(number);
    }

    fn is_multiline(&self) -> bool {
        self.multiline_tracker.is_multiline()
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
    pub fn new(delims: BreakableDelims, formatting_context: &[FormattingContext]) -> Self {
        let in_string_embexpr = formatting_context
            .iter()
            .any(|fc| fc == &FormattingContext::StringEmbexpr);

        BreakableEntry {
            tokens: Vec::new(),
            multiline_tracker: MultilineTracker::new(),
            delims,
            in_string_embexpr,
        }
    }

    pub fn in_string_embexpr(&self) -> bool {
        self.in_string_embexpr
    }

    fn contains_hard_newline(&self) -> bool {
        self.tokens.iter().any(|t| {
            matches!(
                t,
                AbstractLineToken::ConcreteLineToken(ConcreteLineToken::HardNewLine)
            )
        })
    }

    pub fn single_line_len(&self) -> usize {
        self.tokens
            .iter()
            .map(|tok| tok.single_line_len())
            .sum::<usize>()
            + self.delims.single_line_len()
    }

    /// Returns the single-line length of just the block params (if any),
    /// excluding the body. Used by call chain line length calculation.
    pub fn single_line_len_params_only(&self) -> usize {
        if let Some(AbstractLineToken::BreakableEntry(params)) = self.tokens.first()
            && params.delims == BreakableDelims::for_block_params()
        {
            return params.single_line_len() + self.delims.single_line_len();
        }
        0
    }
}

/// This struct is a bit of a hack to support both
/// the Ripper tree and the Prism tree at the same time.
/// The Prism tree has more accurate offset handling that obviates
/// the hacks put in to support Ripper, but for now we need to support
/// and I didn't want to have to fork the implementation of BreakableCallChainEntry.
/// Once Prism is fully featured, we should delete this Ripper handling entirely.
#[derive(Debug, Clone)]
pub enum MultilineHandling {
    Ripper(Vec<CallChainElement>),
    Prism(bool),
}

#[derive(Debug, Clone)]
pub struct BreakableCallChainEntry {
    tokens: Vec<AbstractLineToken>,
    multiline_handling: MultilineHandling,
    in_string_embexpr: bool,
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

    fn insert_at(&mut self, idx: usize, tokens: Box<dyn Iterator<Item = AbstractLineToken> + '_>) {
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
        let mut tokens = self.tokens.as_slice();
        let mut brace_block_params_only_index = None;

        if tokens.len() > 2 {
            let index = tokens.len() - 2;
            let token = &tokens[index];
            if matches!(
                token,
                AbstractLineToken::ConcreteLineToken(ConcreteLineToken::End)
            ) {
                // Pop off all tokens that make up the block (but not the block params!),
                // since we assume that the block contents will handle their own line
                // length appropriately.
                while let Some((token, rest)) = tokens.split_last() {
                    if matches!(
                        token,
                        AbstractLineToken::BreakableEntry(BreakableEntry { delims, .. }) if *delims == BreakableDelims::for_block_params()
                    ) {
                        break;
                    }
                    tokens = rest;
                }
            } else if let AbstractLineToken::BreakableEntry(be) = token
                && be.delims == BreakableDelims::for_brace_block()
            {
                brace_block_params_only_index = Some(index);
            }
        }

        if let Some((AbstractLineToken::BreakableEntry(_), rest)) = tokens.split_first() {
            if let Some(idx) = brace_block_params_only_index {
                brace_block_params_only_index = Some(idx - 1);
            }
            tokens = rest;
        }
        if let Some((
            AbstractLineToken::ConcreteLineToken(ConcreteLineToken::EndCallChainIndent),
            rest,
        )) = tokens.split_last()
        {
            tokens = rest;
        }
        let call_count = tokens
            .iter()
            .filter(|t| {
                matches!(
                    t,
                    AbstractLineToken::ConcreteLineToken(
                        ConcreteLineToken::Dot | ConcreteLineToken::LonelyOperator
                    )
                )
            })
            .count();
        // If the last breakable is multiline (and not a block/block params), ignore it. The user likely
        // intentionally chose a line break strategy, so try our best to respect it.
        //
        // However, if there's only one item in the chain, try our best to leave that in place.
        // `foo\n.bar` is always a little awkward.
        if let Some((AbstractLineToken::BreakableEntry(be), rest)) = tokens.split_last()
            && (call_count == 1 || be.is_multiline())
            && be.delims != BreakableDelims::for_brace_block()
            && be.delims != BreakableDelims::for_block_params()
        {
            if let Some(params_index) = brace_block_params_only_index
                && params_index == tokens.len() - 1
            {
                brace_block_params_only_index = None;
            }
            tokens = rest;
        }

        tokens
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if let Some(params_only_idx) = brace_block_params_only_index
                    && params_only_idx == i
                    && let AbstractLineToken::BreakableEntry(be) = t
                {
                    return be.single_line_len_params_only();
                }
                t.single_line_len()
            })
            .sum::<usize>()
            + current_line_length
    }

    fn push_line_number(&mut self, _number: LineNumber) {
        // No-op, BreakableCallChainEntry has custom multilining logic
        // that doesn't depend on the source line numbers
    }

    fn is_multiline(&self) -> bool {
        if self.begins_with_heredoc() {
            return true;
        }

        match &self.multiline_handling {
            MultilineHandling::Prism(is_user_multilined) => *is_user_multilined,
            MultilineHandling::Ripper(call_chain_elements) => {
                let mut call_chain_to_check = call_chain_elements.as_slice();
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

                let mut lines = call_chain_to_check
                    .iter()
                    .filter_map(|cc_elem| cc_elem.start_line());
                match lines.next() {
                    Some(first) => lines.any(|line| line != first),
                    None => false,
                }
            }
        }
    }

    fn any_collapsing_newline_has_heredoc_content(&self) -> bool {
        self.tokens().iter().any(|t| match t {
            AbstractLineToken::CollapsingNewLine(Some(..)) => true,
            AbstractLineToken::SoftNewline(Some(..)) => true,
            AbstractLineToken::BreakableEntry(be) => {
                be.any_collapsing_newline_has_heredoc_content()
            }
            _ => false,
        }) || matches!(
            self.tokens.first(),
            Some(AbstractLineToken::ConcreteLineToken(
                ConcreteLineToken::HeredocStart { .. }
            ))
        )
    }
}

impl BreakableCallChainEntry {
    pub fn new(
        formatting_context: &[FormattingContext],
        multiline_handling: MultilineHandling,
    ) -> Self {
        let in_string_embexpr = formatting_context
            .iter()
            .any(|fc| fc == &FormattingContext::StringEmbexpr);

        BreakableCallChainEntry {
            tokens: Vec::new(),
            in_string_embexpr,
            multiline_handling,
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
        self.in_string_embexpr
    }

    fn begins_with_heredoc(&self) -> bool {
        matches!(
            self.tokens.first(),
            Some(AbstractLineToken::ConcreteLineToken(
                ConcreteLineToken::HeredocStart { .. }
            ))
        )
    }

    pub fn single_line_len(&self) -> usize {
        self.tokens.iter().map(|tok| tok.single_line_len()).sum()
    }
}

/// Tracks whether tokens span multiple source lines.
#[derive(Debug, Clone)]
struct MultilineTracker {
    first_line: Option<LineNumber>,
    is_multiline: bool,
}

impl MultilineTracker {
    fn new() -> Self {
        Self {
            first_line: None,
            is_multiline: false,
        }
    }

    fn on_line(&mut self, line: LineNumber) {
        match self.first_line {
            None => self.first_line = Some(line),
            Some(first) if first != line => self.is_multiline = true,
            Some(_) => {}
        }
    }

    fn is_multiline(&self) -> bool {
        self.is_multiline
    }
}
