#![allow(clippy::wrong_self_convention)]

#[cfg(debug_assertions)]
use log::debug;
use ripper_deserialize::RipperDeserialize;
use serde::*;

use crate::types::LineNumber;

fn ident_as_cc(i: String, start_end: &StartEnd) -> CallChainElement {
    CallChainElement::IdentOrOpOrKeywordOrConst(IdentOrOpOrKeywordOrConst::Ident(Ident::new(
        i,
        LineCol::from_line(start_end.0),
    )))
}

fn args_as_cc(an: ArgNode, start_end: Option<StartEnd>) -> CallChainElement {
    CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(normalize_args(an), start_end)
}

macro_rules! def_tag {
    ($tag_name:ident) => {
        def_tag!($tag_name, stringify!($tag_name));
    };

    ($tag_name:ident, $tag:expr) => {
        #[derive(Serialize, Debug, Clone)]
        #[allow(non_camel_case_types)]
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
                            #[cfg(debug_assertions)]
                            {
                                debug!("accepted at {:?} {:?}", s, $tag);
                            }
                            Ok(())
                        } else {
                            #[cfg(debug_assertions)]
                            {
                                debug!("rejected at {:?} {:?}", s, $tag);
                            }
                            Err(E::custom("mismatched tag"))
                        }
                    }
                }

                let _tag = deserializer.deserialize_str(TagVisitor)?;
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

#[derive(RipperDeserialize, Debug, Clone)]
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
    Params(Box<Params>),
    MethodCall(MethodCall),
    Call(Call),
    CommandCall(CommandCall),
    MethodAddArg(MethodAddArg),
    Int(Int),
    BareAssocHash(BareAssocHash),
    Symbol(Symbol),
    SymbolLiteral(SymbolLiteral),
    DynaSymbol(DynaSymbol),
    Begin(Begin),
    BeginBlock(BeginBlock),
    EndBlock(EndBlock),
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
    Redo(Redo),
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

impl Expression {
    pub fn is_constant_reference(&self) -> bool {
        use Expression::*;
        matches!(
            self,
            VarRef(..) | TopConstRef(..) | Ident(..) | Const(..) | ConstPathRef(..)
        )
    }

    /// This _is_ a lot of boilerplate, but there's some method to this madness.
    /// Notably, not all expressions have a "trustworthy" `StartEnd`. There's an
    /// important distinction between which of these contructs that come from the parser
    /// and which are lexical constructions. For example, a hash will have an accurate
    /// StartEnd because it's beginning and end are clearly defined in the grammar.
    ///
    /// However, some things -- take a Command for example -- are ambiguous by design.
    /// In the case of Commands, since they don't use parens, their StartEnd will continue
    /// all the way until the next expression, since technically a method argument could
    /// be anywhere in the interim, so it's StartEnd is far longer than the actual call.
    ///
    /// Many such constructs here are unreliable and thus use a proxy instead, e.g. the first
    /// nested expression, the nearest identifier, etc. These are broadly grouped into "categories",
    /// but there are unfortunately many cases of ambiguity, hence this gigantic match statement
    /// (and the similar functions on other related structs).
    pub fn start_line(&self) -> Option<u64> {
        match self {
            // Expressions with a StartEnd (ideally most/all of them would end up here)
            Expression::Class(Class(.., start_end))
            | Expression::If(If(.., start_end))
            | Expression::VCall(VCall(.., start_end))
            | Expression::Def(Def(.., start_end))
            | Expression::Defs(Defs(.., start_end))
            | Expression::SymbolLiteral(SymbolLiteral(.., start_end))
            | Expression::DynaSymbol(DynaSymbol(.., start_end))
            | Expression::Begin(Begin(_, start_end, ..))
            | Expression::Array(Array(.., start_end))
            | Expression::Next(Next(.., start_end))
            | Expression::Super(Super(.., start_end))
            | Expression::Module(Module(.., start_end))
            | Expression::Return(Return(.., start_end))
            | Expression::Return0(Return0(_, start_end))
            | Expression::Hash(Hash(.., start_end))
            | Expression::Yield(Yield(.., start_end))
            | Expression::While(While(.., start_end))
            | Expression::Case(Case(.., start_end))
            | Expression::Retry(Retry(.., start_end))
            | Expression::Redo(Redo(.., start_end))
            | Expression::SClass(SClass(.., start_end))
            | Expression::Break(Break(.., start_end))
            | Expression::StabbyLambda(StabbyLambda(.., start_end))
            | Expression::Unless(Unless(.., start_end))
            | Expression::ZSuper(ZSuper(.., start_end))
            | Expression::Yield0(Yield0(.., start_end)) => Some(start_end.start_line()),
            // Expressions with a LineCol
            Expression::TopConstRef(TopConstRef(_, Const(.., linecol)))
            | Expression::Ident(Ident(.., linecol))
            | Expression::Int(Int(.., linecol))
            | Expression::Const(Const(.., linecol))
            | Expression::Kw(Kw(.., linecol))
            | Expression::Float(Float(.., linecol))
            | Expression::Char(Char(.., linecol))
            | Expression::Backref(Backref(.., linecol))
            | Expression::Imaginary(Imaginary(.., linecol))
            | Expression::Rational(Rational(.., linecol)) => Some(linecol.0),
            // Expressions with locations defined by nested expressions
            Expression::RescueMod(RescueMod(_, expr, _))
            | Expression::ToProc(ToProc(_, expr))
            | Expression::Unary(Unary(.., expr))
            | Expression::ConstPathRef(ConstPathRef(_, expr, ..))
            | Expression::Defined(Defined(.., expr))
            | Expression::Binary(Binary(_, expr, ..))
            | Expression::WhileMod(WhileMod(_, expr, ..))
            | Expression::UntilMod(UntilMod(_, expr, ..))
            | Expression::IfMod(IfMod(_, expr, ..))
            | Expression::UnlessMod(UnlessMod(_, expr, ..))
            | Expression::Until(Until(_, expr, ..))
            | Expression::For(For(_, _, expr, _))
            | Expression::IfOp(IfOp(_, expr, ..)) => expr.start_line(),
            // Miscellaneous expressions with special cases
            Expression::VoidStmt(..) => None,
            Expression::Paren(ParenExpr(.., paren_expr, _)) => paren_expr.start_line(),
            Expression::MLhs(MLhs(mlhs_inners)) => {
                mlhs_inners.first().and_then(|mlhs| mlhs.start_line())
            }
            Expression::MethodAddBlock(MethodAddBlock(_, call_left, ..)) => call_left.start_line(),
            Expression::RegexpLiteral(RegexpLiteral(_, string_content_parts, _)) => {
                string_content_parts
                    .first()
                    .and_then(|scp| scp.start_line())
            }
            Expression::OpAssign(OpAssign(_, assignable, ..)) => assignable.start_line(),
            Expression::Params(params) => Some(params.as_ref().8.start_line()),
            Expression::MethodCall(MethodCall(_, call_chain_elements, ..)) => {
                call_chain_elements.first().and_then(|cce| cce.start_line())
            }
            Expression::MethodAddArg(MethodAddArg(_, call_left, ..))
            | Expression::CommandCall(CommandCall(_, call_left, ..))
            | Expression::Call(Call(_, call_left, ..)) => call_left.start_line(),
            Expression::BareAssocHash(BareAssocHash(_, assocs)) => {
                assocs.first().and_then(|a| a.start_line())
            }
            Expression::Symbol(Symbol(_, symbol_type)) => symbol_type.start_line(),
            Expression::BeginBlock(BeginBlock(_, exprs))
            | Expression::EndBlock(EndBlock(_, exprs)) => {
                exprs.first().and_then(|expr| expr.start_line())
            }
            // Pick the first of either expression, since these can be e.g. `foo..bar` or `foo..` or `..bar`
            Expression::Dot2(Dot2(_, maybe_first_expr, maybe_second_expr))
            | Expression::Dot3(Dot3(_, maybe_first_expr, maybe_second_expr)) => maybe_first_expr
                .as_ref()
                .map(|mfe| mfe.start_line())
                .or_else(|| maybe_second_expr.as_ref().map(|mse| mse.start_line()))
                .flatten(),
            Expression::Alias(Alias(_, symbol, ..)) => Some(symbol.start_line()),
            Expression::StringLiteral(string_literal) => Some(string_literal.start_line()),
            Expression::XStringLiteral(XStringLiteral(_, string_parts)) => {
                string_parts.first().and_then(|sp| sp.start_line())
            }
            Expression::VarRef(VarRef(_, var_ref_type)) => Some(var_ref_type.start_line()),
            Expression::Assign(Assign(_, assignable, ..)) => assignable.start_line(),
            Expression::MAssign(MAssign(_, assignable_list_or_mlhs, ..)) => {
                assignable_list_or_mlhs.start_line()
            }
            Expression::Command(Command(_, ident_or_const, ..)) => {
                Some(ident_or_const.start_line())
            }
            Expression::MRHSAddStar(MRHSAddStar(_, mrhs, ..)) => mrhs.start_line(),
            Expression::StringConcat(StringConcat(_, concat_or_literal, ..)) => {
                concat_or_literal.start_line()
            }
            Expression::Undef(Undef(_, symbol_literals)) => {
                symbol_literals.first().map(|s| s.start_line())
            }
            // Arefs only have an accurate closing line and not a starting line,
            // so don't use it here
            Expression::Aref(Aref(_, expr, ..)) => expr.start_line(),
        }
    }
}

def_tag!(mlhs_tag, "mlhs");
#[derive(Debug, Clone)]
pub struct MLhs(pub Vec<MLhsInner>);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum MLhsInner {
    VarField(VarField),
    Field(Field),
    RestParam(RestParam),
    Ident(Ident),
    MLhs(Box<MLhs>),
}

impl MLhsInner {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            MLhsInner::VarField(VarField(_, var_ref_type)) => Some(var_ref_type.start_line()),
            MLhsInner::Field(Field(_, expr, ..)) => expr.start_line(),
            MLhsInner::RestParam(RestParam(.., rest_param_assignable)) => rest_param_assignable
                .as_ref()
                .and_then(|rpa| rpa.start_line()),
            MLhsInner::Ident(Ident(_, _, linecol)) => Some(linecol.0),
            MLhsInner::MLhs(mlhs) => mlhs
                .as_ref()
                .0
                .first()
                .and_then(|mlhs_inner| mlhs_inner.start_line()),
        }
    }
}

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
                seq.next_element::<mlhs_tag>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(&mut seq)).map(MLhs)
            }
        }

        deserializer.deserialize_seq(MLhsVisitor)
    }
}

def_tag!(zsuper_tag, "zsuper");
#[derive(Deserialize, Debug, Clone)]
pub struct ZSuper(pub zsuper_tag, pub StartEnd);

impl ZSuper {
    fn into_call_chain(self) -> Vec<CallChainElement> {
        vec![ident_as_cc("super".to_string(), &self.1)]
    }
}

def_tag!(yield0_tag, "yield0");
#[derive(Deserialize, Debug, Clone)]
pub struct Yield0(pub yield0_tag, pub StartEnd);

impl Yield0 {
    fn into_call_chain(self) -> Vec<CallChainElement> {
        vec![ident_as_cc("yield".to_string(), &self.1)]
    }
}

def_tag!(if_tag, "if");
#[derive(Deserialize, Debug, Clone)]
pub struct If(
    pub if_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub Option<ElsifOrElse>,
    pub StartEnd,
);

def_tag!(unless_tag, "unless");
#[derive(Deserialize, Debug, Clone)]
pub struct Unless(
    pub unless_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub Option<Else>,
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
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
    pub StartEnd,
);

def_tag!(else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct Else(pub else_tag, pub Vec<Expression>, pub StartEnd);

def_tag!(undef_tag, "undef");
#[derive(Deserialize, Debug, Clone)]
pub struct Undef(pub undef_tag, pub Vec<SymbolLiteralOrDynaSymbol>);

def_tag!(string_concat_tag, "string_concat");
#[derive(Deserialize, Debug, Clone)]
pub struct StringConcat(
    pub string_concat_tag,
    pub StringConcatOrStringLiteral,
    pub StringLiteral,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum StringConcatOrStringLiteral {
    StringConcat(Box<StringConcat>),
    StringLiteral(StringLiteral),
}

impl StringConcatOrStringLiteral {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            StringConcatOrStringLiteral::StringConcat(sc) => sc.1.start_line(),
            StringConcatOrStringLiteral::StringLiteral(sl) => Some(sl.start_line()),
        }
    }
}

def_tag!(mrhs_add_star_tag, "mrhs_add_star");
#[derive(Deserialize, Debug, Clone)]
pub struct MRHSAddStar(
    pub mrhs_add_star_tag,
    pub MRHSNewFromArgsOrEmpty,
    pub Box<Expression>,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum MRHSNewFromArgsOrEmpty {
    MRHSNewFromArgs(MRHSNewFromArgs),
    Empty(Vec<Expression>),
}

impl MRHSNewFromArgsOrEmpty {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            MRHSNewFromArgsOrEmpty::MRHSNewFromArgs(MRHSNewFromArgs(_, args_add_star, ..)) => {
                args_add_star.start_line()
            }
            MRHSNewFromArgsOrEmpty::Empty(exprs) => exprs.first().and_then(|e| e.start_line()),
        }
    }
}

def_tag!(mrhs_new_from_args_tag, "mrhs_new_from_args");
#[derive(Deserialize, Debug, Clone)]
pub struct MRHSNewFromArgs(
    pub mrhs_new_from_args_tag,
    pub ArgsAddStarOrExpressionListOrArgsForward,
    #[serde(default)]
    /// This will be none if only two expressions are given and the last is a
    /// splat. For example, `rescue A, *B`
    pub Option<Box<Expression>>,
);

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

impl Command {
    pub fn into_call_chain(self) -> Vec<CallChainElement> {
        let io = (self.1).into_ident_or_op_or_keyword_or_const();
        let (s, lc) = io.to_def_parts();
        vec![
            ident_as_cc(s, &StartEnd(lc.0, lc.0)),
            CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(
                normalize_args_add_block_or_expression_list(self.2),
                None,
            ),
        ]
    }
}

impl ToMethodCall for Command {
    fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![],
            (self.1).into_ident_or_op_or_keyword_or_const(),
            false,
            normalize_args_add_block_or_expression_list(self.2),
            None,
        )
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ArgsAddBlockOrExpressionList {
    ArgsAddBlock(ArgsAddBlock),
    ExpressionList(Vec<Expression>),
}

def_tag!(assign_tag, "assign");
#[derive(Deserialize, Debug, Clone)]
pub struct Assign(
    pub assign_tag,
    pub Assignable,
    pub ExpressionOrMRHSNewFromArgs,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ExpressionOrMRHSNewFromArgs {
    Expression(Box<Expression>),
    MRHSNewFromArgs(MRHSNewFromArgs),
}

def_tag!(massign_tag, "massign");
#[derive(Deserialize, Debug, Clone)]
pub struct MAssign(pub massign_tag, pub AssignableListOrMLhs, pub MRHSOrArray);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum AssignableListOrMLhs {
    AssignableList(Vec<Assignable>),
    MLhs(MLhs),
}

impl AssignableListOrMLhs {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            AssignableListOrMLhs::AssignableList(al) => {
                al.first().and_then(|assignable| assignable.start_line())
            }
            AssignableListOrMLhs::MLhs(mlhs) => mlhs.0.first().and_then(|inner| inner.start_line()),
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum MRHSOrArray {
    MRHS(MRHS),
    Array(Array),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum RestParamAssignable {
    Ident(Ident),
    VarField(VarField),
    ArefField(ArefField),
}

impl RestParamAssignable {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            RestParamAssignable::ArefField(ArefField(.., linecol))
            | RestParamAssignable::Ident(Ident(.., linecol)) => Some(linecol.0),
            RestParamAssignable::VarField(VarField(.., var_ref_type)) => {
                Some(var_ref_type.start_line())
            }
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum Assignable {
    VarField(VarField),
    ConstPathField(ConstPathField),
    RestParam(RestParam),
    TopConstField(TopConstField),
    ArefField(ArefField),
    Field(Field),
    MLhs(MLhs),
    // 2.6+
    Ident(Ident),
}

impl Assignable {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            Assignable::VarField(VarField(.., var_ref_type)) => Some(var_ref_type.start_line()),
            Assignable::RestParam(RestParam(.., rest_param_assignable)) => rest_param_assignable
                .as_ref()
                .and_then(|rpa| rpa.start_line()),
            Assignable::ConstPathField(ConstPathField(.., Const(.., linecol)))
            | Assignable::Ident(Ident(.., linecol))
            | Assignable::TopConstField(TopConstField(.., Const(.., linecol))) => Some(linecol.0),
            Assignable::ArefField(ArefField(_, expr, ..)) => expr.start_line(),
            Assignable::Field(Field(_, expr, ..)) => expr.start_line(),
            Assignable::MLhs(MLhs(mlhs_inners)) => mlhs_inners
                .first()
                .and_then(|mlhs_inner| mlhs_inner.start_line()),
        }
    }
}

def_tag!(begin_block, "BEGIN");
#[derive(Deserialize, Debug, Clone)]
pub struct BeginBlock(pub begin_block, pub Vec<Expression>);

def_tag!(end_block, "END");
#[derive(Deserialize, Debug, Clone)]
pub struct EndBlock(pub end_block, pub Vec<Expression>);

def_tag!(aref_field_tag, "aref_field");
#[derive(Deserialize, Debug, Clone)]
pub struct ArefField(
    pub aref_field_tag,
    pub Box<Expression>,
    pub Option<ArgsAddBlockOrExpressionList>,
    pub LineCol,
);

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
    pub IdentOrConst,
);

def_tag!(var_ref_tag, "var_ref");
#[derive(Deserialize, Debug, Clone)]
pub struct VarRef(pub var_ref_tag, pub VarRefType);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum VarRefType {
    GVar(GVar),
    IVar(IVar),
    CVar(CVar),
    Ident(Ident),
    Const(Const),
    Kw(Kw),
}

impl VarRefType {
    pub fn start_line(&self) -> u64 {
        match self {
            VarRefType::GVar(GVar(.., linecol))
            | VarRefType::IVar(IVar(.., linecol))
            | VarRefType::CVar(CVar(.., linecol))
            | VarRefType::Ident(Ident(.., linecol))
            | VarRefType::Const(Const(.., linecol))
            | VarRefType::Kw(Kw(.., linecol)) => linecol.0,
        }
    }

    pub fn to_local_string(self) -> String {
        match self {
            VarRefType::GVar(v) => v.1,
            VarRefType::IVar(v) => v.1,
            VarRefType::CVar(v) => v.1,
            VarRefType::Ident(v) => v.1,
            VarRefType::Const(v) => v.1,
            VarRefType::Kw(v) => v.1,
        }
    }
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
pub struct HeredocStringLiteral(
    pub heredoc_string_literal_tag,
    pub (String, String),
    pub StartEnd,
);

def_tag!(string_literal_tag, "string_literal");
#[derive(RipperDeserialize, Debug, Clone)]
pub enum StringLiteral {
    Normal(string_literal_tag, StringContent, StartEnd),
    Heredoc(string_literal_tag, HeredocStringLiteral, StringContent),
}

impl StringLiteral {
    pub fn start_line(&self) -> u64 {
        match self {
            StringLiteral::Heredoc(_, HeredocStringLiteral(.., start_end), ..)
            | StringLiteral::Normal(.., start_end) => start_end.start_line(),
        }
    }
}

def_tag!(xstring_literal_tag, "xstring_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct XStringLiteral(pub xstring_literal_tag, pub Vec<StringContentPart>);

def_tag!(dyna_symbol_tag, "dyna_symbol");
#[derive(Deserialize, Debug, Clone)]
pub struct DynaSymbol(
    pub dyna_symbol_tag,
    pub StringContentOrStringContentParts,
    StartEnd,
);

impl DynaSymbol {
    pub fn to_string_literal(self) -> StringLiteral {
        match self.1 {
            StringContentOrStringContentParts::StringContent(sc) => {
                StringLiteral::Normal(string_literal_tag, sc, self.2)
            }
            StringContentOrStringContentParts::StringContentParts(scp) => StringLiteral::Normal(
                string_literal_tag,
                StringContent(string_content_tag, scp),
                self.2,
            ),
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
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

#[derive(RipperDeserialize, Debug, Clone)]
pub enum StringContentPart {
    TStringContent(TStringContent),
    StringEmbexpr(StringEmbexpr),
    StringDVar(StringDVar),
}

impl StringContentPart {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            StringContentPart::TStringContent(TStringContent(.., linecol)) => Some(linecol.0),
            StringContentPart::StringEmbexpr(StringEmbexpr(_, exprs)) => {
                exprs.first().and_then(|expr| expr.start_line())
            }
            StringContentPart::StringDVar(StringDVar(_, expr)) => expr.as_ref().start_line(),
        }
    }
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
                let tag = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let elements =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(&mut seq))?;
                Ok(StringContent(tag, elements))
            }
        }

        deserializer.deserialize_seq(StringContentVisitor)
    }
}

def_tag!(array_tag, "array");
#[derive(Deserialize, Debug, Clone)]
pub struct Array(pub array_tag, pub SimpleArrayOrPercentArray, pub StartEnd);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum SimpleArrayOrPercentArray {
    SimpleArray(Option<ArgsAddStarOrExpressionListOrArgsForward>),
    LowerPercentArray((String, Vec<TStringContent>, LineCol)),
    UpperPercentArray((String, Vec<Vec<StringContentPart>>, LineCol)),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ArgsAddStarOrExpressionListOrArgsForward {
    ExpressionList(Vec<Expression>),
    ArgsAddStar(ArgsAddStar),
    ArgsForward(ArgsForward),
}

impl ArgsAddStarOrExpressionListOrArgsForward {
    pub fn is_empty(&self) -> bool {
        if let ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el, ..) = self {
            if el.is_empty() {
                return true;
            }
        }

        false
    }

    pub fn empty() -> Self {
        ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![])
    }

    pub fn start_line(&self) -> Option<u64> {
        match self {
            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(exprs) => {
                exprs.first().and_then(|e| e.start_line())
            }
            ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(ArgsAddStar(_, aas, ..)) => {
                aas.start_line()
            }
            ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(ArgsForward(..)) => None,
        }
    }
}

def_tag!(args_add_star_tag, "args_add_star");
#[derive(Debug, Clone)]
pub struct ArgsAddStar(
    pub args_add_star_tag,
    pub Box<ArgsAddStarOrExpressionListOrArgsForward>,
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
                let (tag, left_expressions, star_expression) =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(&mut seq))?;
                let right_expressions =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(&mut seq))?;

                Ok(ArgsAddStar(
                    tag,
                    left_expressions,
                    star_expression,
                    right_expressions,
                ))
            }
        }

        deserializer.deserialize_seq(ArgsAddStarVisitor)
    }
}

def_tag!(alias_tag, "alias");
#[derive(Deserialize, Debug, Clone)]
pub struct Alias(
    pub alias_tag,
    pub SymbolLiteralOrDynaSymbol,
    pub SymbolLiteralOrDynaSymbol,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum SymbolLiteralOrDynaSymbol {
    DynaSymbol(DynaSymbol),
    SymbolLiteral(SymbolLiteral),
}

impl SymbolLiteralOrDynaSymbol {
    pub fn start_line(&self) -> u64 {
        match self {
            SymbolLiteralOrDynaSymbol::DynaSymbol(DynaSymbol(.., start_end))
            | SymbolLiteralOrDynaSymbol::SymbolLiteral(SymbolLiteral(.., start_end)) => {
                start_end.start_line()
            }
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ParenExpressionOrExpressions {
    Expressions(Vec<Expression>),
    Expression(Box<Expression>),
}

impl ParenExpressionOrExpressions {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            ParenExpressionOrExpressions::Expressions(exprs) => {
                exprs.first().and_then(|e| e.start_line())
            }
            ParenExpressionOrExpressions::Expression(expr) => expr.as_ref().start_line(),
        }
    }
}

def_tag!(paren_expr_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct ParenExpr(
    pub paren_expr_tag,
    pub ParenExpressionOrExpressions,
    pub StartEnd,
);

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
pub struct VoidStmt(void_stmt_tag);

def_tag!(def_tag, "def");
#[derive(Deserialize, Debug, Clone)]
pub struct Def(
    pub def_tag,
    pub IdentOrOpOrKeywordOrConst,
    pub ParenOrParams,
    pub Box<DefBodyStmt>,
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum IdentOrOpOrKeywordOrConst {
    Ident(Ident),
    Op((op_tag, String, LineCol)),
    Keyword(Kw),
    Const(Const),
}

impl IdentOrOpOrKeywordOrConst {
    pub fn to_def_parts(self) -> (String, LineCol) {
        match self {
            Self::Ident(Ident(_, string, linecol)) => (string, linecol),
            Self::Op((_, string, linecol)) => (string, linecol),
            Self::Keyword(Kw(_, string, linecol)) => (string, linecol),
            Self::Const(Const(_, string, linecol)) => (string, linecol),
        }
    }

    pub fn into_ident(self) -> Ident {
        let (s, lc) = self.to_def_parts();
        Ident::new(s, lc)
    }

    pub fn get_name(&self) -> String {
        self.clone().to_def_parts().0
    }
}

def_tag!(begin_tag, "begin");
#[derive(Deserialize, Debug, Clone)]
pub struct Begin(pub begin_tag, pub StartEnd, pub Box<BodyStmt>);

def_tag!(bodystmt_tag, "bodystmt");
#[derive(Deserialize, Debug, Clone)]
pub struct BodyStmt(
    pub bodystmt_tag,
    pub Vec<Expression>,
    pub Option<Rescue>,
    pub Option<RescueElseOrExpressionList>,
    pub Option<Ensure>,
);

// The bodystmt for endless defs
#[derive(Deserialize, Debug, Clone)]
pub struct EndlessBodyStmt(pub bodystmt_tag, pub Expression);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum DefBodyStmt {
    EndBodyStmt(BodyStmt),
    EndlessBodyStmt(EndlessBodyStmt),
}

// deals with 2.6, where else is a vec expression and not an else
#[derive(RipperDeserialize, Debug, Clone)]
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
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum MRHS {
    Single(Box<Expression>),
    SingleAsArray(Vec<Expression>),
    MRHSNewFromArgs(MRHSNewFromArgs),
    MRHSAddStar(MRHSAddStar),
    Array(Array),
}

def_tag!(rescue_else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct RescueElse(
    pub rescue_else_tag,
    pub Option<Vec<Expression>>,
    pub StartEnd,
);

def_tag!(ensure_tag, "ensure");
#[derive(Deserialize, Debug, Clone)]
pub struct Ensure(pub ensure_tag, pub Option<Vec<Expression>>, pub StartEnd);

def_tag!(const_tag, "@const");
#[derive(Deserialize, Debug, Clone)]
pub struct Const(pub const_tag, pub String, pub LineCol);

def_tag!(ident_tag, "@ident");
#[derive(Deserialize, Debug, Clone)]
pub struct Ident(pub ident_tag, pub String, pub LineCol);

impl Ident {
    pub fn new(s: String, l: LineCol) -> Self {
        Ident(ident_tag, s, l)
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ParenOrParams {
    Paren(Paren),
    Params(Box<Params>),
}

impl ParenOrParams {
    pub fn is_present(&self) -> bool {
        match self {
            ParenOrParams::Paren(p) => p.is_present(),
            ParenOrParams::Params(p) => p.is_present(),
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum IdentOrMLhs {
    Ident(Ident),
    MLhs(MLhs),
}

def_tag!(paren_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct Paren(pub paren_tag, pub Box<Params>, pub StartEnd);

impl Paren {
    fn is_present(&self) -> bool {
        (self.1).is_present()
    }
}

def_tag!(block_tag, "&");

#[derive(RipperDeserialize, Debug, Clone)]
pub enum BlockArgOrTag {
    BlockArg(BlockArg),
    Tag(block_tag),
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
    pub Option<KwRestParamOrArgsForward>,
    pub Option<BlockArgOrTag>,
    pub StartEnd,
);

impl Params {
    fn is_present(&self) -> bool {
        (self.1).is_some()
            || (self.2).is_some()
            || (self.3).is_some()
            || (self.4).is_some()
            || (self.5).is_some()
            || (self.6).is_some()
            || (self.7).is_some()
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
#[derive(RipperDeserialize, Debug, Clone)]
pub enum RestParamOr0OrExcessedComma {
    Zero(i64),
    RestParam(RestParam),
    ExcessedComma(ExcessedComma),
}

def_tag!(excessed_comma_tag, "excessed_comma");
#[derive(Deserialize, Debug, Clone)]
pub struct ExcessedComma(excessed_comma_tag);

def_tag!(args_forward_tag, "args_forward");
#[derive(Deserialize, Debug, Clone)]
pub struct ArgsForward(args_forward_tag);

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

#[derive(RipperDeserialize, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ExpressionOrFalse {
    Expression(Expression),
    False(bool),
}

def_tag!(rest_param_tag, "rest_param");
#[derive(Deserialize, Debug, Clone)]
pub struct RestParam(pub rest_param_tag, pub Option<RestParamAssignable>);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum KwRestParamOrArgsForward {
    KwRestParam(KwRestParam),
    ArgsForward(ArgsForward),
}

def_tag!(kw_rest_param_tag, "kwrest_param");
#[derive(Deserialize, Debug, Clone)]
pub struct KwRestParam(pub kw_rest_param_tag, pub Option<Ident>);

def_tag!(blockarg_tag, "blockarg");
#[derive(Deserialize, Debug, Clone)]
pub struct BlockArg(pub blockarg_tag, pub Ident);

#[derive(Deserialize, Debug, Clone)]
pub struct LineCol(pub LineNumber, pub u64);

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct StartEnd(pub LineNumber, pub LineNumber);

impl StartEnd {
    pub fn start_line(&self) -> LineNumber {
        self.0
    }

    pub fn end_line(&self) -> LineNumber {
        self.1
    }

    pub fn unknown() -> Self {
        StartEnd(0, 0)
    }
}

impl LineCol {
    fn unknown() -> Self {
        LineCol(0, 0)
    }

    fn from_line(ln: LineNumber) -> Self {
        LineCol(ln, 0)
    }
}

pub fn normalize_arg_paren(ap: ArgParen) -> ArgsAddStarOrExpressionListOrArgsForward {
    match *ap.1 {
        ArgNode::Null(_) => ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![]),
        ae => normalize_args(ae),
    }
}

pub fn normalize_args_add_block_or_expression_list(
    aab: ArgsAddBlockOrExpressionList,
) -> ArgsAddStarOrExpressionListOrArgsForward {
    match aab {
        ArgsAddBlockOrExpressionList::ExpressionList(el) => {
            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el)
        }
        ArgsAddBlockOrExpressionList::ArgsAddBlock(aab) => normalize_args_add_block(aab),
    }
}
pub fn normalize_args_add_block(aab: ArgsAddBlock) -> ArgsAddStarOrExpressionListOrArgsForward {
    // .1 is expression list
    // .2 is block
    match aab.2 {
        ToProcExpr::NotPresent(_) => (aab.1).into_args_add_star_or_expression_list(),
        ToProcExpr::Present(e) => {
            let trailing_expr_as_vec = vec![Expression::ToProc(ToProc(undeserializable, e))];

            match (aab.1).into_args_add_star_or_expression_list() {
                ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(items) => {
                    ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(
                        vec![items, trailing_expr_as_vec].concat(),
                    )
                }
                ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(aas) => {
                    let mut new_aas = aas;
                    new_aas.3 = vec![new_aas.3, trailing_expr_as_vec].concat();
                    ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(new_aas)
                }
                ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(af) => {
                    ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(af)
                }
            }
        }
    }
}

pub fn normalize_args(arg_node: ArgNode) -> ArgsAddStarOrExpressionListOrArgsForward {
    match arg_node {
        ArgNode::ArgParen(ap) => normalize_arg_paren(ap),
        ArgNode::ArgsAddBlock(aab) => normalize_args_add_block(aab),
        ArgNode::ArgsAddStar(aas) => ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(aas),
        ArgNode::Exprs(exprs) => ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(exprs),
        ArgNode::Const(c) => {
            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![Expression::Const(c)])
        }
        ArgNode::Ident(c) => {
            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![Expression::Ident(c)])
        }
        ArgNode::ArgsForward(af) => ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(af),
        ArgNode::Null(_) => panic!("should never be called with null"),
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ArgNode {
    ArgParen(ArgParen),
    ArgsAddBlock(ArgsAddBlock),
    ArgsAddStar(ArgsAddStar),
    ArgsForward(ArgsForward),
    Exprs(Vec<Expression>),
    Const(Const),
    Ident(Ident),
    Null(Option<String>),
}

def_tag!(arg_paren_tag, "arg_paren");
#[derive(Deserialize, Debug, Clone)]
pub struct ArgParen(pub arg_paren_tag, pub Box<ArgNode>, pub StartEnd);

// See: https://dev.to/penelope_zone/understanding-ruby-s-block-proc-parsing-4a89
#[derive(RipperDeserialize, Debug, Clone)]
pub enum ToProcExpr {
    NotPresent(bool),
    Present(Box<Expression>),
}

// ArgsAddBlock
def_tag!(args_add_block_tag, "args_add_block");
#[derive(Deserialize, Debug, Clone)]
pub struct ArgsAddBlock(
    pub args_add_block_tag,
    pub ArgsAddBlockInner,
    pub ToProcExpr,
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum AABParen {
    Paren((paren_tag, Box<Expression>)),
    EmptyParen((paren_tag, bool)),
    Expression(Box<Expression>),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ArgsAddBlockInner {
    Parens(Vec<AABParen>),
    ArgsAddStarOrExpressionListOrArgsForward(ArgsAddStarOrExpressionListOrArgsForward),
}

impl ArgsAddBlockInner {
    pub fn into_args_add_star_or_expression_list(self) -> ArgsAddStarOrExpressionListOrArgsForward {
        match self {
            ArgsAddBlockInner::ArgsAddStarOrExpressionListOrArgsForward(a) => a,
            ArgsAddBlockInner::Parens(ps) => {
                let el = ps
                    .into_iter()
                    .filter(|aabp| !matches!(aabp, AABParen::EmptyParen(..)))
                    .map(|aabp| match aabp {
                        AABParen::Paren(p) => *p.1,
                        AABParen::Expression(e) => *e,
                        AABParen::EmptyParen(..) => {
                            unreachable!("We should have already filtered these out")
                        }
                    })
                    .collect();
                ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el)
            }
        }
    }
}

def_tag!(int_tag, "@int");
#[derive(Deserialize, Debug, Clone)]
pub struct Int(pub int_tag, pub String, pub LineCol);

def_tag!(bare_assoc_hash_tag, "bare_assoc_hash");
#[derive(Deserialize, Debug, Clone)]
pub struct BareAssocHash(pub bare_assoc_hash_tag, pub Vec<AssocNewOrAssocSplat>);

def_tag!(hash_tag, "hash");
#[derive(Deserialize, Debug, Clone)]
pub struct Hash(pub hash_tag, pub Option<AssocListFromArgs>, pub StartEnd);

def_tag!(assoclist_from_args_tag, "assoclist_from_args");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocListFromArgs(pub assoclist_from_args_tag, pub Vec<AssocNewOrAssocSplat>);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum AssocNewOrAssocSplat {
    AssocNew(Box<AssocNew>),
    AssocSplat(Box<AssocSplat>),
}

impl AssocNewOrAssocSplat {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            AssocNewOrAssocSplat::AssocNew(assoc_new) => assoc_new.as_ref().1.start_line(),
            AssocNewOrAssocSplat::AssocSplat(assoc_splat) => assoc_splat.as_ref().1.start_line(),
        }
    }
}

def_tag!(assoc_new_tag, "assoc_new");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocNew(pub assoc_new_tag, pub AssocKey, pub Option<Expression>);

def_tag!(assoc_splat_tag, "assoc_splat");
#[derive(Deserialize, Debug, Clone)]
pub struct AssocSplat(pub assoc_splat_tag, pub Expression);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum AssocKey {
    Label(Label),
    Expression(Expression),
}

impl AssocKey {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            AssocKey::Label(Label(.., linecol)) => Some(linecol.0),
            AssocKey::Expression(expr) => expr.start_line(),
        }
    }
}

def_tag!(label_tag, "@label");
#[derive(Deserialize, Debug, Clone)]
pub struct Label(pub label_tag, pub String, pub LineCol);

def_tag!(symbol_literal_tag, "symbol_literal");
#[derive(Deserialize, Debug, Clone)]
pub struct SymbolLiteral(pub symbol_literal_tag, pub SymbolOrBare, pub StartEnd);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum SymbolOrBare {
    Ident(Ident),
    Op(Op),
    Kw(Kw),
    Symbol(Symbol),
    GVar(GVar),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum IdentOrConst {
    Ident(Ident),
    Const(Const),
}

impl IdentOrConst {
    pub fn into_ident_or_op_or_keyword_or_const(self) -> IdentOrOpOrKeywordOrConst {
        match self {
            IdentOrConst::Ident(i) => IdentOrOpOrKeywordOrConst::Ident(i),
            IdentOrConst::Const(c) => IdentOrOpOrKeywordOrConst::Const(c),
        }
    }

    pub fn start_line(&self) -> u64 {
        match self {
            IdentOrConst::Ident(Ident(.., linecol)) | IdentOrConst::Const(Const(.., linecol)) => {
                linecol.0
            }
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick {
    Ident(Ident),
    Const(Const),
    Keyword(Kw),
    Op(Op),
    IVar(IVar),
    GVar(GVar),
    CVar(CVar),
    Backtick(Backtick),
}

impl IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Ident(Ident(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Const(Const(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Keyword(Kw(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Op(Op(_, _, linecol, _))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::IVar(IVar(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::GVar(GVar(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::CVar(CVar(.., linecol))
            | IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Backtick(Backtick(.., linecol)) => {
                Some(linecol.0)
            }
        }
    }
}

def_tag!(symbol_tag, "symbol");
#[derive(Deserialize, Debug, Clone)]
pub struct Symbol(
    pub symbol_tag,
    pub IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick,
);

impl Symbol {
    pub fn from_string(s: String, l: LineCol) -> Self {
        Symbol(
            symbol_tag,
            IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Ident(Ident::new(s, l)),
        )
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum CallLeft {
    Paren(ParenExpr),
    SingleParen(paren_tag, Box<Expression>),
    Call(Call),
    FCall(FCall),
    VCall(VCall),
    MethodAddArg(MethodAddArg),
    MethodAddBlock(MethodAddBlock),
    VarRef(VarRef),
    Super(Super),
    ZSuper(ZSuper),
    Next(Next),
    Yield(Yield),
    Yield0(Yield0),
    Command(Command),
    CommandCall(CommandCall),
    Expression(Box<Expression>),
}

impl CallLeft {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            CallLeft::ZSuper(ZSuper(.., start_end))
            | CallLeft::Next(Next(.., start_end))
            | CallLeft::Yield(Yield(.., start_end))
            | CallLeft::Yield0(Yield0(.., start_end))
            | CallLeft::Super(Super(.., start_end)) => Some(start_end.start_line()),
            CallLeft::Paren(ParenExpr(_, paren_expr_or_exprs, ..)) => {
                paren_expr_or_exprs.start_line()
            }
            CallLeft::SingleParen(_, expr) => expr.start_line(),
            CallLeft::Command(Command(_, ident_or_const, ..))
            | CallLeft::VCall(VCall(_, ident_or_const, _))
            | CallLeft::FCall(FCall(_, ident_or_const)) => Some(match ident_or_const {
                IdentOrConst::Ident(Ident(.., linecol))
                | IdentOrConst::Const(Const(.., linecol)) => linecol.0,
            }),
            CallLeft::CommandCall(CommandCall(_, call_left, ..))
            | CallLeft::MethodAddArg(MethodAddArg(_, call_left, ..))
            | CallLeft::Call(Call(_, call_left, ..))
            | CallLeft::MethodAddBlock(MethodAddBlock(_, call_left, ..)) => call_left.start_line(),
            CallLeft::VarRef(VarRef(_, var_ref_type)) => Some(var_ref_type.start_line()),
            CallLeft::Expression(expr) => expr.start_line(),
        }
    }
    pub fn into_call_chain(self) -> Vec<CallChainElement> {
        match self {
            CallLeft::Paren(p) => vec![CallChainElement::Paren(p)],
            CallLeft::SingleParen(_, e) => vec![CallChainElement::Paren(ParenExpr(
                paren_expr_tag,
                ParenExpressionOrExpressions::Expressions(vec![*e]),
                StartEnd::unknown(),
            ))],
            CallLeft::FCall(FCall(_, ic)) => vec![CallChainElement::IdentOrOpOrKeywordOrConst(
                ic.into_ident_or_op_or_keyword_or_const(),
            )],
            CallLeft::VCall(VCall(_, ic, _)) => vec![CallChainElement::IdentOrOpOrKeywordOrConst(
                ic.into_ident_or_op_or_keyword_or_const(),
            )],
            CallLeft::Call(Call(_, left, dot, name, _)) => {
                let mut res = left.into_call_chain();
                res.push(CallChainElement::DotTypeOrOp(dot));

                if let CallMethodName::IdentOrOpOrKeywordOrConst(ic) = name {
                    res.push(CallChainElement::IdentOrOpOrKeywordOrConst(ic));
                }
                res
            }
            CallLeft::MethodAddArg(MethodAddArg(_, left, an, start_end)) => {
                let mut res = left.into_call_chain();
                res.push(args_as_cc(an, Some(start_end)));
                res
            }
            CallLeft::MethodAddBlock(MethodAddBlock(_, left, block)) => {
                let mut res = left.into_call_chain();
                res.push(CallChainElement::Block(block));
                res
            }
            CallLeft::VarRef(v) => vec![CallChainElement::VarRef(v)],
            CallLeft::Super(s) => s.into_call_chain(),
            CallLeft::ZSuper(zs) => zs.into_call_chain(),
            CallLeft::Next(next) => next.into_call_chain(),
            CallLeft::Yield(y) => y.into_call_chain(),
            CallLeft::Yield0(y) => y.into_call_chain(),
            CallLeft::Command(c) => c.into_call_chain(),
            CallLeft::CommandCall(c) => c.into_call_chain(),
            CallLeft::Expression(e) => vec![CallChainElement::Expression(e)],
        }
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum CallChainElement {
    IdentOrOpOrKeywordOrConst(IdentOrOpOrKeywordOrConst),
    Block(Block),
    VarRef(VarRef),
    ArgsAddStarOrExpressionListOrArgsForward(
        ArgsAddStarOrExpressionListOrArgsForward,
        Option<StartEnd>,
    ),
    DotTypeOrOp(DotTypeOrOp),
    Paren(ParenExpr),
    Expression(Box<Expression>),
}

impl CallChainElement {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            CallChainElement::IdentOrOpOrKeywordOrConst(ident) => {
                Some(ident.clone().to_def_parts().1 .0)
            }
            CallChainElement::Block(block) => Some(block.start_line()),
            CallChainElement::VarRef(VarRef(.., var_ref_type)) => Some(var_ref_type.start_line()),
            CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..) => {
                // The start_end for these is generally incorrect, since the parser event
                // only fires at the _end_ of the expression list
                None
            }
            CallChainElement::DotTypeOrOp(d) => d.start_line(),
            CallChainElement::Paren(pe) => pe.1.start_line(),
            CallChainElement::Expression(expr) => expr.start_line(),
        }
    }
}

pub type DotCall = call_tag;

def_tag!(method_add_arg_tag, "method_add_arg");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodAddArg(
    pub method_add_arg_tag,
    pub Box<CallLeft>,
    pub ArgNode,
    pub StartEnd,
);

impl ToMethodCall for MethodAddArg {
    fn to_method_call(self) -> MethodCall {
        let mut orig_chain = (self.1).into_call_chain();
        let last = orig_chain
            .pop()
            .expect("cannot be empty with method add arg");
        if let CallChainElement::IdentOrOpOrKeywordOrConst(n) = last {
            MethodCall::new(orig_chain, n, true, normalize_args(self.2), Some(self.3))
        } else {
            MethodCall::new(
                orig_chain,
                IdentOrOpOrKeywordOrConst::Ident(Ident::new(".()".to_string(), LineCol::unknown())),
                true,
                normalize_args(self.2),
                Some(self.3),
            )
        }
    }
}

def_tag!(method_add_block_tag, "method_add_block");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodAddBlock(method_add_block_tag, pub Box<CallLeft>, pub Block);

impl ToMethodCall for MethodAddBlock {
    fn to_method_call(self) -> MethodCall {
        let mut orig_chain = (self.1).into_call_chain();
        let last = orig_chain.pop().expect("cannot be empty");
        let (args, start_end) = match last {
            CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(aas, start_end) => {
                (aas, start_end)
            }
            _ => (
                ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(Vec::new()),
                None,
            ),
        };
        MethodCall::new(
            orig_chain,
            IdentOrOpOrKeywordOrConst::Ident(Ident::new(".()".to_string(), LineCol::unknown())),
            true,
            args,
            start_end,
        )
    }
}

def_tag!(fcall_tag, "fcall");
#[derive(Deserialize, Debug, Clone)]
pub struct FCall(pub fcall_tag, pub IdentOrConst);

def_tag!(vcall);
#[derive(Deserialize, Debug, Clone)]
pub struct VCall(pub vcall, pub IdentOrConst, pub StartEnd);

pub trait ToMethodCall {
    fn to_method_call(self) -> MethodCall;
}

impl ToMethodCall for VCall {
    fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![],
            (self.1).into_ident_or_op_or_keyword_or_const(),
            false,
            ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![]),
            Some(self.2),
        )
    }
}

// isn't parsable, but we do create it in our "normalized tree"
def_tag!(method_call_tag, "method_call");
#[derive(Deserialize, Debug, Clone)]
pub struct MethodCall(
    pub method_call_tag,
    // call chain
    pub Vec<CallChainElement>,
    // method name
    pub IdentOrOpOrKeywordOrConst,
    // original used parens
    pub bool,
    // args
    pub ArgsAddStarOrExpressionListOrArgsForward,
    pub Option<StartEnd>,
);

impl MethodCall {
    pub fn new(
        chain: Vec<CallChainElement>,
        name: IdentOrOpOrKeywordOrConst,
        use_parens: bool,
        args: ArgsAddStarOrExpressionListOrArgsForward,
        start_end: Option<StartEnd>,
    ) -> Self {
        MethodCall(method_call_tag, chain, name, use_parens, args, start_end)
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum CallMethodName {
    IdentOrOpOrKeywordOrConst(IdentOrOpOrKeywordOrConst),
    DotCall(DotCall),
}

def_tag!(call_tag, "call");
#[derive(Deserialize, Debug, Clone)]
pub struct Call(
    pub call_tag,
    pub Box<CallLeft>,
    pub DotTypeOrOp,
    pub CallMethodName,
    pub StartEnd,
);

impl ToMethodCall for Call {
    fn to_method_call(self) -> MethodCall {
        let mut chain = (self.1).into_call_chain();
        let method_name = match self.3 {
            CallMethodName::IdentOrOpOrKeywordOrConst(i) => i,
            CallMethodName::DotCall(_) => {
                IdentOrOpOrKeywordOrConst::Ident(Ident::new("".to_string(), LineCol::unknown()))
            }
        };
        chain.push(CallChainElement::DotTypeOrOp(self.2));
        MethodCall::new(
            chain,
            method_name,
            false,
            ArgsAddStarOrExpressionListOrArgsForward::empty(),
            Some(self.4),
        )
    }
}

def_tag!(command_call_tag, "command_call");
#[derive(Deserialize, Debug, Clone)]
pub struct CommandCall(
    command_call_tag,
    pub Box<CallLeft>,
    pub DotTypeOrOp,
    pub IdentOrOpOrKeywordOrConst,
    pub ArgNode,
);

impl CommandCall {
    pub fn into_call_chain(self) -> Vec<CallChainElement> {
        let mut recur = (self.1).into_call_chain();
        recur.push(CallChainElement::DotTypeOrOp(self.2));
        recur.push(CallChainElement::IdentOrOpOrKeywordOrConst(self.3));
        recur.push(args_as_cc(self.4, None));
        recur
    }
}

impl ToMethodCall for CommandCall {
    fn to_method_call(self) -> MethodCall {
        let mut chain = (self.1).into_call_chain();
        chain.push(CallChainElement::DotTypeOrOp(self.2));

        MethodCall::new(chain, self.3, false, normalize_args(self.4), None)
    }
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum DotType {
    Dot(Dot),
    LonelyOperator(LonelyOperator),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum DotTypeOrOp {
    DotType(DotType),
    Period(Period),
    ColonColon(ColonColon),
    Op(Op),
    StringDot(String),
}

impl DotTypeOrOp {
    pub fn start_line(&self) -> Option<u64> {
        match self {
            DotTypeOrOp::Period(Period(.., linecol)) => Some(linecol.0),
            DotTypeOrOp::DotType(
                DotType::Dot(Dot(.., start_end))
                | DotType::LonelyOperator(LonelyOperator(.., start_end)),
            )
            | DotTypeOrOp::Op(Op(.., start_end)) => Some(start_end.start_line()),
            DotTypeOrOp::ColonColon(..) | DotTypeOrOp::StringDot(..) => None,
        }
    }
}

def_tag!(period_tag, "@period");
#[derive(Deserialize, Debug, Clone)]
pub struct Period(pub period_tag, pub String, pub LineCol);

def_tag!(equals_tag, "==");
#[derive(Deserialize, Debug, Clone)]
pub struct Equals(pub equals_tag);

def_tag!(dot_tag, ".");
#[derive(Deserialize, Debug, Clone)]
pub struct Dot(pub dot_tag, pub StartEnd);

def_tag!(colon_colon_tag, "::");
#[derive(Deserialize, Debug, Clone)]
pub struct ColonColon(pub colon_colon_tag);

def_tag!(lonely_operator_tag, "&.");
#[derive(Deserialize, Debug, Clone)]
pub struct LonelyOperator(pub lonely_operator_tag, pub StartEnd);

def_tag!(op_tag, "@op");
#[derive(Deserialize, Debug, Clone)]
pub struct Op(pub op_tag, pub Operator, pub LineCol, pub StartEnd);

#[derive(RipperDeserialize, Debug, Clone)]
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
pub struct Next(pub next_tag, pub ArgsAddBlockOrExpressionList, pub StartEnd);

impl Next {
    pub fn into_call_chain(self) -> Vec<CallChainElement> {
        vec![
            ident_as_cc("next".to_string(), &self.2),
            CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(
                normalize_args_add_block_or_expression_list(self.1),
                Some(self.2),
            ),
        ]
    }
}

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
    BitwiseNot,
}

impl<'de> Deserialize<'de> for UnaryType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Deserialize::deserialize(deserializer)? {
            "not" => Ok(Self::Not),
            "-@" => Ok(Self::Negative),
            "+@" => Ok(Self::Positive),
            "!" => Ok(Self::BooleanNot),
            "~" => Ok(Self::BitwiseNot),
            s => Err(de::Error::invalid_value(
                de::Unexpected::Str(s),
                &"not, -@, +@, or !",
            )),
        }
    }
}

def_tag!(unary_tag, "unary");
#[derive(Deserialize, Debug, Clone)]
pub struct Unary(pub unary_tag, pub UnaryType, pub Box<Expression>);

def_tag!(super_tag, "super");
#[derive(Deserialize, Debug, Clone)]
pub struct Super(pub super_tag, pub ArgNode, pub StartEnd);

impl Super {
    pub fn into_call_chain(self) -> Vec<CallChainElement> {
        vec![
            ident_as_cc("super".to_string(), &self.2),
            args_as_cc(self.1, Some(self.2)),
        ]
    }
}

impl ToMethodCall for Super {
    fn to_method_call(self) -> MethodCall {
        MethodCall::new(
            vec![],
            IdentOrOpOrKeywordOrConst::Ident(Ident::new(
                "super".to_string(),
                LineCol::from_line((self.2).0),
            )),
            true,
            normalize_args(self.1),
            Some(self.2),
        )
    }
}

def_tag!(kw_tag, "@kw");
#[derive(Deserialize, Debug, Clone)]
pub struct Kw(pub kw_tag, pub String, pub LineCol);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ConstPathRefOrConstRefOrTopConstRef {
    ConstPathRef(ConstPathRef),
    ConstRef(ConstRef),
    TopConstRef(TopConstRef),
}

def_tag!(class_tag, "class");
#[derive(Deserialize, Debug, Clone)]
pub struct Class(
    pub class_tag,
    pub ConstPathRefOrConstRefOrTopConstRef,
    pub Option<Box<Expression>>,
    pub Box<BodyStmt>,
    pub StartEnd,
);

def_tag!(module_tag, "module");
#[derive(Deserialize, Debug, Clone)]
pub struct Module(
    pub module_tag,
    pub ConstPathRefOrConstRefOrTopConstRef,
    pub Box<BodyStmt>,
    pub StartEnd,
);

def_tag!(defs_tag, "defs");
#[derive(Deserialize, Debug, Clone)]
pub struct Defs(
    pub defs_tag,
    pub Singleton,
    pub DotOrColon,
    pub IdentOrOpOrKeywordOrConst,
    pub ParenOrParams,
    pub Box<DefBodyStmt>,
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum IdentOrKw {
    Ident(Ident),
    Kw(Kw),
}

#[derive(RipperDeserialize, Debug, Clone)]
pub enum Singleton {
    VarRef(VarRef),
    Paren(ParenExpr),
    VCall(VCall),
}

// can only occur in defs, Op is always `::`
#[derive(RipperDeserialize, Debug, Clone)]
pub enum DotOrColon {
    Period(Period),
    Op(Operator),
}

#[derive(Deserialize, Debug, Clone)]
pub struct BinaryOperator(pub String, pub StartEnd);

def_tag!(binary_tag, "binary");
#[derive(Deserialize, Debug, Clone)]
pub struct Binary(
    pub binary_tag,
    pub Box<Expression>,
    pub BinaryOperator,
    pub Box<Expression>,
);

def_tag!(float_tag, "@float");
#[derive(Deserialize, Debug, Clone)]
pub struct Float(float_tag, pub String, pub LineCol);

def_tag!(aref_tag, "aref");
#[derive(Deserialize, Debug, Clone)]
pub struct Aref(
    aref_tag,
    pub Box<Expression>,
    pub Option<ArgNode>,
    pub LineCol,
);

def_tag!(char_tag, "@CHAR");
#[derive(Deserialize, Debug, Clone)]
pub struct Char(char_tag, pub String, pub LineCol);

def_tag!(return_tag, "return");
#[derive(Deserialize, Debug, Clone)]
pub struct Return(return_tag, pub ArgNode, pub StartEnd);

def_tag!(return0_tag, "return0");
#[derive(Deserialize, Debug, Clone)]
pub struct Return0(return0_tag, pub StartEnd);

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
pub struct Yield(yield_tag, pub ParenOrArgsAddBlock, pub StartEnd);

impl Yield {
    fn into_call_chain(self) -> Vec<CallChainElement> {
        let arg = (self.1).into_arg_node();
        vec![
            ident_as_cc("yield".to_string(), &self.2),
            args_as_cc(arg, Some(self.2)),
        ]
    }
}

impl ToMethodCall for Yield {
    fn to_method_call(self) -> MethodCall {
        let use_parens = match &self.1 {
            ParenOrArgsAddBlock::ArgsAddBlock(ArgsAddBlock(
                _,
                ArgsAddBlockInner::Parens(aabparen),
                ..,
            )) => {
                aabparen.len() == 1
                    && match &aabparen[0] {
                        AABParen::Expression(expr) => {
                            matches!(**expr, Expression::BareAssocHash(_))
                        }
                        _ => false,
                    }
            }
            x => x.is_paren(),
        };

        // This will be used to determine whether or not to wind to the end of the expression,
        // but for calls without parens, this will wind _too far_, so we return `None` in those
        // cases to avoid over-winding
        let start_end = if use_parens {
            Some(self.2.clone())
        } else {
            None
        };

        MethodCall::new(
            vec![],
            IdentOrOpOrKeywordOrConst::Ident(Ident::new(
                "yield".to_string(),
                LineCol::from_line((self.2).0),
            )),
            use_parens,
            normalize_args((self.1).into_arg_node()),
            start_end,
        )
    }
}

def_tag!(break_tag, "break");
#[derive(Deserialize, Debug, Clone)]
pub struct Break(break_tag, pub ParenOrArgsAddBlock, pub StartEnd);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum ParenOrArgsAddBlock {
    YieldParen(YieldParen),
    ArgsAddBlock(ArgsAddBlock),
    Empty(Vec<()>),
}

impl ParenOrArgsAddBlock {
    fn is_paren(&self) -> bool {
        matches!(self, ParenOrArgsAddBlock::YieldParen(_))
    }

    fn into_arg_node(self) -> ArgNode {
        match self {
            ParenOrArgsAddBlock::YieldParen(yp) => *yp.1,
            ParenOrArgsAddBlock::ArgsAddBlock(aab) => ArgNode::ArgsAddBlock(aab),
            ParenOrArgsAddBlock::Empty(_) => ArgNode::Null(None),
        }
    }
}

def_tag!(yield_paren_tag, "paren");
#[derive(Deserialize, Debug, Clone)]
pub struct YieldParen(yield_paren_tag, pub Box<ArgNode>);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum Block {
    BraceBlock(BraceBlock),
    DoBlock(DoBlock),
}

impl Block {
    pub fn start_line(&self) -> u64 {
        match self {
            Block::BraceBlock(BraceBlock(.., start_end))
            | Block::DoBlock(DoBlock(.., start_end)) => start_end.start_line(),
        }
    }
}

// block local variables are a nightmare, they can be false, nil, or an array
// of idents:
//
// 1. nil if params are not present, and block local variables are also not
//    specified
// 2. false if params are present, and block local variables are not present
// 3. a vec of idents either if params are or are not present, and block local
//    variables are present
#[derive(RipperDeserialize, Debug, Clone)]
pub enum BlockLocalVariables {
    EmptyBecauseParamsWerePresent(bool),
    NilBecauseParamsWereNotPresent(Option<()>),
    Present(Vec<Ident>),
}

def_tag!(block_var_tag, "block_var");
#[derive(Deserialize, Debug, Clone)]
pub struct BlockVar(
    block_var_tag,
    pub Option<Box<Params>>,
    pub BlockLocalVariables,
    pub StartEnd,
);

def_tag!(do_block_tag, "do_block");
#[derive(Deserialize, Debug, Clone)]
pub struct DoBlock(
    do_block_tag,
    pub Option<BlockVar>,
    pub Box<BodyStmt>,
    pub StartEnd,
);

def_tag!(brace_block_tag, "brace_block");
#[derive(Deserialize, Debug, Clone)]
pub struct BraceBlock(
    brace_block_tag,
    pub Option<BlockVar>,
    pub Vec<Expression>,
    pub StartEnd,
);

def_tag!(while_tag, "while");
#[derive(Deserialize, Debug, Clone)]
pub struct While(
    while_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub StartEnd,
);

def_tag!(until_tag, "until");
#[derive(Deserialize, Debug, Clone)]
pub struct Until(
    until_tag,
    pub Box<Expression>,
    pub Vec<Expression>,
    pub StartEnd,
);

def_tag!(while_mod_tag, "while_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct WhileMod(while_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(until_mod_tag, "until_mod");
#[derive(Deserialize, Debug, Clone)]
pub struct UntilMod(until_mod_tag, pub Box<Expression>, pub Box<Expression>);

def_tag!(case_tag, "case");
#[derive(Deserialize, Debug, Clone)]
pub struct Case(
    case_tag,
    pub Option<Box<Expression>>,
    pub When,
    pub StartEnd,
);

def_tag!(when_tag, "when");
#[derive(Deserialize, Debug, Clone)]
pub struct When(
    when_tag,
    pub ArgsAddStarOrExpressionListOrArgsForward,
    pub Vec<Expression>,
    pub Option<Box<WhenOrElse>>,
    pub StartEnd,
);

#[derive(RipperDeserialize, Debug, Clone)]
pub enum WhenOrElse {
    When(When),
    Else(CaseElse),
}

def_tag!(case_else_tag, "else");
#[derive(Deserialize, Debug, Clone)]
pub struct CaseElse(case_else_tag, pub Vec<Expression>, pub StartEnd);

def_tag!(retry_tag, "retry");
#[derive(Deserialize, Debug, Clone)]
pub struct Retry(pub retry_tag, pub StartEnd);

def_tag!(redo_tag, "redo");
#[derive(Deserialize, Debug, Clone)]
pub struct Redo(pub redo_tag, pub StartEnd);

def_tag!(sclass_tag, "sclass");
#[derive(Deserialize, Debug, Clone)]
pub struct SClass(
    sclass_tag,
    pub Box<Expression>,
    pub Box<BodyStmt>,
    pub StartEnd,
);

// some constructs were expressionlist in 2.5 and bodystmt in 2.6 so this
// deals with both cases
#[derive(RipperDeserialize, Debug, Clone)]
pub enum ExpressionListOrBodyStmt {
    ExpressionList(Vec<Expression>),
    BodyStmt(Box<BodyStmt>),
}

def_tag!(stabby_lambda_tag, "lambda");
#[derive(Deserialize, Debug, Clone)]
pub struct StabbyLambda(
    stabby_lambda_tag,
    pub ParenOrParams,
    pub ExpressionListOrBodyStmt,
    pub StartEnd,
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

#[derive(RipperDeserialize, Debug, Clone)]
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

def_tag!(backtick_tag, "@backtick");
#[derive(Deserialize, Debug, Clone)]
pub struct Backtick(backtick_tag, pub String, pub LineCol);
