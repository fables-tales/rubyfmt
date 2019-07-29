use regex::Regex;

pub trait LineToken {
    fn as_single_line(self) -> Box<dyn LineToken>
    where
        Self: Sized + 'static,
    {
        return Box::new(self);
    }

    fn as_multi_line(self) -> Box<dyn LineToken>
    where
        Self: Sized + 'static,
    {
        return Box::new(self);
    }

    fn is_hard_newline(&self) -> bool {
        false
    }

    fn is_open_curly(&self) -> bool {
        false
    }

    fn is_close_curly(&self) -> bool {
        false
    }

    fn is_comma(&self) -> bool {
        false
    }

    fn is_newline(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn is_keyword(&self) -> bool {
        false
    }

    fn is_indent(&self) -> bool {
        false
    }

    fn declares_class_or_module(&self) -> bool {
        false
    }

    fn declares_if_or_unless(&self) -> bool {
        false
    }

    fn is_end(&self) -> bool {
        false
    }

    fn is_do(&self) -> bool {
        false
    }

    fn is_def(&self) -> bool {
        false
    }

    fn is_else(&self) -> bool {
        false
    }

    fn is_require(&self) -> bool {
        false
    }

    fn is_requirish(&self) -> bool {
        false
    }

    fn is_private(&self) -> bool {
        false
    }

    fn is_empty_string(&self) -> bool {
        false
    }

    fn consume_to_string(self: Box<Self>) -> String;
}

pub struct BreakableState {
    indentation_depth: u32,
}

impl LineToken for BreakableState {
    fn consume_to_string(self: Box<Self>) -> String {
        panic!("dont call to_s on BreakableState");
    }
}

pub struct HardNewLine {}

impl HardNewLine {
    pub fn new() -> Self {
        HardNewLine {}
    }
}

impl LineToken for HardNewLine {
    fn is_hard_newline(&self) -> bool {
        true
    }

    fn is_newline(&self) -> bool {
        true
    }
    fn consume_to_string(self: Box<Self>) -> String {
        "\n".into()
    }
}

pub struct Indent {
    depth: u16,
}
impl Indent {
    pub fn new(depth: u16) -> Self {
        Indent { depth: depth }
    }
}

impl LineToken for Indent {
    fn is_indent(&self) -> bool {
        true
    }

    fn consume_to_string(self: Box<Self>) -> String {
        (0..self.depth).map(|_| ' ').collect()
    }
}

pub struct Keyword {
    keyword: String,
}

impl Keyword {
    pub fn new(keyword: String) -> Self {
        Keyword { keyword: keyword }
    }
}

impl LineToken for Keyword {
    fn is_keyword(&self) -> bool {
        true
    }

    fn declares_class_or_module(&self) -> bool {
        self.keyword == "class" || self.keyword == "module"
    }

    fn declares_if_or_unless(&self) -> bool {
        self.keyword == "if" || self.keyword == "unless"
    }

    fn is_end(&self) -> bool {
        self.keyword == "end"
    }

    fn is_do(&self) -> bool {
        self.keyword == "do"
    }

    fn is_def(&self) -> bool {
        self.keyword == "def"
    }

    fn is_else(&self) -> bool {
        self.keyword == "else"
    }

    fn consume_to_string(self: Box<Self>) -> String {
        self.keyword
    }
}

pub struct DirectPart {
    part: String,
}

impl DirectPart {
    pub fn new(part: String) -> Self {
        DirectPart { part: part }
    }

    pub fn null() -> Self {
        DirectPart {
            part: "".to_string(),
        }
    }
}

impl LineToken for DirectPart {
    fn is_require(&self) -> bool {
        self.part == "require"
    }

    fn is_requirish(&self) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new("([^A-Za-z0-9]|^)require([^A-Za-z0-9])?").unwrap();
        }
        RE.is_match(&self.part)
    }

    fn consume_to_string(self: Box<Self>) -> String {
        self.part
    }
}

pub struct CommaSpace;

impl CommaSpace {
    pub fn new() -> Self {
        return CommaSpace;
    }
}

impl LineToken for CommaSpace {
    fn consume_to_string(self: Box<Self>) -> String {
        ", ".to_string()
    }
}

pub struct SoftIndent {
    depth: u16,
}
impl SoftIndent {
    pub fn new(depth: u16) -> Self {
        SoftIndent { depth: depth }
    }
}

impl LineToken for SoftIndent {
    fn is_indent(&self) -> bool {
        true
    }

    fn consume_to_string(self: Box<Self>) -> String {
        (0..self.depth).map(|_| ' ').collect()
    }

    fn as_single_line(self) -> Box<dyn LineToken> {
        Box::new(DirectPart::null())
    }
}

pub struct Comma;

impl Comma {
    pub fn new() -> Self {
        Comma
    }
}

impl LineToken for Comma {
    fn consume_to_string(self: Box<Self>) -> String {
        ",".to_string()
    }

    fn is_comma(&self) -> bool {
        true
    }
}

pub struct SoftNewline;

impl SoftNewline {
    pub fn new() -> Self {
        SoftNewline
    }
}

impl LineToken for SoftNewline {
    fn consume_to_string(self: Box<Self>) -> String {
        "\n".to_string()
    }

    fn is_newline(&self) -> bool {
        true
    }

    fn as_single_line(self) -> Box<LineToken> {
        Box::new(Space::new())
    }
}

pub struct Space;

impl Space {
    pub fn new() -> Self {
        Space
    }
}

impl LineToken for Space {
    fn consume_to_string(self: Box<Self>) -> String {
        " ".to_string()
    }
}
