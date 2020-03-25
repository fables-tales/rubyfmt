#![allow(warnings)]
use serde::*;
use serde_json::Value;

use crate::types::LineNumber;

macro_rules! def_tag {
    ($tag_name:ident) => {
        def_tag!($tag_name, stringify!($tag_name));
    };

    ($tag_name:ident, $tag:expr) => {
        #[derive(Serialize, Debug, Clone)]
        pub struct $tag_name;
        impl<'de> Deserialize<'de> for $tag_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct TagVisitor;

                impl<'de> de::Visitor<'de> for TagVisitor {
                    type Value = ();

                    fn expecting(
                        &self,
                        f: &mut std::fmt::Formatter<'_>,
                    ) -> Result<(), std::fmt::Error> {
                        write!(f, $tag)
                    }

                    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        if s == $tag {
                            Ok(())
                        } else {
                            Err(E::custom("mismatched tag"))
                        }
                    }
                }

                let tag = deserializer.deserialize_str(TagVisitor)?;
                Ok($tag_name)
            }
        }
    };
}

def_tag!(program_tag, "program");
#[derive(Deserialize, Debug, Clone)]
pub struct Program(pub program_tag, pub Vec<Expression>);

def_tag!(undeserializable, "oiqjweoifjqwoeifjwqoiefjqwoiej");
#[derive(Deserialize, Debug, Clone)]
pub struct ToProc(pub undeserializable, pub Box<Expression>);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Expression {
    ToProc(ToProc),
    Class(Class),
    If(If),
    Unary(Unary),
    VoidStmt(VoidStmt),
    Def(Def),
    Defs(Defs),
    VCall(VCall),
    Ident(Ident),
    Params(Params),
    MethodCall(MethodCall),
    DotCall(DotCall),
    Call(Call),
    CommandCall(CommandCall),
    MethodAddArg(MethodAddArg),
    Int(Int),
    BareAssocHash(BareAssocHash),
    Symbol(Symbol),
    SymbolLiteral(SymbolLiteral),
    DynaSymbol(DynaSymbol),
    Begin(Begin),
    Paren(ParenExpr),
    Dot2(Dot2),
    Dot3(Dot3),
    Alias(Alias),
    Array(Array),
    StringLiteral(StringLiteral),
    XStringLiteral(XStringLiteral),
    VarRef(VarRef),
    Assign(Assign),
    MAssign(MAssign),
    Const(Const),
    Command(Command),
    ConstPathRef(ConstPathRef),
    Defined(Defined),
    TopConstRef(TopConstRef),
    RescueMod(RescueMod),
    MRHSAddStar(MRHSAddStar),
    Next(Next),
    StringConcat(StringConcat),
    Super(Super),
    Kw(Kw),
    Undef(Undef),
    Binary(Binary),
    Float(Float),
    Aref(Aref),
    Char(Char),
    Module(Module),
    Return(Return),
    Return0(Return0),
    Hash(Hash),
    RegexpLiteral(RegexpLiteral),
    Backref(Backref),
    Yield(Yield),
    MethodAddBlock(MethodAddBlock),
    While(While),
    WhileMod(WhileMod),
    UntilMod(UntilMod),
    IfMod(IfMod),
    UnlessMod(UnlessMod),
    Case(Case),
    Retry(Retry),
    SClass(SClass),
    Break(Break),
    StabbyLambda(StabbyLambda),
    Imaginary(Imaginary),
    Rational(Rational),
    MLhs(MLhs),
    Until(Until),
    For(For),
    IfOp(IfOp),
    OpAssign(OpAssign),
    Unless(Unless),
    ZSuper(ZSuper),
    Yield0(Yield0),
}

#[derive(Debug, Clone)]
pub struct MLhs(pub Vec<Expression>);

impl<'de> Deserialize<'de> for MLhs {
    fn deserialize<D>(deserializer: D) -> Result<MLhs, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MLhsVisitor;

        impl<'de> de::Visitor<'de> for MLhsVisitor {
            type Value = MLhs;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "[mlhs, (expression)*]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = match seq.next_element() {
                    Ok(Some(s)) => s,
                    _ => {
                        return Err(de::Error::custom("didn't get right tag"));
                    }
                };

                if tag != "mlhs" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let mut elements = Vec::new();
                let mut expr: Option<Expression> = seq.next_element()?;
                while expr.is_some() {
                    elements.push(expr.expect("we checked it's some"));
                    expr = seq.next_element()?;
                }

                Ok(MLhs(elements))
            }
        }

        deserializer.deserialize_seq(MLhsVisitor)
    }
}

def_tag!(zsuper_tag, "zsuper");
#[derive(Deserialize, Debug, Clone)]
pub struct ZSuper((zsuper_tag,));

def_tag!(yield0_tag, "yield0");
#[derive(Deserialize, Debug, Clone)]
pub struct Yield0((yield0_tag,));

def_tag!(if_tag, "if");
#[derive(Deserialize, Debug, Clone)]
pub struct If(
    pub if_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub Option<ElsifOrElse>,
);

def_tag!(unless_tag, "unless");
#[derive(Deserialize, Debug, Clone)]
pub struct Unless(
    pub unless_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub Option<Else>,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ElsifOrElse {
    Elsif(Elsif),
    Else(Else),
}

def_tag!(elsif_tag, "elsif");
#[derive(Deserialize, Debug, Clone)]
pub struct Elsif(
    pub elsif_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub Option<Box<ElsifOrElse>>,
);

def_tag!(else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct Else(pub else_tag, pub Vec<Expression>);

def_tag!(undef_tag, "undef");
#[derive(Deserialize, Debug, Clone)]
pub struct Undef(pub undef_tag, pub Vec<SymbolLiteral>);

def_tag!(string_concat_tag, "string_concat");
#[derive(Deserialize, Debug, Clone)]
pub struct StringConcat(
    pub string_concat_tag,
    pub StringConcatOrStringLiteral,
    pub StringLiteral,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringConcatOrStringLiteral {
    StringConcat(Box<StringConcat>),
    StringLiteral(StringLiteral),
}

def_tag!(mrhs_add_star_tag, "mrhs_add_star");
#[derive(Deserialize, Debug, Clone)]
pub struct MRHSAddStar(
    pub mrhs_add_star_tag,
    pub MRHSNewFromArgsOrEmpty,
    pub Box<Expression>,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MRHSNewFromArgsOrEmpty {
    MRHSNewFromArgs(MRHSNewFromArgs),
    Empty(Vec<Expression>),
}

// MRHSNewFromArgs is really annoying to parse, consider:
//
// ```ruby
// begin
// rescue A,*B
// end
// ```
// this will parse as:
//
// [:program,
//  [[:begin,
//    [:bodystmt,
//     [[:void_stmt]],
//     [:rescue,
//      [:mrhs_add_star,
//       [:mrhs_new_from_args, [[:var_ref, [:@const, "A", [1, 13]]]]],
//       [:var_ref, [:@const, "B", [1, 16]]]],
//      nil,
//      [[:void_stmt]],
//      nil],
//     nil,
//     nil]]]]
//
// however, if you have
// ```ruby
// a,b = 1,2
// ```
// you get
// [:program,
// [[:massign,
//   [[:var_field, [:@ident, "a", [1, 0]]],
//    [:var_field, [:@ident, "b", [1, 2]]]],
//   [:mrhs_new_from_args, [[:@int, "1", [1, 4]]], [:@int, "2", [1, 6]]]]]]
//
// in the first case, the mrhs_new_from_args looks like:
// [:mrhs_new_from_args, Vec<Expression>]
// in the second case, the mrhs_new_from_args_tag looks like:
// [:mrhs_new_from_args, Vec<Expression>, Expression]
//
// so we need to implement a custom deserializer, I am sad
def_tag!(mrhs_new_from_args_tag, "mrhs_new_from_args");
#[derive(Debug, Clone)]
pub struct MRHSNewFromArgs(
    pub mrhs_new_from_args_tag,
    pub ArgsAddStarOrExpressionList,
    pub Option<Box<Expression>>,
);

impl<'de> Deserialize<'de> for MRHSNewFromArgs {
    fn deserialize<D>(deserializer: D) -> Result<MRHSNewFromArgs, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MRHSNewFromArgsVisitor;

        impl<'de> de::Visitor<'de> for MRHSNewFromArgsVisitor {
            type Value = MRHSNewFromArgs;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "[mrhs_new_from_args, Vec<Expression>(, Expression?)]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = match seq.next_element() {
                    Ok(Some(s)) => s,
                    _ => {
                        return Err(de::Error::custom("didn't get right tag"));
                    }
                };
                if tag != "mrhs_new_from_args" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let expressions: ArgsAddStarOrExpressionList = seq
                    .next_element()?
                    .expect("didn't get args add star or expression list in mrhs new");
                let tail_expression: Option<Box<Expression>> = match seq.next_element() {
                    Ok(Some(v)) => Some(Box::new(v)),
                    Ok(None) => None,
                    Err(e) => None,
                };

                Ok(MRHSNewFromArgs(
                    mrhs_new_from_args_tag,
                    expressions,
                    tail_expression,
                ))
            }
        }

        deserializer.deserialize_seq(MRHSNewFromArgsVisitor)
    }
}

def_tag!(rescue_mod_tag, "rescue_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct RescueMod(pub rescue_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(defined_tag, "defined");
#[derive(Deserialize, Debug, Clone)]
pub struct Defined(pub defined_tag, pub Box<Expression>);

def_tag!(top_const_ref_tag, "top_const_ref");
#[derive(Deserialize, Debug, Clone)]
pub struct TopConstRef(pub top_const_ref_tag, pub Const);

def_tag!(top_const_field_tag, "top_const_field");
#[derive(Deserialize, Debug, Clone)]
pub struct TopConstField(pub top_const_field_tag, pub Const);

def_tag!(const_path_ref_tag, "const_path_ref");
#[derive(Deserialize, Debug, Clone)]
pub struct ConstPathRef(pub const_path_ref_tag, pub Box<Expression>, pub Const);

def_tag!(const_ref_tag, "const_ref");
#[derive(Deserialize, Debug, Clone)]
pub struct ConstRef(pub const_ref_tag, pub Const);

def_tag!(command_tag, "command");
#[derive(Deserialize, Debug, Clone)]
pub struct Command(
    pub command_tag,
    pub IdentOrConst,
    pub ArgsAddBlockOrExpressionList,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgsAddBlockOrExpressionList {
    ArgsAddBlock(ArgsAddBlock),
    ExpressionList(Vec<Expression>),
}

impl Command {
    pub fn to_method_call(self) -> MethodCall {
        let arg_node = match self.2 {
            ArgsAddBlockOrExpressionList::ArgsAddBlock(n) => ArgNode::ArgsAddBlock(n),
            ArgsAddBlockOrExpressionList::ExpressionList(es) => ArgNode::Exprs(es),
        };
        let id = match self.1 {
            IdentOrConst::Ident(i) => i,
            IdentOrConst::Const(c) => Ident(ident_tag, c.1, c.2),
        };

        MethodCall::new(
            vec![],
            Box::new(Expression::Ident(id)),
            false,
            normalize_args(arg_node),
        )
    }
}

def_tag!(assign_tag, "assign");
#[derive(Deserialize, Debug, Clone)]
pub struct Assign(pub assign_tag, pub Assignable, pub Box<Expression>);

def_tag!(massign_tag, "massign");
#[derive(Deserialize, Debug, Clone)]
pub struct MAssign(pub massign_tag, pub Vec<Assignable>, pub MRHS);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrVarField {
    Ident(Ident),
    VarField(VarField),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Assignable {
    VarField(VarField),
    ConstPathField(ConstPathField),
    RestParam(RestParam),
    TopConstField(TopConstField),
    ArefField(ArefField),
    Field(Field),
    // 2.6+
    Ident(Ident),
}

def_tag!(aref_field_tag, "aref_field");
#[derive(Deserialize, Debug, Clone)]
pub struct ArefField(pub aref_field_tag, pub Box<Expression>, pub ArgsAddBlock);

def_tag!(const_path_field_tag, "const_path_field");
#[derive(Deserialize, Debug, Clone)]
pub struct ConstPathField(pub const_path_field_tag, pub Box<Expression>, pub Const);

def_tag!(var_field_tag, "var_field");
#[derive(Deserialize, Debug, Clone)]
pub struct VarField(pub var_field_tag, pub VarRefType);

def_tag!(field_tag, "field");
#[derive(Deserialize, Debug, Clone)]
pub struct Field(
    pub field_tag,
    pub Box<Expression>,
    pub DotTypeOrOp,
    pub Ident,
);

def_tag!(var_ref_tag, "var_ref");
#[derive(Deserialize, Debug, Clone)]
pub struct VarRef(pub var_ref_tag, pub VarRefType);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VarRefType {
    GVar(GVar),
    IVar(IVar),
    CVar(CVar),
    Ident(Ident),
    Const(Const),
    Kw(Kw),
}

def_tag!(gvar_tag, "@gvar");
#[derive(Deserialize, Debug, Clone)]
pub struct GVar(pub gvar_tag, pub String, pub LineCol);

def_tag!(ivar_tag, "@ivar");
#[derive(Deserialize, Debug, Clone)]
pub struct IVar(pub ivar_tag, pub String, pub LineCol);

def_tag!(cvar_tag, "@cvar");
#[derive(Deserialize, Debug, Clone)]
pub struct CVar(pub cvar_tag, pub String, pub LineCol);

def_tag!(heredoc_string_literal_tag, "heredoc_string_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct HeredocStringLiteral(pub heredoc_string_literal_tag, pub (String, String));

def_tag!(string_literal_tag, "string_literal");
#[derive(Debug, Clone)]
pub struct StringLiteral(
    pub string_literal_tag,
    pub Option<HeredocStringLiteral>,
    pub StringContent,
);

impl<'de> Deserialize<'de> for StringLiteral {
    fn deserialize<D>(deserializer: D) -> Result<StringLiteral, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringLiteralVisitor;

        #[derive(Deserialize, Debug, Clone)]
        #[serde(untagged)]
        enum HeredocOrStringContent {
            Heredoc(HeredocStringLiteral),
            StringContent(StringContent),
        };

        impl<'de> de::Visitor<'de> for StringLiteralVisitor {
            type Value = StringLiteral;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "[string_literal, [heredoc]?, string_content]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = match seq.next_element()? {
                    Some(x) => x,
                    None => return Err(de::Error::custom("got no tag")),
                };

                if tag != "string_literal" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let mut h_or_sc: Option<HeredocOrStringContent> = seq.next_element()?;
                let h_or_sc =
                    h_or_sc.expect("didn't get either string content or heredoc in string literal");

                let sl = match h_or_sc {
                    HeredocOrStringContent::Heredoc(hd) => {
                        let sc: Option<StringContent> = seq.next_element()?;
                        let sc = sc.expect("didn't get string content in heredoc");
                        StringLiteral(string_literal_tag, Some(hd), sc)
                    }
                    HeredocOrStringContent::StringContent(sc) => {
                        StringLiteral(string_literal_tag, None, sc)
                    }
                };

                Ok(sl)
            }
        }

        deserializer.deserialize_seq(StringLiteralVisitor)
    }
}

def_tag!(xstring_literal_tag, "xstring_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct XStringLiteral(pub xstring_literal_tag, pub Vec<StringContentPart>);

def_tag!(dyna_symbol_tag, "dyna_symbol");
#[derive(Deserialize, Debug, Clone)]
pub struct DynaSymbol(pub dyna_symbol_tag, pub StringContentOrStringContentParts);

impl DynaSymbol {
    pub fn to_string_literal(self) -> StringLiteral {
        match self.1 {
            StringContentOrStringContentParts::StringContent(sc) => {
                StringLiteral(string_literal_tag, None, sc)
            }
            StringContentOrStringContentParts::StringContentParts(scp) => StringLiteral(
                string_literal_tag,
                None,
                StringContent(string_content_tag, scp),
            ),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringContentOrStringContentParts {
    StringContent(StringContent),
    StringContentParts(Vec<StringContentPart>),
}

def_tag!(tstring_content_tag, "@tstring_content");
#[derive(Deserialize, Debug, Clone)]
pub struct TStringContent(pub tstring_content_tag, pub String, pub LineCol);

def_tag!(string_embexpr_tag, "string_embexpr");
#[derive(Deserialize, Debug, Clone)]
pub struct StringEmbexpr(pub string_embexpr_tag, pub Vec<Expression>);

def_tag!(string_dvar_tag, "string_dvar");
#[derive(Deserialize, Debug, Clone)]
pub struct StringDVar(pub string_dvar_tag, pub Box<Expression>);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringContentPart {
    TStringContent(TStringContent),
    StringEmbexpr(StringEmbexpr),
    StringDVar(StringDVar),
}

def_tag!(string_content_tag, "string_content");
#[derive(Debug, Clone)]
pub struct StringContent(pub string_content_tag, pub Vec<StringContentPart>);

impl<'de> Deserialize<'de> for StringContent {
    fn deserialize<D>(deserializer: D) -> Result<StringContent, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringContentVisitor;

        impl<'de> de::Visitor<'de> for StringContentVisitor {
            type Value = StringContent;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(
                    f,
                    "[string_content, (tstring_content, string_embexpr, string_dvar)*]"
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = match seq.next_element()? {
                    Some(x) => x,
                    None => return Err(de::Error::custom("got no tag")),
                };
                if tag != "string_content" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let mut elements = vec![];
                let mut t_or_e: Option<StringContentPart> = seq.next_element()?;
                while t_or_e.is_some() {
                    elements.push(t_or_e.expect("we checked it's some"));
                    t_or_e = seq.next_element()?;
                }

                Ok(StringContent(string_content_tag, elements))
            }
        }

        deserializer.deserialize_seq(StringContentVisitor)
    }
}

def_tag!(array_tag, "array");
#[derive(Deserialize, Debug, Clone)]
pub struct Array(
    pub array_tag,
    pub SimpleArrayOrPercentArray,
    pub Option<LineCol>,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SimpleArrayOrPercentArray {
    SimpleArray(Option<ArgsAddStarOrExpressionList>),
    LowerPercentArray((String, Vec<TStringContent>, LineCol)),
    UpperPercentArray((String, Vec<Vec<StringContentPart>>, LineCol)),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgsAddStarOrExpressionList {
    ExpressionList(Vec<Expression>),
    ArgsAddStar(ArgsAddStar),
}

impl ArgsAddStarOrExpressionList {
    pub fn is_empty(&self) -> bool {
        match self {
            ArgsAddStarOrExpressionList::ExpressionList(el) => {
                if el.is_empty() {
                    return true;
                }
            }
            _ => {}
        };

        false
    }
}

def_tag!(args_add_star_tag, "args_add_star");
#[derive(Debug, Clone)]
pub struct ArgsAddStar(
    pub args_add_star_tag,
    pub Box<ArgsAddStarOrExpressionList>,
    pub Box<Expression>,
    pub Vec<Expression>,
);

impl<'de> Deserialize<'de> for ArgsAddStar {
    fn deserialize<D>(deserializer: D) -> Result<ArgsAddStar, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArgsAddStarVisitor;

        impl<'de> de::Visitor<'de> for ArgsAddStarVisitor {
            type Value = ArgsAddStar;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(
                    f,
                    "[args_add_star, [expression*], expression, expression*] or [args_add_star, [args_add_star, ...], expression, expression*"
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = match seq.next_element()? {
                    Some(x) => x,
                    None => return Err(de::Error::custom("got no tag")),
                };

                if tag != "args_add_star" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let left_expressions: ArgsAddStarOrExpressionList = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("didn't get array of expressions"))?;

                let star_expression: Expression = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("didn't get single star expression"))?;

                let mut right_expressions = vec![];
                let mut next_expression: Option<Expression> = seq.next_element()?;
                while next_expression.is_some() {
                    right_expressions.push(next_expression.expect("we checked it's some"));
                    next_expression = seq.next_element()?;
                }

                Ok(ArgsAddStar(
                    args_add_star_tag,
                    Box::new(left_expressions),
                    Box::new(star_expression),
                    right_expressions,
                ))
            }
        }

        deserializer.deserialize_seq(ArgsAddStarVisitor)
    }
}

def_tag!(alias_tag, "alias");
#[derive(Deserialize, Debug, Clone)]
pub struct Alias(pub alias_tag, pub SymbolLiteral, pub SymbolLiteral);

def_tag!(paren_expr_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct ParenExpr(pub paren_expr_tag, pub Vec<Expression>);

def_tag!(dot2_tag, "dot2");
#[derive(Deserialize, Debug, Clone)]
pub struct Dot2(
    pub dot2_tag,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(dot3_tag, "dot3");
#[derive(Deserialize, Debug, Clone)]
pub struct Dot3(
    pub dot3_tag,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(void_stmt_tag, "void_stmt");
#[derive(Deserialize, Debug, Clone)]
pub struct VoidStmt(pub (void_stmt_tag,));

// isn't parsable, but we do create it in our "normalized tree"
def_tag!(method_call_tag, "method_call");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodCall(
    pub method_call_tag,
    pub Vec<CallChainElement>,
    pub Box<Expression>,
    pub bool,
    pub ArgsAddStarOrExpressionList,
);

impl MethodCall {
    pub fn new(
        chain: Vec<CallChainElement>,
        method: Box<Expression>,
        use_parens: bool,
        args: ArgsAddStarOrExpressionList,
    ) -> Self {
        MethodCall(method_call_tag, chain, method, use_parens, args)
    }
}

def_tag!(def_tag, "def");
#[derive(Deserialize, Debug, Clone)]
pub struct Def(pub def_tag, pub IdentOrOp, pub ParenOrParams, pub BodyStmt);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrOp {
    Ident(Ident),
    Op((op_tag, String, LineCol)),
}

impl IdentOrOp {
    pub fn to_def_parts(self) -> (String, LineCol) {
        match self {
            Self::Ident(Ident(_, string, linecol)) => (string, linecol),
            Self::Op((_, string, linecol)) => (string, linecol),
        }
    }
}

def_tag!(begin_tag, "begin");
#[derive(Deserialize, Debug, Clone)]
pub struct Begin(pub begin_tag, pub BodyStmt);

def_tag!(bodystmt_tag, "bodystmt");
#[derive(Deserialize, Debug, Clone)]
pub struct BodyStmt(
    pub bodystmt_tag,
    pub Vec<Expression>,
    pub Option<Rescue>,
    pub Option<RescueElseOrExpressionList>,
    pub Option<Ensure>,
);

// deals with 2.6, where else is a vec expression and not an else
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RescueElseOrExpressionList {
    RescueElse(RescueElse),
    ExpressionList(Vec<Expression>),
}

def_tag!(rescue_tag, "rescue");
#[derive(Deserialize, Debug, Clone)]
pub struct Rescue(
    pub rescue_tag,
    pub Option<MRHS>,
    pub Option<Assignable>,
    pub Option<Vec<Expression>>,
    pub Option<Box<Rescue>>,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MRHS {
    Single(Box<Expression>),
    SingleAsArray(Vec<Expression>),
    MRHSNewFromArgs(MRHSNewFromArgs),
    MRHSAddStar(MRHSAddStar),
    Array(Array),
}

def_tag!(rescue_else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct RescueElse(pub rescue_else_tag, pub Option<Vec<Expression>>);

def_tag!(ensure_tag, "ensure");
#[derive(Deserialize, Debug, Clone)]
pub struct Ensure(pub ensure_tag, pub Option<Vec<Expression>>);

def_tag!(vcall);
#[derive(Deserialize, Debug, Clone)]
pub struct VCall(pub vcall, pub Box<Expression>);

def_tag!(command_call_tag, "command_call");
#[derive(Deserialize, Debug, Clone)]
pub struct CommandCall(
    command_call_tag,
    pub Box<Expression>,
    pub DotTypeOrOp,
    pub IdentOrConst,
    pub ArgNode,
);

impl CommandCall {
    pub fn to_method_call(self) -> MethodCall {
        let expr = match self.3 {
            IdentOrConst::Ident(i) => Box::new(Expression::Ident(i)),
            IdentOrConst::Const(c) => Box::new(Expression::Const(c)),
        };
        MethodCall::new(
            vec![
                CallChainElement::Expression(self.1),
                CallChainElement::Dot(self.2),
            ],
            expr,
            false,
            normalize_args(self.4),
        )
    }
}

def_tag!(const_tag, "@const");
#[derive(Deserialize, Debug, Clone)]
pub struct Const(pub const_tag, pub String, pub LineCol);

impl Const {
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

def_tag!(ident_tag, "@ident");
#[derive(Deserialize, Debug, Clone)]
pub struct Ident(pub ident_tag, pub String, pub LineCol);

impl Ident {
    pub fn new(s: String, l: LineCol) -> Self {
        Ident(ident_tag, s, l)
    }
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParenOrParams {
    Paren(Paren),
    Params(Params),
}

impl ParenOrParams {
    pub fn is_present(&self) -> bool {
        match self {
            ParenOrParams::Paren(p) => { p.is_present() },
            ParenOrParams::Params(p) => { p.is_present() },
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrMLhs {
    Ident(Ident),
    MLhs(MLhs),
}

def_tag!(paren_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct Paren(pub paren_tag, pub Params);

impl Paren {
    fn is_present(&self) -> bool {
        (self.1).is_present()
    }
}

def_tag!(params_tag, "params");
#[derive(Deserialize, Debug, Clone)]
pub struct Params(
    pub params_tag,
    pub Option<Vec<IdentOrMLhs>>,
    pub Option<Vec<(Ident, Expression)>>,
    pub Option<RestParamOr0OrExcessedComma>,
    pub Option<Vec<IdentOrMLhs>>,
    pub Option<Vec<(Label, ExpressionOrFalse)>>,
    pub Option<KwRestParam>,
    pub Option<BlockArg>,
);

impl Params {
    fn is_present(&self) -> bool {
        (self.1).is_some() ||
            (self.2).is_some() ||
            (self.3).is_some() ||
            (self.4).is_some() ||
            (self.5).is_some() ||
            (self.6).is_some() ||
            (self.7).is_some()
    }
}

// on ruby 2.5 and 2.6 the params lists for blocks (only), permit a trailing
// comma (presumably because of f params). Params lists for functions do
// not allow this.
//
// valid:
// ```ruby
// lambda { |x,| }
// lambda { |x, ;f }
// ```
// not valid:
// ``` ruby
// def foo(x,)
// end
// ```
//
// this causes the parser to parse the *params* node differently, even though
// the wrapping structure is a block var:
//
// on 2.5:
//
// rr 'lambda { |x,| }'
// [:program,
//  [[:method_add_block,
//    [:method_add_arg, [:fcall, [:@ident, "lambda", [1, 0]]], []],
//    [:brace_block,
//     [:block_var,
//      [:params, [[:@ident, "x", [1, 10]]], nil, 0, nil, nil, nil, nil],
//      false],
//     [[:void_stmt]]]]]]
// on 2.6:
//
// [:program,
//  [[:method_add_block,
//    [:method_add_arg, [:fcall, [:@ident, "lambda", [1, 0]]], []],
//    [:brace_block,
//     [:block_var,
//      [:params,
//       [[:@ident, "x", [1, 10]]],
//       nil,
//       [:excessed_comma],
//       nil,
//       nil,
//       nil,
//       nil],
//      false],
//     [[:void_stmt]]]]]]
// this difference is in the "rest_args" position, and on 2.5 is a literal
// integer 0 and on 2.6 a unit parser tag [:excessed_comma]. These nodes don't
// appear to cause any semantic difference in the program.
// So:
//   the Zero deserialzer deals with the 2.5 case, and the ExcessedComma node
//   deals with the 2.6 case, I will note that I tried to collapse them in to
//   a single representative node, but that didn't work with the serde setup
//   we have for some reason.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RestParamOr0OrExcessedComma {
    Zero(i64),
    RestParam(RestParam),
    ExcessedComma(ExcessedComma),
}

def_tag!(excessed_comma_tag, "excessed_comma");
#[derive(Deserialize, Debug, Clone)]
pub struct ExcessedComma((excessed_comma_tag,));

impl Params {
    pub fn non_null_positions(&self) -> Vec<bool> {
        vec![
            (self.1).is_some(),
            (self.2).is_some(),
            (self.3).is_some(),
            (self.4).is_some(),
            (self.5).is_some(),
            (self.6).is_some(),
            (self.7).is_some(),
        ]
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExpressionOrFalse {
    Expression(Expression),
    False(bool),
}

def_tag!(rest_param_tag, "rest_param");
#[derive(Deserialize, Debug, Clone)]
pub struct RestParam(pub rest_param_tag, pub Option<IdentOrVarField>);

def_tag!(kw_rest_param_tag, "kwrest_param");
#[derive(Deserialize, Debug, Clone)]
pub struct KwRestParam(pub kw_rest_param_tag, pub Option<Ident>);

def_tag!(blockarg_tag, "blockarg");
#[derive(Deserialize, Debug, Clone)]
pub struct BlockArg(pub blockarg_tag, pub Ident);

#[derive(Deserialize, Debug, Clone)]
pub struct LineCol(pub LineNumber, pub u64);

def_tag!(dotCall, "call");
#[derive(Deserialize, Debug, Clone)]
pub struct DotCall(pub dotCall);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CallExpr {
    FCall(FCall),
    Call(Call),
    Expression(Box<Expression>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CallChainElement {
    Expression(Box<Expression>),
    Dot(DotTypeOrOp),
}

def_tag!(method_add_arg_tag, "method_add_arg");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodAddArg(pub method_add_arg_tag, pub CallExpr, pub ArgNode);

pub fn normalize_inner_call(call_expr: CallExpr) -> (Vec<CallChainElement>, Box<Expression>) {
    match call_expr {
        CallExpr::FCall(FCall(_, i)) => {
            let id = match i {
                IdentOrConst::Ident(i) => i,
                IdentOrConst::Const(c) => Ident(ident_tag, c.1, c.2),
            };
            (vec![], Box::new(Expression::Ident(id)))
        }
        CallExpr::Call(Call(_, left, dot, right)) => {
            let (mut chain, method) = normalize_inner_call(CallExpr::Expression(left));
            chain.push(CallChainElement::Expression(method));
            chain.push(CallChainElement::Dot(dot));
            (chain, right)
        }
        CallExpr::Expression(e) => (Vec::new(), e),
    }
}

pub fn normalize_arg_paren(ap: ArgParen) -> ArgsAddStarOrExpressionList {
    match *ap.1 {
        ArgNode::Null(_) => ArgsAddStarOrExpressionList::ExpressionList(vec![]),
        ae => normalize_args(ae),
    }
}

pub fn normalize_args_add_block(aab: ArgsAddBlock) -> ArgsAddStarOrExpressionList {
    // .1 is expression list
    // .2 is block
    match aab.2 {
        ToProcExpr::NotPresent(_) => aab.1,
        ToProcExpr::Present(e) => {
            let trailing_expr_as_vec = vec![Expression::ToProc(ToProc(undeserializable, e))];

            match aab.1 {
                ArgsAddStarOrExpressionList::ExpressionList(items) => {
                    ArgsAddStarOrExpressionList::ExpressionList(
                        vec![items, trailing_expr_as_vec].concat(),
                    )
                }
                ArgsAddStarOrExpressionList::ArgsAddStar(aas) => {
                    let mut new_aas = aas;
                    new_aas.3 = vec![new_aas.3, trailing_expr_as_vec].concat();
                    ArgsAddStarOrExpressionList::ArgsAddStar(new_aas)
                }
            }
        }
    }
}

pub fn normalize_args(arg_node: ArgNode) -> ArgsAddStarOrExpressionList {
    match arg_node {
        ArgNode::ArgParen(ap) => normalize_arg_paren(ap),
        ArgNode::ArgsAddBlock(aab) => normalize_args_add_block(aab),
        ArgNode::ArgsAddStar(aas) => ArgsAddStarOrExpressionList::ArgsAddStar(aas),
        ArgNode::Exprs(exprs) => ArgsAddStarOrExpressionList::ExpressionList(exprs),
        ArgNode::Const(c) => {
            ArgsAddStarOrExpressionList::ExpressionList(vec![Expression::Const(c)])
        }
        ArgNode::Ident(c) => {
            ArgsAddStarOrExpressionList::ExpressionList(vec![Expression::Ident(c)])
        }
        ArgNode::Null(_) => panic!("should never be called with null"),
    }
}

impl MethodAddArg {
    pub fn to_method_call(self) -> MethodCall {
        let (chain, name) = normalize_inner_call(self.1);
        let args = normalize_args(self.2);
        MethodCall::new(chain, name, true, args)
    }
}

def_tag!(fcall_tag, "fcall");
#[derive(Deserialize, Debug, Clone)]
pub struct FCall(pub fcall_tag, pub IdentOrConst);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgNode {
    ArgParen(ArgParen),
    ArgsAddBlock(ArgsAddBlock),
    ArgsAddStar(ArgsAddStar),
    Exprs(Vec<Expression>),
    Const(Const),
    Ident(Ident),
    Null(Option<String>),
}

def_tag!(arg_paren_tag, "arg_paren");
#[derive(Deserialize, Debug, Clone)]
pub struct ArgParen(pub arg_paren_tag, pub Box<ArgNode>);

// See: https://dev.to/penelope_zone/understanding-ruby-s-block-proc-parsing-4a89
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ToProcExpr {
    NotPresent(bool),
    Present(Box<Expression>),
}

// ArgsAddBlock
def_tag!(args_add_block_tag, "args_add_block");
#[derive(Deserialize, Debug, Clone)]
pub struct ArgsAddBlock(
    pub args_add_block_tag,
    pub ArgsAddStarOrExpressionList,
    pub ToProcExpr,
);

def_tag!(int_tag, "@int");
#[derive(Deserialize, Debug, Clone)]
pub struct Int(pub int_tag, pub String, pub LineCol);

def_tag!(bare_assoc_hash_tag, "bare_assoc_hash");
#[derive(Deserialize, Debug, Clone)]
pub struct BareAssocHash(pub bare_assoc_hash_tag, pub Vec<AssocNewOrAssocSplat>);

def_tag!(hash_tag, "hash");
#[derive(Deserialize, Debug, Clone)]
pub struct Hash(pub hash_tag, pub Option<AssocListFromArgs>, pub LineCol);

def_tag!(assoclist_from_args_tag, "assoclist_from_args");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocListFromArgs(pub assoclist_from_args_tag, pub Vec<AssocNewOrAssocSplat>);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AssocNewOrAssocSplat {
    AssocNew(AssocNew),
    AssocSplat(AssocSplat),
}

def_tag!(assoc_new_tag, "assoc_new");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocNew(pub assoc_new_tag, pub AssocKey, pub Expression);

def_tag!(assoc_splat_tag, "assoc_splat");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocSplat(pub assoc_splat_tag, pub Expression);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AssocKey {
    Label(Label),
    Expression(Expression),
}

def_tag!(label_tag, "@label");
#[derive(Deserialize, Debug, Clone)]
pub struct Label(pub label_tag, pub String, pub LineCol);

def_tag!(symbol_literal_tag, "symbol_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct SymbolLiteral(pub symbol_literal_tag, pub SymbolOrBare);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SymbolOrBare {
    Ident(Ident),
    Op(Op),
    Symbol(Symbol),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrConst {
    Ident(Ident),
    Const(Const),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrConstOrKwOrOpOrIvar {
    Ident(Ident),
    Const(Const),
    Keyword(Kw),
    Op(Op),
    IVar(IVar),
}

def_tag!(symbol_tag, "symbol");
#[derive(Deserialize, Debug, Clone)]
pub struct Symbol(pub symbol_tag, pub IdentOrConstOrKwOrOpOrIvar);

def_tag!(call_tag, "call");
#[derive(Deserialize, Debug, Clone)]
pub struct Call(
    pub call_tag,
    pub Box<Expression>,
    pub DotTypeOrOp,
    pub Box<Expression>,
);

impl Call {
    pub fn to_method_call(self) -> MethodCall {
        let (chain, method) = normalize_inner_call(CallExpr::Call(self));
        MethodCall::new(
            chain,
            method,
            false,
            ArgsAddStarOrExpressionList::ExpressionList(Vec::new()),
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DotType {
    Dot(Dot),
    LonelyOperator(LonelyOperator),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DotTypeOrOp {
    DotType(DotType),
    Period(Period),
    ColonColon(ColonColon),
    Op(Op),
    StringDot(String),
}

def_tag!(period_tag, "@period");
#[derive(Deserialize, Debug, Clone)]
pub struct Period(pub period_tag, pub String, pub LineCol);

def_tag!(equals_tag, "==");
#[derive(Deserialize, Debug, Clone)]
pub struct Equals(pub equals_tag);

def_tag!(dot_tag, ".");
#[derive(Deserialize, Debug, Clone)]
pub struct Dot(pub dot_tag);

def_tag!(colon_colon_tag, "::");
#[derive(Deserialize, Debug, Clone)]
pub struct ColonColon(pub colon_colon_tag);

def_tag!(lonely_operator_tag, "&.");
#[derive(Deserialize, Debug, Clone)]
pub struct LonelyOperator(pub lonely_operator_tag);

def_tag!(op_tag, "@op");
#[derive(Deserialize, Debug, Clone)]
pub struct Op(pub op_tag, pub Operator, pub LineCol);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Operator {
    Equals(Equals),
    Dot(Dot),
    LonelyOperator(LonelyOperator),
    StringOperator(String),
}

def_tag!(opassign_tag, "opassign");
#[derive(Deserialize, Debug, Clone)]
pub struct OpAssign(
    pub opassign_tag,
    pub Assignable,
    pub Op,
    pub Box<Expression>,
);

def_tag!(next_tag, "next");
#[derive(Deserialize, Debug, Clone)]
pub struct Next(pub next_tag, pub ArgsAddBlockOrExpressionList);

def_tag!(if_mod_tag, "if_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct IfMod(pub if_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(unless_mod_tag, "unless_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct UnlessMod(pub unless_mod_tag, pub Box<Expression>, pub Box<Expression>);

#[derive(Debug, Clone)]
pub enum UnaryType {
    Not,
    Negative,
    Positive,
    BooleanNot,
}

def_tag!(unary_tag, "unary");
#[derive(Debug, Clone)]
pub struct Unary(pub unary_tag, pub UnaryType, pub Box<Expression>);

impl<'de> Deserialize<'de> for Unary {
    fn deserialize<D>(deserializer: D) -> Result<Unary, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UnaryVisitor;

        impl<'de> de::Visitor<'de> for UnaryVisitor {
            type Value = Unary;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "a unary operation")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tag: &str = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("didn't get array of expressions"))?;
                if tag != "unary" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let unary_type_string: &str = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("didn't get array of expressions"))?;
                let unary_type = match unary_type_string {
                    "not" => UnaryType::Not,
                    "-@" => UnaryType::Negative,
                    "+@" => UnaryType::Positive,
                    "!" => UnaryType::BooleanNot,
                    _ => panic!("got unknown unary type {}", unary_type_string),
                };

                let expression: Expression = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::custom("didn't get array of expressions"))?;
                Ok(Unary(unary_tag, unary_type, Box::new(expression)))
            }
        }

        deserializer.deserialize_seq(UnaryVisitor)
    }
}

def_tag!(super_tag, "super");
#[derive(Deserialize, Debug, Clone)]
pub struct Super(pub super_tag, pub ArgNode, pub LineCol);

impl Super {
    pub fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![],
            Box::new(Expression::Ident(Ident::new("super".to_string(), self.2))),
            true,
            normalize_args(self.1),
        )
    }
}

def_tag!(kw_tag, "@kw");
#[derive(Deserialize, Debug, Clone)]
pub struct Kw(pub kw_tag, pub String, pub LineCol);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ConstPathRefOrConstRef {
    ConstPathRef(ConstPathRef),
    ConstRef(ConstRef),
}

def_tag!(class_tag, "class");
#[derive(Deserialize, Debug, Clone)]
pub struct Class(
    pub class_tag,
    pub ConstPathRefOrConstRef,
    pub Option<Box<Expression>>,
    pub BodyStmt,
);

def_tag!(module_tag, "module");
#[derive(Deserialize, Debug, Clone)]
pub struct Module(pub module_tag, pub ConstPathRefOrConstRef, pub BodyStmt);

def_tag!(defs_tag, "defs");
#[derive(Deserialize, Debug, Clone)]
pub struct Defs(
    pub defs_tag,
    pub Singleton,
    pub DotOrColon,
    pub IdentOrKw,
    pub ParenOrParams,
    pub BodyStmt,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IdentOrKw {
    Ident(Ident),
    Kw(Kw),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Singleton {
    VarRef(VarRef),
    Paren(ParenExpr),
}

// can only occur in defs, Op is always `::`
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DotOrColon {
    Period(Period),
    Op(Operator),
}

def_tag!(binary_tag, "binary");
#[derive(Deserialize, Debug, Clone)]
pub struct Binary(
    pub binary_tag,
    pub Box<Expression>,
    pub String,
    pub Box<Expression>,
);

def_tag!(float_tag, "@float");
#[derive(Deserialize, Debug, Clone)]
pub struct Float(float_tag, pub String, pub LineCol);

def_tag!(aref_tag, "aref");
#[derive(Deserialize, Debug, Clone)]
pub struct Aref(aref_tag, pub Box<Expression>, pub Option<ArgNode>);

def_tag!(char_tag, "@CHAR");
#[derive(Deserialize, Debug, Clone)]
pub struct Char(char_tag, pub String, pub LineCol);

def_tag!(return_tag, "return");
#[derive(Deserialize, Debug, Clone)]
pub struct Return(return_tag, pub ArgNode, pub LineCol);

def_tag!(return0_tag, "return0");
#[derive(Deserialize, Debug, Clone)]
pub struct Return0((return0_tag,));

def_tag!(regexp_literal_tag, "regexp_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct RegexpLiteral(
    regexp_literal_tag,
    pub Vec<StringContentPart>,
    pub RegexpEnd,
);

def_tag!(regexp_end_tag, "@regexp_end");
#[derive(Deserialize, Debug, Clone)]
pub struct RegexpEnd(regexp_end_tag, pub String, pub LineCol, pub String);

def_tag!(backref_tag, "@backref");
#[derive(Deserialize, Debug, Clone)]
pub struct Backref(backref_tag, pub String, pub LineCol);

def_tag!(yield_tag, "yield");
#[derive(Deserialize, Debug, Clone)]
pub struct Yield(yield_tag, pub ParenOrArgsAddBlock, pub LineCol);

def_tag!(break_tag, "break");
#[derive(Deserialize, Debug, Clone)]
pub struct Break(break_tag, pub ParenOrArgsAddBlock, pub LineCol);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ParenOrArgsAddBlock {
    YieldParen(YieldParen),
    ArgsAddBlock(ArgsAddBlock),
    Empty(Vec<()>),
}

def_tag!(yield_paren_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct YieldParen(yield_paren_tag, pub Box<ArgNode>);

def_tag!(method_add_block_tag, "method_add_block");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodAddBlock(method_add_block_tag, pub CallType, pub Block);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CallType {
    MethodAddArg(MethodAddArg),
    Call(Call),
    CommandCall(CommandCall),
    Command(Command),
}

impl CallType {
    pub fn to_method_call(self) -> MethodCall {
        match self {
            Self::MethodAddArg(maa) => maa.to_method_call(),
            Self::Call(call) => call.to_method_call(),
            Self::CommandCall(cc) => cc.to_method_call(),
            Self::Command(command) => command.to_method_call(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Block {
    BraceBlock(BraceBlock),
    DoBlock(DoBlock),
}

// block local variables are a nightmare, they can be false, nil, or an array
// of idents:
//
// 1. nil if params are not present, and block local variables are also not
//    specified
// 2. false if params are present, and block local variables are not present
// 3. a vec of idents either if params are or are not present, and block local
//    variables are present
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum BlockLocalVariables {
    EmptyBecauseParamsWerePresent(bool),
    NilBecauseParamsWereNotPresent(Option<()>),
    Present(Vec<Ident>),
}

def_tag!(block_var_tag, "block_var");
#[derive(Deserialize, Debug, Clone)]
pub struct BlockVar(block_var_tag, pub Option<Params>, pub BlockLocalVariables);

def_tag!(do_block_tag, "do_block");
#[derive(Deserialize, Debug, Clone)]
pub struct DoBlock(do_block_tag, pub Option<BlockVar>, pub BodyStmt);

def_tag!(brace_block_tag, "brace_block");
#[derive(Deserialize, Debug, Clone)]
pub struct BraceBlock(brace_block_tag, pub Option<BlockVar>, pub Vec<Expression>);

def_tag!(while_tag, "while");
#[derive(Deserialize, Debug, Clone)]
pub struct While(while_tag, pub Box<Expression>, pub Vec<Expression>);

def_tag!(until_tag, "until");
#[derive(Deserialize, Debug, Clone)]
pub struct Until(until_tag, pub Box<Expression>, pub Vec<Expression>);

def_tag!(while_mod_tag, "while_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct WhileMod(while_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(until_mod_tag, "until_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct UntilMod(until_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(case_tag, "case");
#[derive(Deserialize, Debug, Clone)]
pub struct Case(case_tag, pub Option<Box<Expression>>, pub When, pub LineCol);

def_tag!(when_tag, "when");
#[derive(Deserialize, Debug, Clone)]
pub struct When(
    when_tag,
    pub ArgsAddStarOrExpressionList,
    pub Vec<Expression>,
    pub Option<Box<WhenOrElse>>,
    pub LineCol,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WhenOrElse {
    When(When),
    Else(CaseElse),
}

def_tag!(case_else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct CaseElse(case_else_tag, pub Vec<Expression>);

def_tag!(retry_tag, "retry");
#[derive(Deserialize, Debug, Clone)]
pub struct Retry((retry_tag,));

def_tag!(sclass_tag, "sclass");
#[derive(Deserialize, Debug, Clone)]
pub struct SClass(sclass_tag, pub Box<Expression>, pub BodyStmt);

// some constructs were expressionlist in 2.5 and bodystmt in 2.6 so this
// deals with both cases
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExpressionListOrBodyStmt {
    ExpresionList(Vec<Expression>),
    BodyStmt(BodyStmt),
}

def_tag!(stabby_lambda_tag, "lambda");
#[derive(Deserialize, Debug, Clone)]
pub struct StabbyLambda(
    stabby_lambda_tag,
    pub ParenOrParams,
    pub String,
    pub ExpressionListOrBodyStmt,
    pub LineCol,
);

def_tag!(imaginary_tag, "@imaginary");
#[derive(Deserialize, Debug, Clone)]
pub struct Imaginary(imaginary_tag, pub String, pub LineCol);

def_tag!(rational_tag, "@rational");
#[derive(Deserialize, Debug, Clone)]
pub struct Rational(rational_tag, pub String, pub LineCol);

def_tag!(for_tag, "for");
#[derive(Deserialize, Debug, Clone)]
pub struct For(
    for_tag,
    pub VarFieldOrVarFields,
    pub Box<Expression>,
    pub Vec<Expression>,
);

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VarFieldOrVarFields {
    VarField(VarField),
    VarFields(Vec<VarField>),
}

// ternary
def_tag!(ifop_tag, "ifop");
#[derive(Deserialize, Debug, Clone)]
pub struct IfOp(
    ifop_tag,
    pub Box<Expression>,
    pub Box<Expression>,
    pub Box<Expression>,
);
