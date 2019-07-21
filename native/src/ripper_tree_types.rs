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
        struct $tag_name;

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

def_tag!(program);
#[derive(Deserialize, Debug)]
pub struct Program(program, pub Vec<Expression>);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Expression {
    Def(Def),
    BodyStmt(BodyStmt),
    VCall(VCall),
    Ident(Ident),
    Param(Param),
    MethodCall(MethodCall),
}

// isn't parsable, but we do create it in our "normalized tree"
def_tag!(method_call);
#[derive(Deserialize, Debug)]
pub struct MethodCall(
    method_call,
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
        MethodCall(method_call, chain, method, use_parens, args)
    }
}

def_tag!(def);
#[derive(Deserialize, Debug)]
pub struct Def(def, pub Ident, pub Param, pub BodyStmt);

def_tag!(bodystmt);
#[derive(Deserialize, Debug)]
pub struct BodyStmt(
    bodystmt,
    pub Vec<Expression>,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
    pub Option<Box<Expression>>,
);

def_tag!(vcall);
#[derive(Deserialize, Debug)]
pub struct VCall(vcall, pub Box<Expression>);

def_tag!(ident, "@ident");
#[derive(Deserialize, Debug)]
pub struct Ident(ident, pub String, pub LineCol);

impl Ident {
    pub fn line_number(&self) -> LineNumber {
        (self.2).0
    }
}

def_tag!(params);
#[derive(Deserialize, Debug)]
pub struct Param(params, Value, Value, Value, Value, Value, Value, Value);

#[derive(Deserialize, Debug)]
pub struct LineCol(pub LineNumber, pub u64);
