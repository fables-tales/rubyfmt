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
#[derive(Deserialize, Debug)]
pub struct Program(pub program_tag, pub Vec<Expression>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Expression {
    Def(Def),
    BodyStmt(BodyStmt),
    VCall(VCall),
    Ident(Ident),
    Params(Params),
    MethodCall(MethodCall),
    DotCall(DotCall),
    MethodAddArg(MethodAddArg),
    Int(Int),
}

// isn't parsable, but we do create it in our "normalized tree"
def_tag!(method_call_tag, "method_call");
#[derive(Deserialize, Debug)]
pub struct MethodCall(
    pub method_call_tag,
    pub Vec<Expression>,
    pub Box<Expression>,
    pub bool,
    pub Vec<Expression>,
);

impl MethodCall {
    pub fn new(
        chain: Vec<Expression>,
        method: Box<Expression>,
        use_parens: bool,
        args: Vec<Expression>,
    ) -> Self {
        MethodCall(method_call_tag, chain, method, use_parens, args)
    }
}

def_tag!(def_tag, "def");
#[derive(Deserialize, Debug)]
pub struct Def(pub def_tag, pub Ident, pub Params, pub BodyStmt);

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

def_tag!(ident_tag, "@ident");
#[derive(Deserialize, Debug)]
pub struct Ident(pub ident_tag, pub String, pub LineCol);

impl Ident {
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

def_tag!(params_tag, "params");
#[derive(Deserialize, Debug)]
pub struct Params(
    pub params_tag,
    pub Value,
    pub Value,
    pub Value,
    pub Value,
    pub Value,
    pub Value,
    pub Value,
);

#[derive(Deserialize, Debug)]
pub struct LineCol(pub LineNumber, pub u64);

def_tag!(dotCall, "call");
#[derive(Deserialize, Debug)]
pub struct DotCall(pub dotCall);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum CallExpr {
    FCall(FCall),
}

def_tag!(method_add_arg_tag, "method_add_arg");
#[derive(Deserialize, Debug)]
pub struct MethodAddArg(pub method_add_arg_tag, pub CallExpr, pub ArgExpr);

pub fn normalize_inner_call(call_expr: CallExpr) -> (Vec<Expression>, Box<Expression>) {
    match call_expr {
        CallExpr::FCall(FCall(_, i)) => (vec![], Box::new(Expression::Ident(i))),
    }
}

pub fn normalize_arg_paren(ap: ArgParen) -> Vec<Expression> {
    match *ap.1 {
        ArgExpr::Null(_) => vec![],
        ae => normalize_args(ae),
    }
}

pub fn normalize_args_add_block(aab: ArgsAddBlock) -> Vec<Expression> {
    match aab.2 {
        MaybeBlock::NoBlock(_) => normalize_args(*aab.1),
    }
}

pub fn normalize_args(arg_expr: ArgExpr) -> Vec<Expression> {
    match arg_expr {
        ArgExpr::ArgParen(ap) => normalize_arg_paren(ap),
        ArgExpr::ArgsAddBlock(aab) => normalize_args_add_block(aab),
        ArgExpr::ExpressionList(expr) => expr,
        ArgExpr::Null(_) => panic!("should never be called with null"),
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
pub enum ArgExpr {
    ArgParen(ArgParen),
    ArgsAddBlock(ArgsAddBlock),
    ExpressionList(Vec<Expression>),
    Null(Option<String>),
}

def_tag!(arg_paren_tag, "arg_paren");
#[derive(Deserialize, Debug)]
pub struct ArgParen(pub arg_paren_tag, pub Box<ArgExpr>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeBlock {
    NoBlock(bool),
}

def_tag!(args_add_block_tag, "args_add_block");
#[derive(Deserialize, Debug)]
pub struct ArgsAddBlock(pub args_add_block_tag, pub Box<ArgExpr>, pub MaybeBlock);

def_tag!(int_tag, "@int");
#[derive(Deserialize, Debug)]
pub struct Int(pub int_tag, pub String, pub LineCol);
