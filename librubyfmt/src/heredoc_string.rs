use crate::types::ColNumber;
#[derive(Debug, Clone)]
pub struct HeredocString {
    pub symbol: String,
    pub squiggly: bool,
    pub buf: Vec<u8>,
    pub indent: ColNumber,
}

impl HeredocString {
    pub fn new(symbol: String, squiggly: bool, buf: Vec<u8>, indent: ColNumber) -> Self {
        HeredocString {
            symbol,
            squiggly,
            buf,
            indent,
        }
    }
}
