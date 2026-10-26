use crate::delimiters::BreakableDelims;
use crate::heredoc_string::HeredocString;
use crate::line_tokens::{AbstractLineToken, ConcreteLineToken, ConcreteLineTokenAndTargets};
use crate::parser_state::FormattingContext;
use crate::types::LineNumber;

fn insert_at<T>(idx: usize, target: &mut Vec<T>, input: impl IntoIterator<Item = T>) {
    target.splice(idx..idx, input);
}

/// Recursively checks if a token tree contains any `HardNewLine`
fn tokens_contain_hard_newline(token: &AbstractLineToken<'_>) -> bool {
    match token {
        AbstractLineToken::ConcreteLineToken(ConcreteLineToken::HardNewLine) => true,
        AbstractLineToken::BreakableEntry(be) => {
            be.tokens().iter().any(tokens_contain_hard_newline)
        }
        AbstractLineToken::BreakableCallChainEntry(bcce) => {
            bcce.tokens().iter().any(tokens_contain_hard_newline)
        }
        AbstractLineToken::ConditionalLayoutEntry(cle) => cle.contains_hard_newline(),
        _ => false,
    }
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
        self.tokens.last().is_some_and(|x| x.is_newline())
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
        out.reserve(self.tokens.len() + 2);
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

    pub fn contains_hard_newline(&self) -> bool {
        self.tokens.iter().any(tokens_contain_hard_newline)
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
        out.reserve(self.tokens.len());
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
        if self.heredoc_in_receiver() {
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
        }) || self.heredoc_in_receiver()
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

    /// Removes `BeginCallChainIndent` and `EndCallChainIndent`, which is only really
    /// necessary when rendering a call chain as single-line. This prevents unnecessarily
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

    fn heredoc_in_receiver(&self) -> bool {
        let mut seen_heredoc = false;
        for t in &self.tokens {
            match t {
                AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::HeredocStart { .. } | ConcreteLineToken::HeredocClose { .. },
                ) => seen_heredoc = true,
                AbstractLineToken::ConcreteLineToken(
                    ConcreteLineToken::Dot
                    | ConcreteLineToken::LonelyOperator
                    | ConcreteLineToken::ColonColon,
                ) => return seen_heredoc,
                _ => {}
            }
        }

        // If we've made it to here, that means this is a caller with no dot,
        // so it cannot possibly have had a heredoc anywhere in
        false
    }

    pub fn single_line_len(&self) -> usize {
        self.tokens.iter().map(|tok| tok.single_line_len()).sum()
    }
}

#[derive(Debug)]
pub enum Breakable<'src> {
    DelimiterExpr(BreakableEntry<'src>),
    CallChain(BreakableCallChainEntry<'src>),
    InlineConditional(ConditionalLayoutEntry<'src>),
}

impl<'src> Breakable<'src> {
    pub fn push(&mut self, lt: AbstractLineToken<'src>) {
        match self {
            Breakable::DelimiterExpr(be) => be.push(lt),
            Breakable::CallChain(bcce) => bcce.push(lt),
            Breakable::InlineConditional(cle) => cle.push(lt),
        }
    }

    pub fn insert_at<I>(&mut self, idx: usize, tokens: I)
    where
        I: IntoIterator<Item = AbstractLineToken<'src>>,
    {
        match self {
            Breakable::DelimiterExpr(be) => be.insert_at(idx, tokens),
            Breakable::CallChain(bcce) => bcce.insert_at(idx, tokens),
            Breakable::InlineConditional(cle) => cle.insert_at(idx, tokens),
        }
    }

    pub fn push_line_number(&mut self, number: LineNumber) {
        match self {
            Breakable::DelimiterExpr(be) => be.push_line_number(number),
            Breakable::CallChain(_) | Breakable::InlineConditional(_) => {
                // `InlineConditional` line numbers are tracked by nested breakables,
                // and `CallChain` has its own multilining logic
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Breakable::DelimiterExpr(be) => be.len(),
            Breakable::CallChain(bcce) => bcce.len(),
            Breakable::InlineConditional(cle) => cle.len(),
        }
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        match self {
            Breakable::DelimiterExpr(be) => be.last_token_is_a_newline(),
            Breakable::CallChain(bcce) => bcce.last_token_is_a_newline(),
            Breakable::InlineConditional(cle) => cle.last_token_is_a_newline(),
        }
    }

    pub fn index_of_prev_newline(&self) -> Option<usize> {
        match self {
            Breakable::DelimiterExpr(be) => be.index_of_prev_newline(),
            Breakable::CallChain(bcce) => bcce.index_of_prev_newline(),
            Breakable::InlineConditional(cle) => cle.index_of_prev_newline(),
        }
    }

    pub fn into_breakable_entry(self) -> Option<BreakableEntry<'src>> {
        match self {
            Breakable::DelimiterExpr(be) => Some(be),
            _ => None,
        }
    }

    pub fn into_breakable_call_chain(self) -> Option<BreakableCallChainEntry<'src>> {
        match self {
            Breakable::CallChain(bcce) => Some(bcce),
            _ => None,
        }
    }

    pub fn into_conditional_layout(self) -> Option<ConditionalLayoutEntry<'src>> {
        match self {
            Breakable::InlineConditional(cle) => Some(cle),
            _ => None,
        }
    }

    pub fn as_conditional_layout_mut(&mut self) -> Option<&mut ConditionalLayoutEntry<'src>> {
        match self {
            Breakable::InlineConditional(cle) => Some(cle),
            _ => None,
        }
    }
}

/// Tracks whether tokens span multiple source lines.
#[derive(Debug, Clone)]
struct MultilineTracker {
    first_line: Option<LineNumber>,
    is_multiline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalLayoutPhase {
    Predicate,
    Statement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalKeyword {
    If,
    Elsif,
    Unless,
    While,
    Until,
}

impl ConditionalKeyword {
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::If => b"if",
            Self::Elsif => b"elsif",
            Self::Unless => b"unless",
            Self::While => b"while",
            Self::Until => b"until",
        }
    }
}

#[derive(Debug, Clone)]
struct ConditionalHeredocs<'src> {
    /// Use [`Self::into_phase_heredocs`] instead of indexing this directly
    heredocs: Vec<HeredocString<'src>>,
    predicate_start: usize,
}

impl<'src> ConditionalHeredocs<'src> {
    fn new() -> Self {
        Self {
            heredocs: Vec::new(),
            predicate_start: 0,
        }
    }

    fn has_statement_heredocs(&self) -> bool {
        self.predicate_start != 0
    }

    fn gather(&mut self, phase: ConditionalLayoutPhase, heredocs: Vec<HeredocString<'src>>) {
        self.heredocs.extend(heredocs);
        if phase == ConditionalLayoutPhase::Statement {
            self.predicate_start = self.heredocs.len();
        }
    }

    fn into_phase_heredocs(mut self) -> (Vec<HeredocString<'src>>, Vec<HeredocString<'src>>) {
        let predicate = self.heredocs.split_off(self.predicate_start);
        (predicate, self.heredocs)
    }

    fn into_all(self) -> Vec<HeredocString<'src>> {
        self.heredocs
    }
}

/// An entry that holds predicate and statement tokens for a conditional modifier.
///
/// This is used for conditionals like:
/// - Inline: `statement if predicate`
/// - Block:  `if predicate\n  statement\nend`
#[derive(Debug, Clone)]
pub struct ConditionalLayoutEntry<'src> {
    /// Use [`Self::statement_tokens`] and [`Self::predicate_tokens`] instead of indexing this directly
    tokens: Vec<AbstractLineToken<'src>>,
    /// Index of the boundary between the two phases in `tokens`
    predicate_start: usize,
    /// Boxed to keep `AbstractLineToken` small, since this is rarely used
    heredocs: Option<Box<ConditionalHeredocs<'src>>>,
    keyword: ConditionalKeyword,
    indent_depth: u32,
    phase: ConditionalLayoutPhase,
}

impl<'src> ConditionalLayoutEntry<'src> {
    pub fn new(keyword: ConditionalKeyword, indent_depth: u32) -> Self {
        ConditionalLayoutEntry {
            tokens: Vec::new(),
            predicate_start: 0,
            heredocs: None,
            keyword,
            indent_depth,
            phase: ConditionalLayoutPhase::Statement,
        }
    }

    pub fn gather_heredocs(&mut self, heredocs: Vec<HeredocString<'src>>) {
        if heredocs.is_empty() {
            return;
        }
        self.heredocs
            .get_or_insert_with(|| Box::new(ConditionalHeredocs::new()))
            .gather(self.phase, heredocs);
    }

    fn statement_tokens(&self) -> &[AbstractLineToken<'src>] {
        &self.tokens[..self.predicate_start]
    }

    fn predicate_tokens(&self) -> &[AbstractLineToken<'src>] {
        &self.tokens[self.predicate_start..]
    }

    fn drain_statement_tokens(&mut self) -> impl Iterator<Item = AbstractLineToken<'src>> + '_ {
        let end = self.predicate_start;
        self.predicate_start = 0;
        self.tokens.drain(..end)
    }

    fn drain_predicate_tokens(&mut self) -> impl Iterator<Item = AbstractLineToken<'src>> + '_ {
        self.tokens.drain(self.predicate_start..)
    }

    fn phase_tokens(&self) -> &[AbstractLineToken<'src>] {
        match self.phase {
            ConditionalLayoutPhase::Predicate => self.predicate_tokens(),
            ConditionalLayoutPhase::Statement => self.statement_tokens(),
        }
    }

    pub fn push(&mut self, token: AbstractLineToken<'src>) {
        self.tokens.push(token);
        if self.phase == ConditionalLayoutPhase::Statement {
            self.predicate_start = self.tokens.len();
        }
    }

    pub fn switch_to_predicate(&mut self) {
        debug_assert_eq!(
            self.phase,
            ConditionalLayoutPhase::Statement,
            "switch_to_predicate called when not in Statement phase"
        );
        debug_assert_eq!(
            self.predicate_start,
            self.tokens.len(),
            "all statement tokens should have been collected before switching phases"
        );
        self.phase = ConditionalLayoutPhase::Predicate;
    }

    /// Returns the number of tokens in the current phase's token list.
    ///
    /// This returns only the current phase's count (not the combined total) because
    /// `len()` is used by `shift_comments_at_index` to determine where to insert
    /// comments. Since we divide the token list at `predicate_start`, the
    /// index needs to be relative to whichever phase we're currently building.
    pub fn len(&self) -> usize {
        self.phase_tokens().len()
    }

    pub fn insert_at(
        &mut self,
        idx: usize,
        tokens: impl IntoIterator<Item = AbstractLineToken<'src>>,
    ) {
        match self.phase {
            ConditionalLayoutPhase::Predicate => {
                insert_at(self.predicate_start + idx, &mut self.tokens, tokens);
            }
            ConditionalLayoutPhase::Statement => {
                insert_at(idx, &mut self.tokens, tokens);
                self.predicate_start = self.tokens.len();
            }
        }
    }

    pub fn last_token_is_a_newline(&self) -> bool {
        self.phase_tokens().last().is_some_and(|x| x.is_newline())
    }

    pub fn index_of_prev_newline(&self) -> Option<usize> {
        let tokens = self.phase_tokens();
        tokens
            .iter()
            .rposition(|v| v.is_newline() || v.is_indent())
            .map(|x| {
                let token = &tokens[x];
                if matches!(token, AbstractLineToken::CollapsingNewLine(_))
                    || matches!(token, AbstractLineToken::SoftNewline(_))
                {
                    x + 1
                } else {
                    x
                }
            })
    }

    pub fn inline_single_line_len(&self) -> usize {
        let statement_len: usize = self
            .statement_tokens()
            .iter()
            .map(|t| t.single_line_len())
            .sum();
        let predicate_len: usize = self
            .predicate_tokens()
            .iter()
            .map(|t| t.single_line_len())
            .sum();
        // statement + space + keyword + space + predicate
        statement_len + 1 + self.keyword.as_bytes().len() + 1 + predicate_len
    }

    fn is_multiline(&self) -> bool {
        self.statement_tokens().iter().any(Self::token_is_multiline)
            || self.predicate_tokens().iter().any(Self::token_is_multiline)
    }

    pub fn contains_hard_newline(&self) -> bool {
        self.statement_tokens()
            .iter()
            .any(tokens_contain_hard_newline)
            || self
                .predicate_tokens()
                .iter()
                .any(tokens_contain_hard_newline)
    }

    fn token_is_multiline(token: &AbstractLineToken<'src>) -> bool {
        match token {
            AbstractLineToken::ConcreteLineToken(ConcreteLineToken::HardNewLine) => true,
            AbstractLineToken::BreakableEntry(be) => be.is_multiline(),
            AbstractLineToken::BreakableCallChainEntry(bcce) => {
                bcce.is_multiline() || bcce.tokens().iter().any(Self::token_is_multiline)
            }
            AbstractLineToken::ConditionalLayoutEntry(cle) => cle.is_multiline(),
            _ => false,
        }
    }

    pub fn should_use_block_form(&self, current_line_length: usize) -> bool {
        // If there's a heredoc in the body, it's multi-line
        if self
            .heredocs
            .as_deref()
            .is_some_and(|heredocs| heredocs.has_statement_heredocs())
        {
            return true;
        }
        self.is_multiline()
            || current_line_length + self.inline_single_line_len()
                > crate::render_queue_writer::MAX_LINE_LENGTH
    }

    pub fn write_inline_tokens(mut self, result: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        result.reserve(self.tokens.len() + 3);

        for token in self.drain_statement_tokens() {
            token.write_single_line(result);
        }

        result.push(ConcreteLineToken::Space.into());
        result.push(
            ConcreteLineToken::ConditionalKeyword {
                contents: self.keyword.as_bytes(),
            }
            .into(),
        );
        result.push(ConcreteLineToken::Space.into());

        for token in self.drain_predicate_tokens() {
            token.write_single_line(result);
        }

        if let Some(heredocs) = self.heredocs {
            // If we get here, there are no heredocs in the statement (since we checked in `should_use_block_form`)
            // so here we're in the predicate and render all stored heredocs
            debug_assert!(!heredocs.has_statement_heredocs());
            let heredocs = heredocs.into_all();
            if !heredocs.is_empty() {
                result.push(ConcreteLineToken::HardNewLine.into());
                AbstractLineToken::write_heredocs(Some(heredocs), result);
            }
        }
    }

    pub fn write_block_tokens(mut self, result: &mut Vec<ConcreteLineTokenAndTargets<'src>>) {
        result.reserve(self.tokens.len() + 7);

        let (predicate_heredocs, statement_heredocs) = self
            .heredocs
            .take()
            .map(|heredocs| heredocs.into_phase_heredocs())
            .unwrap_or_default();

        result.push(
            ConcreteLineToken::ConditionalKeyword {
                contents: self.keyword.as_bytes(),
            }
            .into(),
        );
        result.push(ConcreteLineToken::Space.into());

        for token in self.drain_predicate_tokens() {
            token.write_multi_line(result);
        }

        result.push(ConcreteLineToken::HardNewLine.into());
        if !predicate_heredocs.is_empty() {
            AbstractLineToken::write_heredocs(Some(predicate_heredocs), result);
        }
        result.push(
            ConcreteLineToken::Indent {
                depth: self.indent_depth + 2,
            }
            .into(),
        );

        for token in self.drain_statement_tokens() {
            token.write_multi_line(result);
        }

        result.push(ConcreteLineToken::HardNewLine.into());
        if !statement_heredocs.is_empty() {
            AbstractLineToken::write_heredocs(Some(statement_heredocs), result);
        }
        result.push(
            ConcreteLineToken::Indent {
                depth: self.indent_depth,
            }
            .into(),
        );
        result.push(ConcreteLineToken::End.into());
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(name: &'static [u8]) -> AbstractLineToken<'static> {
        AbstractLineToken::ConcreteLineToken(ConcreteLineToken::MethodName { name })
    }

    fn names(tokens: &[AbstractLineToken<'static>]) -> Vec<&'static [u8]> {
        tokens
            .iter()
            .filter_map(|token| match token {
                AbstractLineToken::ConcreteLineToken(ConcreteLineToken::MethodName { name }) => {
                    Some(*name)
                }
                _ => None,
            })
            .collect()
    }

    fn concrete_names(tokens: &[ConcreteLineTokenAndTargets<'static>]) -> Vec<&'static [u8]> {
        tokens
            .iter()
            .filter_map(|token| match token {
                ConcreteLineTokenAndTargets::ConcreteLineToken(ConcreteLineToken::MethodName {
                    name,
                }) => Some(*name),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn conditional_layout_tracks_phase_boundary() {
        let mut cle = ConditionalLayoutEntry::new(ConditionalKeyword::If, 0);

        cle.push(ident(b"statement"));
        assert_eq!(cle.len(), 1);
        assert_eq!(names(cle.statement_tokens()), vec![&b"statement"[..]]);
        assert!(cle.predicate_tokens().is_empty());

        cle.switch_to_predicate();
        assert_eq!(cle.len(), 0);

        cle.push(ident(b"predicate"));
        assert_eq!(cle.len(), 1);
        assert_eq!(names(cle.statement_tokens()), vec![&b"statement"[..]]);
        assert_eq!(names(cle.predicate_tokens()), vec![&b"predicate"[..]]);
    }

    #[test]
    fn conditional_layout_insert_at_respects_phase() {
        let mut cle = ConditionalLayoutEntry::new(ConditionalKeyword::If, 0);

        cle.push(ident(b"statement_a"));
        cle.insert_at(1, [ident(b"statement_b")]);
        assert_eq!(
            names(cle.statement_tokens()),
            vec![&b"statement_a"[..], &b"statement_b"[..]]
        );
        assert!(cle.predicate_tokens().is_empty());

        cle.switch_to_predicate();
        cle.push(ident(b"predicate_a"));
        cle.insert_at(0, [ident(b"predicate_b")]);
        assert_eq!(cle.len(), 2);
        assert_eq!(
            names(cle.predicate_tokens()),
            vec![&b"predicate_b"[..], &b"predicate_a"[..]]
        );
        assert_eq!(
            names(cle.statement_tokens()),
            vec![&b"statement_a"[..], &b"statement_b"[..]]
        );
    }

    #[test]
    fn conditional_layout_renders_in_phase_order() {
        let mut cle = ConditionalLayoutEntry::new(ConditionalKeyword::If, 0);
        cle.push(ident(b"statement"));
        cle.switch_to_predicate();
        cle.push(ident(b"predicate"));

        let mut inline = Vec::new();
        cle.clone().write_inline_tokens(&mut inline);
        assert_eq!(
            concrete_names(&inline),
            vec![&b"statement"[..], &b"predicate"[..]]
        );

        let mut block = Vec::new();
        cle.write_block_tokens(&mut block);
        assert_eq!(
            concrete_names(&block),
            vec![&b"predicate"[..], &b"statement"[..]]
        );
    }
}
