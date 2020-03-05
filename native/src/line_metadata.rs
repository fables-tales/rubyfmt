#[derive(Debug)]
pub struct LineMetadata {
    class_or_module: bool,
    conditional: bool,
    end: bool,
    def: bool,
}

impl LineMetadata {
    pub fn new() -> Self {
        LineMetadata {
            class_or_module: false,
            conditional: false,
            end: false,
            def: false,
        }
    }

    pub fn has_class_or_module_definition(&self) -> bool {
        self.class_or_module
    }

    pub fn set_defines_class_or_module(&mut self) {
        self.class_or_module = true;
    }

    pub fn set_has_conditional(&mut self) {
        self.conditional = true;
    }

    pub fn set_has_end(&mut self) {
        self.end = true;
    }

    pub fn set_has_def(&mut self) {
        self.def = true;
    }

    pub fn wants_spacer_for_conditional(&self) -> bool {
        !(
            self.conditional ||
            self.class_or_module ||
            self.end ||
            self.def
        )
    }
}
