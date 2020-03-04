#[derive(Debug)]
pub struct LineMetadata {
    class_or_module: bool,
}

impl LineMetadata {
    pub fn new() -> Self {
        LineMetadata {
            class_or_module: false,
        }
    }

    pub fn has_class_or_module_definition(&self) -> bool {
        self.class_or_module
    }

    pub fn set_defines_class_or_module(&mut self) {
        self.class_or_module = true;
    }
}
