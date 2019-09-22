#![allow(warnings)]
use serde::*;
use serde_json::Value;

use crate::types::LineNumber;

macro_rules! def_tag {
    ($tag_name:ident) => {
        def_tag!($tag_name, stringify!($tag_name));
    };

    ($tag_name:ident, $tag:expr) => {
        #[derive(Serialize, Debug)]
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
                            println!("accepgted at {}", s);
                            Ok(())
                        } else {
                            println!("rejected at {}", s);
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
#[derive(Deserialize, Debug)]
pub struct Program(pub program_tag, pub Vec<Expression>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Expression {
    VoidStmt(VoidStmt),
    Def(Def),
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
    Const(Const),
    Command(Command),
    ConstPathRef(ConstPathRef),
    Defined(Defined),
    TopConstRef(TopConstRef),
}

def_tag!(defined_tag, "defined");
#[derive(Deserialize, Debug)]
pub struct Defined(pub defined_tag, pub Box<Expression>);

def_tag!(top_const_ref_tag, "top_const_ref");
#[derive(Deserialize, Debug)]
pub struct TopConstRef(pub top_const_ref_tag, pub Const);

def_tag!(const_path_ref_tag, "const_path_ref");
#[derive(Deserialize, Debug)]
pub struct ConstPathRef(pub const_path_ref_tag, pub Box<Expression>, pub Const);

def_tag!(command_tag, "command");
#[derive(Deserialize, Debug)]
pub struct Command(pub command_tag, pub Ident, pub ArgsAddBlock);

impl Command {
    pub fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![],
            Box::new(Expression::Ident(self.1)),
            false,
            normalize_args(ArgNode::ArgsAddBlock(self.2)),
        )
    }
}

def_tag!(assign_tag, "assign");
#[derive(Deserialize, Debug)]
pub struct Assign(
    pub assign_tag,
    pub VarFieldOrConstField,
    pub Box<Expression>,
);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum VarFieldOrConstField {
    VarField(VarField),
    ConstPathField(ConstPathField),
}

def_tag!(const_path_field_tag, "const_path_field");
#[derive(Deserialize, Debug)]
pub struct ConstPathField(pub const_path_field_tag, pub Box<Expression>, pub Const);

def_tag!(var_field_tag, "var_field");
#[derive(Deserialize, Debug)]
pub struct VarField(pub var_field_tag, pub VarRefType);

def_tag!(var_ref_tag, "var_ref");
#[derive(Deserialize, Debug)]
pub struct VarRef(pub var_ref_tag, pub VarRefType);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum VarRefType {
    GVar(GVar),
    IVar(IVar),
    CVar(CVar),
    Ident(Ident),
    Const(Const),
}

def_tag!(gvar_tag, "@gvar");
#[derive(Deserialize, Debug)]
pub struct GVar(pub gvar_tag, pub String, pub LineCol);

def_tag!(ivar_tag, "@ivar");
#[derive(Deserialize, Debug)]
pub struct IVar(pub ivar_tag, pub String, pub LineCol);

def_tag!(cvar_tag, "@cvar");
#[derive(Deserialize, Debug)]
pub struct CVar(pub cvar_tag, pub String, pub LineCol);

def_tag!(string_literal_tag, "string_literal");
#[derive(Deserialize, Debug)]
pub struct StringLiteral(pub string_literal_tag, pub StringContent);

def_tag!(xstring_literal_tag, "xstring_literal");
#[derive(Deserialize, Debug)]
pub struct XStringLiteral(pub xstring_literal_tag, pub Vec<StringContentPart>);

def_tag!(dyna_symbol_tag, "dyna_symbol");
#[derive(Deserialize, Debug)]
pub struct DynaSymbol(pub dyna_symbol_tag, pub StringContent);

def_tag!(tstring_content_tag, "@tstring_content");
#[derive(Deserialize, Debug)]
pub struct TStringContent(pub tstring_content_tag, pub String, pub LineCol);

def_tag!(string_embexpr_tag, "string_embexpr");
#[derive(Deserialize, Debug)]
pub struct StringEmbexpr(pub string_embexpr_tag, pub Vec<Expression>);

def_tag!(string_dvar_tag, "string_dvar");
#[derive(Deserialize, Debug)]
pub struct StringDVar(pub string_dvar_tag, pub Box<Expression>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StringContentPart {
    TStringContent(TStringContent),
    StringEmbexpr(StringEmbexpr),
    StringDVar(StringDVar),
}

def_tag!(string_content_tag, "string_content");
#[derive(Debug)]
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
                let tag: &str = seq.next_element()?.ok_or_else(|| panic!("what"))?;
                if tag != "string_content" {
                    return Err(de::Error::custom("didn't get right tag"));
                }

                let mut elements = vec![];
                let mut t_or_e: Option<StringContentPart> = seq.next_element()?;
                println!("{:?}", t_or_e);
                while t_or_e.is_some() {
                    elements.push(t_or_e.expect("we checked it's some"));
                    t_or_e = seq.next_element()?;
                    println!("{:?}", t_or_e);
                }

                Ok(StringContent(string_content_tag, elements))
            }
        }

        deserializer.deserialize_seq(StringContentVisitor)
    }
}

def_tag!(array_tag, "array");
#[derive(Deserialize, Debug)]
pub struct Array(pub array_tag, pub SimpleArrayOrPercentArray);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SimpleArrayOrPercentArray {
    SimpleArray(Option<ArgsAddStarOrExpressionList>),
    PercentArray((String, Vec<TStringContent>)),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgsAddStarOrExpressionList {
    ExpressionList(Vec<Expression>),
    ArgsAddStar(ArgsAddStar),
}

def_tag!(args_add_star_tag, "args_add_star");
#[derive(Debug)]
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
                let tag: &str = seq.next_element()?.ok_or_else(|| panic!("what"))?;
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
#[derive(Deserialize, Debug)]
pub struct Alias(pub alias_tag, pub SymbolLiteral, pub SymbolLiteral);

def_tag!(paren_expr_tag, "paren");
#[derive(Deserialize, Debug)]
pub struct ParenExpr(pub paren_expr_tag, pub Vec<Expression>);

def_tag!(dot2_tag, "dot2");
#[derive(Deserialize, Debug)]
pub struct Dot2(
    pub dot2_tag,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(dot3_tag, "dot3");
#[derive(Deserialize, Debug)]
pub struct Dot3(
    pub dot3_tag,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(void_stmt_tag, "void_stmt");
#[derive(Deserialize, Debug)]
pub struct VoidStmt(pub (void_stmt_tag,));

// isn't parsable, but we do create it in our "normalized tree"
def_tag!(method_call_tag, "method_call");
#[derive(Deserialize, Debug)]
pub struct MethodCall(
    pub method_call_tag,
    pub Vec<CallChainElement>,
    pub Box<Expression>,
    pub bool,
    pub Vec<Expression>,
);

impl MethodCall {
    pub fn new(
        chain: Vec<CallChainElement>,
        method: Box<Expression>,
        use_parens: bool,
        args: Vec<Expression>,
    ) -> Self {
        MethodCall(method_call_tag, chain, method, use_parens, args)
    }
}

def_tag!(def_tag, "def");
#[derive(Deserialize, Debug)]
pub struct Def(pub def_tag, pub Ident, pub ParenOrParams, pub BodyStmt);

def_tag!(begin_tag, "begin");
#[derive(Deserialize, Debug)]
pub struct Begin(pub begin_tag, pub BodyStmt);

def_tag!(bodystmt_tag, "bodystmt");
#[derive(Deserialize, Debug)]
pub struct BodyStmt(
    pub bodystmt_tag,
    pub Vec<Expression>,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(vcall);
#[derive(Deserialize, Debug)]
pub struct VCall(vcall, pub Box<Expression>);

def_tag!(command_call_tag, "command_call");
#[derive(Deserialize, Debug)]
pub struct CommandCall(
    command_call_tag,
    pub Box<Expression>,
    pub DotTypeOrOp,
    pub Ident,
    pub ArgNode,
);

impl CommandCall {
    pub fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![
                CallChainElement::Expression(self.1),
                CallChainElement::Dot(self.2),
            ],
            Box::new(Expression::Ident(self.3)),
            false,
            normalize_args(self.4),
        )
    }
}

def_tag!(const_tag, "@const");
#[derive(Deserialize, Debug)]
pub struct Const(pub const_tag, pub String, pub LineCol);

impl Const {
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

def_tag!(ident_tag, "@ident");
#[derive(Deserialize, Debug)]
pub struct Ident(pub ident_tag, pub String, pub LineCol);

impl Ident {
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ParenOrParams {
    Paren(Paren),
    Params(Params),
}

def_tag!(paren_tag, "paren");
#[derive(Deserialize, Debug)]
pub struct Paren(pub paren_tag, pub Params);

def_tag!(params_tag, "params");
#[derive(Deserialize, Debug)]
pub struct Params(
    pub params_tag,
    pub Option<Vec<Ident>>,
    pub Option<Vec<(Ident, Expression)>>,
    pub Option<RestParam>,
    pub Option<Vec<Ident>>,
    pub Option<Vec<(Label, ExpressionOrFalse)>>,
    pub Option<KwRestParam>,
    pub Option<BlockArg>,
);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ExpressionOrFalse {
    Expression(Expression),
    False(bool),
}

def_tag!(rest_param_tag, "rest_param");
#[derive(Deserialize, Debug)]
pub struct RestParam(pub rest_param_tag, pub Option<Ident>);

def_tag!(kw_rest_param_tag, "kwrest_param");
#[derive(Deserialize, Debug)]
pub struct KwRestParam(pub kw_rest_param_tag, pub Option<Ident>);

def_tag!(blockarg_tag, "blockarg");
#[derive(Deserialize, Debug)]
pub struct BlockArg(pub blockarg_tag, pub Ident);

#[derive(Deserialize, Debug)]
pub struct LineCol(pub LineNumber, pub u64);

def_tag!(dotCall, "call");
#[derive(Deserialize, Debug)]
pub struct DotCall(pub dotCall);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum CallExpr {
    FCall(FCall),
    Call(Call),
    Expression(Box<Expression>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum CallChainElement {
    Expression(Box<Expression>),
    Dot(DotTypeOrOp),
}

def_tag!(method_add_arg_tag, "method_add_arg");
#[derive(Deserialize, Debug)]
pub struct MethodAddArg(pub method_add_arg_tag, pub CallExpr, pub ArgNode);

pub fn normalize_inner_call(call_expr: CallExpr) -> (Vec<CallChainElement>, Box<Expression>) {
    match call_expr {
        CallExpr::FCall(FCall(_, i)) => (vec![], Box::new(Expression::Ident(i))),
        CallExpr::Call(Call(_, left, dot, right)) => {
            let (mut chain, method) = normalize_inner_call(CallExpr::Expression(left));
            chain.push(CallChainElement::Expression(method));
            chain.push(CallChainElement::Dot(dot));
            (chain, right)
        }
        CallExpr::Expression(e) => (Vec::new(), e),
    }
}

pub fn normalize_arg_paren(ap: ArgParen) -> Vec<Expression> {
    match *ap.1 {
        ArgNode::Null(_) => vec![],
        ae => normalize_args(ae),
    }
}

pub fn normalize_args_add_block(aab: ArgsAddBlock) -> Vec<Expression> {
    // .1 is expression list
    // .2 is block
    match aab.2 {
        MaybeBlock::NoBlock(_) => aab.1,
        MaybeBlock::ToProcExpr(e) => vec![*e],
    }
}

pub fn normalize_args(arg_node: ArgNode) -> Vec<Expression> {
    match arg_node {
        ArgNode::ArgParen(ap) => normalize_arg_paren(ap),
        ArgNode::ArgsAddBlock(aab) => normalize_args_add_block(aab),
        ArgNode::Exprs(exprs) => exprs,
        ArgNode::Const(c) => vec![Expression::Const(c)],
        ArgNode::Ident(c) => vec![Expression::Ident(c)],
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
#[derive(Deserialize, Debug)]
pub struct FCall(pub fcall_tag, pub Ident);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgNode {
    ArgParen(ArgParen),
    ArgsAddBlock(ArgsAddBlock),
    Exprs(Vec<Expression>),
    Const(Const),
    Ident(Ident),
    Null(Option<String>),
}

def_tag!(arg_paren_tag, "arg_paren");
#[derive(Deserialize, Debug)]
pub struct ArgParen(pub arg_paren_tag, pub Box<ArgNode>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeBlock {
    NoBlock(bool),
    ToProcExpr(Box<Expression>),
}

def_tag!(args_add_block_tag, "args_add_block");
#[derive(Deserialize, Debug)]
pub struct ArgsAddBlock(pub args_add_block_tag, pub Vec<Expression>, pub MaybeBlock);

def_tag!(int_tag, "@int");
#[derive(Deserialize, Debug)]
pub struct Int(pub int_tag, pub String, pub LineCol);

def_tag!(bare_assoc_hash_tag, "bare_assoc_hash");
#[derive(Deserialize, Debug)]
pub struct BareAssocHash(pub bare_assoc_hash_tag, pub Vec<AssocNewOrAssocSplat>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AssocNewOrAssocSplat {
    AssocNew(AssocNew),
    AssocSplat(AssocSplat),
}

def_tag!(assoc_new_tag, "assoc_new");
#[derive(Deserialize, Debug)]
pub struct AssocNew(
    pub assoc_new_tag,
    pub LabelOrSymbolLiteralOrDynaSymbol,
    pub Expression,
);

def_tag!(assoc_splat_tag, "assoc_splat");
#[derive(Deserialize, Debug)]
pub struct AssocSplat(pub assoc_splat_tag, pub Expression);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LabelOrSymbolLiteralOrDynaSymbol {
    Label(Label),
    SymbolLiteral(SymbolLiteral),
    DynaSymbol(DynaSymbol),
}

def_tag!(label_tag, "@label");
#[derive(Deserialize, Debug)]
pub struct Label(pub label_tag, pub String, pub LineCol);

def_tag!(symbol_literal_tag, "symbol_literal");
#[derive(Deserialize, Debug)]
pub struct SymbolLiteral(pub symbol_literal_tag, pub SymbolOrBare);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SymbolOrBare {
    Ident(Ident),
    Op(Op),
    Symbol(Symbol),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IdentOrConst {
    Ident(Ident),
    Const(Const),
}

def_tag!(symbol_tag, "symbol");
#[derive(Deserialize, Debug)]
pub struct Symbol(pub symbol_tag, pub IdentOrConst);

def_tag!(call_tag, "call");
#[derive(Deserialize, Debug)]
pub struct Call(
    pub call_tag,
    pub Box<Expression>,
    pub DotTypeOrOp,
    pub Box<Expression>,
);

impl Call {
    pub fn to_method_call(self) -> MethodCall {
        let (chain, method) = normalize_inner_call(CallExpr::Call(self));
        MethodCall::new(chain, method, false, Vec::new())
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Operator {
    Equals(Equals),
    Dot(Dot),
    LonelyOperator(LonelyOperator),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DotType {
    Dot(Dot),
    LonelyOperator(LonelyOperator),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DotTypeOrOp {
    DotType(DotType),
    Period(Period),
    ColonColon(ColonColon),
    Op(Op),
}

def_tag!(period_tag, "@period");
#[derive(Deserialize, Debug)]
pub struct Period(pub period_tag, pub String, pub LineCol);

def_tag!(equals_tag, "==");
#[derive(Deserialize, Debug)]
pub struct Equals(pub equals_tag);

def_tag!(dot_tag, ".");
#[derive(Deserialize, Debug)]
pub struct Dot(pub dot_tag);

def_tag!(colon_colon_tag, "::");
#[derive(Deserialize, Debug)]
pub struct ColonColon(pub colon_colon_tag);

def_tag!(lonely_operator_tag, "&.");
#[derive(Deserialize, Debug)]
pub struct LonelyOperator(pub lonely_operator_tag);

def_tag!(op_tag, "@op");
#[derive(Deserialize, Debug)]
pub struct Op(pub op_tag, pub Operator, pub LineCol);
