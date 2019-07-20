pub type LineNumber = u64;

#[derive(Debug)]
pub enum FormatStatus {
    BadlyFormedTree,
    UnknownToken,
}
