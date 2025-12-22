#[derive(Debug)]
pub struct LineMetadata {
    gets_indented: bool,
    indent_level: Option<u32>,
    require: bool,
}

impl LineMetadata {
    pub fn indent_level_increases_between(prev: &LineMetadata, current: &LineMetadata) -> bool {
        prev.indent_level < current.indent_level
    }

    pub fn new() -> Self {
        LineMetadata {
            gets_indented: false,
            indent_level: None,
            require: false,
        }
    }

    pub fn set_has_require(&mut self) {
        self.require = true;
    }

    pub fn has_require(&self) -> bool {
        self.require
    }

    pub fn observe_indent_level(&mut self, level: u32) {
        self.indent_level = Some(level);
    }

    pub fn gets_indented(&self) -> bool {
        self.gets_indented
    }

    pub fn set_gets_indented(&mut self) {
        self.gets_indented = true;
    }
}
