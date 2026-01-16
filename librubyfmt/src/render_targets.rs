use crate::delimiters::BreakableDelims;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::FormattingContext;
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
pub struct BaseQueue<'src> {
    tokens: Vec<ConcreteLineTokenAndTargets<'src>>,
}

impl<'src> BaseQueue<'src> {
    pub fn push(&mut self, lt: ConcreteLineTokenAndTargets<'src>) {
        self.tokens.push(lt)
    }

    pub fn insert_at(
        &mut self,
        idx: usize,
        tokens: impl IntoIterator<Item = ConcreteLineTokenAndTargets<'src>>,
    ) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn into_tokens(self) -> Vec<ConcreteLineTokenAndTargets<'src>> {
        self.tokens
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.tokens.last().map(|x| x.is_newline()).unwrap_or(false)
    }

    pub fn index_of_prev_newline(&self) -> Option<usize> {
        self.tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_indent())
    }
}

pub trait AbstractTokenTarget<'src>: std::fmt::Debug {
    fn push(&mut self, lt: AbstractLineToken<'src>);
    fn write_tokens(self, ct: ConvertType, out: &mut Vec<ConcreteLineTokenAndTargets<'src>>);
    fn is_multiline(&self) -> bool;
    fn single_line_string_length(&self, current_line_length: usize) -> usize;
    fn tokens(&self) -> &Vec<AbstractLineToken<'src>>;
    fn any_collapsing_newline_has_heredoc_content(&self) -> bool;

    fn len(&self) -> usize {
        self.tokens().len()
    }

    fn index_of_prev_newline(&self) -> Option<usize> {
        self.tokens()
            .iter()
            .rposition(|v| v.is_newline() || v.is_indent())
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
pub struct BreakableEntry<'src> {
    tokens: Vec<AbstractLineToken<'src>>,
    multiline_tracker: MultilineTracker,
    pub delims: BreakableDelims,
    in_string_embexpr: bool,
}

impl<'src> AbstractTokenTarget<'src> for BreakableEntry<'src> {
    fn push(&mut self, lt: AbstractLineToken<'src>) {
        self.tokens.push(lt);
    }

    fn write_tokens(self, ct: ConvertType, out: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        match ct {
            ConvertType::MultiLine => {
                out.push(self.delims.multi_line_open().into());
                for t in self.tokens {
                    t.write_multi_line(out);
                }
                out.push(self.delims.multi_line_close().into());
            }
            ConvertType::SingleLine => {
                out.push(self.delims.single_line_open().into());
                for t in self.tokens {
                    t.write_single_line(out);
                }
                out.push(self.delims.single_line_close().into());
            }
        }
    }

    fn single_line_string_length(&self, current_line_length: usize) -> usize {
        self.single_line_len() + current_line_length
    }

    fn is_multiline(&self) -> bool {
        self.multiline_tracker.is_multiline()
            || self.any_collapsing_newline_has_heredoc_content()
            || self.contains_hard_newline()
    }

    fn tokens(&self) -> &Vec<AbstractLineToken<'src>> {
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

impl<'src> BreakableEntry<'src> {
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

    pub fn insert_at(
        &mut self,
        idx: usize,
        tokens: impl IntoIterator<Item = AbstractLineToken<'src>>,
    ) {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn push_line_number(&mut self, number: LineNumber) {
        self.multiline_tracker.on_line(number);
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

#[derive(Debug, Clone)]
pub struct BreakableCallChainEntry<'src> {
    tokens: Vec<AbstractLineToken<'src>>,
    is_user_multilined: bool,
    in_string_embexpr: bool,
}

impl<'src> AbstractTokenTarget<'src> for BreakableCallChainEntry<'src> {
    fn tokens(&self) -> &Vec<AbstractLineToken<'src>> {
        &self.tokens
    }

    fn push(&mut self, lt: AbstractLineToken<'src>) {
        self.tokens.push(lt);
    }

    fn write_tokens(self, ct: ConvertType, out: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        match ct {
            ConvertType::MultiLine => {
                for t in self.tokens {
                    t.write_multi_line(out);
                }
            }
            ConvertType::SingleLine => {
                for t in self.tokens {
                    t.write_single_line(out);
                }
            }
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

    fn is_multiline(&self) -> bool {
        if self.begins_with_heredoc() {
            return true;
        }

        self.is_user_multilined
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

impl<'src> BreakableCallChainEntry<'src> {
    pub fn new(formatting_context: &[FormattingContext], is_user_multilined: bool) -> Self {
        let in_string_embexpr = formatting_context
            .iter()
            .any(|fc| fc == &FormattingContext::StringEmbexpr);

        BreakableCallChainEntry {
            tokens: Vec::new(),
            in_string_embexpr,
            is_user_multilined,
        }
    }

    pub fn insert_at<I>(&mut self, idx: usize, tokens: I)
    where
        I: IntoIterator<Item = AbstractLineToken<'src>>,
    {
        insert_at(idx, &mut self.tokens, tokens)
    }

    pub fn push_line_number(&mut self, _number: LineNumber) {
        // No-op, BreakableCallChainEntry has custom multilining logic
        // that doesn't depend on the source line numbers
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

#[derive(Debug)]
pub enum Breakable<'src> {
    DelimiterExpr(BreakableEntry<'src>),
    CallChain(BreakableCallChainEntry<'src>),
}

impl<'src> Breakable<'src> {
    pub fn push(&mut self, lt: AbstractLineToken<'src>) {
        match self {
            Breakable::DelimiterExpr(be) => be.push(lt),
            Breakable::CallChain(bcce) => bcce.push(lt),
        }
    }

    pub fn insert_at<I>(&mut self, idx: usize, tokens: I)
    where
        I: IntoIterator<Item = AbstractLineToken<'src>>,
    {
        match self {
            Breakable::DelimiterExpr(be) => be.insert_at(idx, tokens),
            Breakable::CallChain(bcce) => bcce.insert_at(idx, tokens),
        }
    }

    pub fn push_line_number(&mut self, number: LineNumber) {
        match self {
            Breakable::DelimiterExpr(be) => be.push_line_number(number),
            Breakable::CallChain(bcce) => bcce.push_line_number(number),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Breakable::DelimiterExpr(be) => be.len(),
            Breakable::CallChain(bcce) => bcce.len(),
        }
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        match self {
            Breakable::DelimiterExpr(be) => be.last_token_is_a_newline(),
            Breakable::CallChain(bcce) => bcce.last_token_is_a_newline(),
        }
    }

    pub fn index_of_prev_newline(&self) -> Option<usize> {
        match self {
            Breakable::DelimiterExpr(be) => be.index_of_prev_newline(),
            Breakable::CallChain(bcce) => bcce.index_of_prev_newline(),
        }
    }

    pub fn into_breakable_entry(self) -> Option<BreakableEntry<'src>> {
        match self {
            Breakable::DelimiterExpr(be) => Some(be),
            Breakable::CallChain(_) => None,
        }
    }

    pub fn into_breakable_call_chain(self) -> Option<BreakableCallChainEntry<'src>> {
        match self {
            Breakable::DelimiterExpr(_) => None,
            Breakable::CallChain(bcce) => Some(bcce),
        }
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
