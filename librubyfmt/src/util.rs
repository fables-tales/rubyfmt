use std::borrow::Cow;

use ruby_prism::Location;

/// Maximum indent depth we cache without allocation.
const MAX_CACHED_INDENT: usize = 128;
/// A pre-allocated string of spaces for efficient indentation
const SPACES: &str = unsafe { std::str::from_utf8_unchecked(&[b' '; MAX_CACHED_INDENT]) };

/// Returns a string of `depth` spaces. For common indent depths (<= 128),
/// this returns a static slice and only allocates for huge indents.
pub fn get_indent(depth: usize) -> Cow<'static, str> {
    if depth <= MAX_CACHED_INDENT {
        Cow::Borrowed(&SPACES[..depth])
    } else {
        Cow::Owned(" ".repeat(depth))
    }
}

fn u8_to_str(arr: &[u8]) -> &str {
    std::str::from_utf8(arr).unwrap()
}

pub fn loc_to_str(loc: Location<'_>) -> &str {
    u8_to_str(loc.as_slice())
}
