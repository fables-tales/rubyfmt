use std::borrow::Cow;

/// Maximum indent depth we cache without allocation.
const MAX_CACHED_INDENT: usize = 128;
/// A pre-allocated array of spaces for efficient indentation
const SPACES: &[u8] = &[b' '; MAX_CACHED_INDENT];

/// Returns a byte slice of `depth` spaces. For common indent depths (<= 128),
/// this returns a static slice and only allocates for huge indents.
pub fn get_indent(depth: usize) -> Cow<'static, [u8]> {
    if depth <= MAX_CACHED_INDENT {
        Cow::Borrowed(&SPACES[..depth])
    } else {
        Cow::Owned(vec![b' '; depth])
    }
}
