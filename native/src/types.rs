pub type LineNumber = u64;

#[derive(Debug)]
pub enum FormatStatus {
    BadlyFormedTree,
    UnknownToken,
}

// 4 billion characters ought to be enough for anyone
pub type ColNumber = u32;
