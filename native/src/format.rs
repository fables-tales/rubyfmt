use crate::parser_state::{FormattingContext, ParserState};
use crate::ripper_tree_types::*;
use std::borrow::Borrow;

pub fn format_def(ps: &mut ParserState, def: Def) {
    let def_expression = def.1;
    let params = match def.2 {
        ParenOrParams::Paren(p) => p.1,
        ParenOrParams::Params(p) => p,
    };

    let body = def.3;
    ps.on_line((def_expression.2).0);
    ps.emit_indent();
    ps.emit_def(def_expression.1);
    format_params(ps, params, "(".to_string(), ")".to_string());
    ps.emit_newline();

    ps.with_formatting_context(FormattingContext::Def, |ps| {
        ps.new_block(|ps| {
            format_bodystmt(ps, body, false);
        });
    });

    ps.emit_end();
    ps.emit_newline();
}

pub fn format_params(
    ps: &mut ParserState,
    params: Params,
    open_delim: String,
    close_delim: String,
) {
    let non_null_positions = vec![
        (params.1).is_some(),
        (params.2).is_some(),
        (params.3).is_some(),
        (params.4).is_some(),
        (params.5).is_some(),
        (params.6).is_some(),
        (params.7).is_some(),
    ];

    let have_any_params = non_null_positions.iter().any(|&x| x);
    if !have_any_params {
        return;
    }

    ps.breakable_of(open_delim, close_delim, |ps| {
        ps.breakable_entry(|ps| {
            //def foo(a, b=nil, *args, d, e:, **kwargs, &blk)
            //        ^  ^___^  ^___^  ^  ^    ^_____^   ^
            //        |    |      |    |  |      |       |
            //        |    |      |    |  |      |    block_arg
            //        |    |      |    |  |      |
            //        |    |      |    |  |  kwrest_params
            //        |    |      |    |  |
            //        |    |      |    | kwargs
            //        |    |      |    |
            //        |    |      | more_required_params
            //        |    |      |
            //        |    |  rest_params
            //        |    |
            //        | optional params
            //        |
            //    required params
            let required_params = (params.1).unwrap_or(Vec::new());
            let optional_params = (params.2).unwrap_or(Vec::new());
            let rest_param = params.3;
            let more_required_params = (params.4).unwrap_or(Vec::new());
            let kwargs = (params.5).unwrap_or(Vec::new());
            let kwrest_params = params.6;
            let block_arg = params.7;

            let formats: Vec<Box<dyn FnOnce(&mut ParserState) -> bool>> = vec![
                Box::new(move |ps: &mut ParserState| format_required_params(ps, required_params)),
                Box::new(move |ps: &mut ParserState| format_optional_params(ps, optional_params)),
                Box::new(move |ps: &mut ParserState| format_rest_param(ps, rest_param)),
                Box::new(move |ps: &mut ParserState| {
                    format_required_params(ps, more_required_params)
                }),
                Box::new(move |ps: &mut ParserState| format_kwargs(ps, kwargs)),
                Box::new(move |ps: &mut ParserState| format_kwrest_params(ps, kwrest_params)),
                Box::new(move |ps: &mut ParserState| format_block_arg(ps, block_arg)),
            ];

            for (idx, format_fn) in formats.into_iter().enumerate() {
                let did_emit = format_fn(ps);
                let have_more = non_null_positions[idx + 1..].into_iter().any(|&v| v);

                if did_emit && have_more {
                    ps.emit_comma();
                    ps.emit_soft_newline();
                }
            }
            ps.emit_collapsing_newline();
        });
    });
}

pub fn format_kwrest_params(ps: &mut ParserState, kwrest_params: Option<KwRestParam>) -> bool {
    if kwrest_params.is_none() {
        return false;
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_soft_indent();
        ps.emit_ident("**".to_string());
        let ident = (kwrest_params.unwrap()).1;
        if ident.is_some() {
            format_ident(ps, ident.unwrap());
        }
    });
    return true;
}

pub fn format_block_arg(ps: &mut ParserState, block_arg: Option<BlockArg>) -> bool {
    if block_arg.is_none() {
        return false;
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_soft_indent();
        ps.emit_ident("&".to_string());
        format_ident(ps, block_arg.unwrap().1);
    });

    return true;
}

pub fn format_kwargs(ps: &mut ParserState, kwargs: Vec<(Label, ExpressionOrFalse)>) -> bool {
    if kwargs.is_empty() {
        return false;
    }

    ps.with_start_of_line(false, |ps| {
        let len = kwargs.len();
        for (idx, (label, expr_or_false)) in kwargs.into_iter().enumerate() {
            ps.emit_soft_indent();
            ps.emit_ident(label.1);
            match expr_or_false {
                ExpressionOrFalse::Expression(e) => {
                    ps.emit_space();
                    format_expression(ps, e);
                }
                ExpressionOrFalse::False(_) => {}
            }
            emit_params_separator(ps, idx, len);
        }
    });

    return true;
}

pub fn format_rest_param(ps: &mut ParserState, rest_param: Option<RestParam>) -> bool {
    if rest_param.is_none() {
        return false;
    }

    ps.emit_soft_indent();
    ps.emit_ident("*".to_string());
    ps.with_start_of_line(false, |ps| {
        let ident = (rest_param.unwrap()).1;
        if ident.is_some() {
            format_ident(ps, ident.unwrap());
        }
    });

    return true;
}

pub fn format_optional_params(
    ps: &mut ParserState,
    optional_params: Vec<(Ident, Expression)>,
) -> bool {
    if optional_params.is_empty() {
        return false;
    }

    ps.with_start_of_line(false, |ps| {
        let len = optional_params.len();
        for (idx, (left, right)) in optional_params.into_iter().enumerate() {
            ps.emit_soft_indent();
            format_ident(ps, left);
            ps.emit_ident(" = ".to_string());
            format_expression(ps, right);
            emit_params_separator(ps, idx, len);
        }
    });

    return true;
}

pub fn format_required_params(ps: &mut ParserState, required_params: Vec<Ident>) -> bool {
    if required_params.is_empty() {
        return false;
    }

    ps.with_start_of_line(false, |ps| {
        let len = required_params.len();
        for (idx, ident) in required_params.into_iter().enumerate() {
            ps.emit_soft_indent();
            format_ident(ps, ident);
            emit_params_separator(ps, idx, len);
        }
    });
    return true;
}

pub fn emit_params_separator(ps: &mut ParserState, index: usize, length: usize) {
    if index != length - 1 {
        ps.emit_comma();
        ps.emit_soft_newline();
    }
}

pub fn format_bodystmt(ps: &mut ParserState, bodystmt: BodyStmt, inside_begin: bool) {
    let expressions = bodystmt.1;
    let rescue_part = bodystmt.2;
    let else_part = bodystmt.3;
    let ensure_part = bodystmt.4;

    for expression in expressions {
        format_expression(ps, expression);
    }

    format_rescue(ps, rescue_part);
    format_else(ps, else_part);
    format_ensure(ps, ensure_part);
}

pub fn format_rescue(ps: &mut ParserState, rescue_part: Option<Box<Expression>>) {
    if rescue_part.is_none() {
        return;
    }
    unimplemented!();
}

pub fn format_else(ps: &mut ParserState, else_part: Option<Box<Expression>>) {
    if else_part.is_none() {
        return;
    }
    unimplemented!();
}

pub fn format_ensure(ps: &mut ParserState, ensure_part: Option<Box<Expression>>) {
    if ensure_part.is_none() {
        return;
    }
    unimplemented!();
}

pub fn use_parens_for_method_call(
    method: &Box<Expression>,
    chain: &Vec<CallChainElement>,
    args: &Vec<Expression>,
    original_used_parens: bool,
    context: &FormattingContext,
) -> bool {
    match method.borrow() {
        Expression::DotCall(_) => return true,
        Expression::Ident(Ident(_, name, _)) => {
            if name.starts_with("attr_") && context == &FormattingContext::ClassOrModule {
                return false;
            }

            if name == "return" || name == "raise" {
                return false;
            }

            if name == "super" || name == "yield" || name == "require" {
                return original_used_parens;
            }

            if name == "new" {
                return true;
            }

            if args.is_empty() {
                return false;
            }

            if context == &FormattingContext::ClassOrModule && !original_used_parens {
                return false;
            }

            return true;
        }
        _ => panic!(
            "method should always be ident or dotcall, got: {:?}",
            method
        ),
    };
}

pub fn format_dot_type(ps: &mut ParserState, dt: DotType) {
    match dt {
        DotType::Dot(_) => ps.emit_dot(),
        DotType::LonelyOperator(_) => ps.emit_lonely_operator(),
    }
}

pub fn format_dot(ps: &mut ParserState, dot: DotTypeOrOp) {
    match dot {
        DotTypeOrOp::DotType(dt) => format_dot_type(ps, dt),
        DotTypeOrOp::Op(op) => match op.1 {
            Operator::Dot(dot) => format_dot_type(ps, DotType::Dot(dot)),
            Operator::LonelyOperator(dot) => format_dot_type(ps, DotType::LonelyOperator(dot)),
            _ => panic!("should be impossible, dot position operator parsed as not a dot"),
        },
        DotTypeOrOp::Period(_) => {
            ps.emit_dot();
        }
    }
}

pub fn format_method_call(ps: &mut ParserState, method_call: MethodCall) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let (chain, method, original_used_parens, args) =
        (method_call.1, method_call.2, method_call.3, method_call.4);

    let use_parens = use_parens_for_method_call(
        &method,
        &chain,
        &args,
        original_used_parens,
        &ps.current_formatting_context(),
    );

    ps.with_start_of_line(false, |ps| {
        for expr in chain {
            match expr {
                CallChainElement::Expression(e) => format_expression(ps, *e),
                CallChainElement::Dot(dot) => format_dot(ps, dot),
            };
        }

        match *method {
            Expression::Ident(i) => format_ident(ps, i),
            _ => unimplemented!(),
        };
        if use_parens {
            ps.emit_ident("(".to_string());
        } else if !args.is_empty() {
            ps.emit_ident(" ".to_string());
        }

        ps.with_formatting_context(FormattingContext::ArgsList, |ps| {
            format_list_like_thing_items(ps, args, true);
        });

        if use_parens {
            ps.emit_ident(")".to_string());
        }
    });
}

pub fn format_list_like_thing_items(
    ps: &mut ParserState,
    args: Vec<Expression>,
    single_line: bool,
) -> bool {
    let mut emitted_args = false;
    let args_count = args.len();

    for (idx, expr) in args.into_iter().enumerate() {
        // this raise was present in the ruby source code of rubyfmt
        // but I'm pretty sure it's categorically impossible now. Thanks
        // type system
        //raise "this is bad" if expr[0] == :tstring_content

        if single_line {
            format_expression(ps, expr);
            if !(idx == args_count - 1) {
                ps.emit_comma_space();
            }
        } else {
            ps.emit_soft_indent();
            ps.with_start_of_line(false, |ps| {
                format_expression(ps, expr);
                ps.emit_comma();
                ps.emit_soft_newline();
            });
        };
        emitted_args = true;
    }

    emitted_args
}

pub fn format_ident(ps: &mut ParserState, ident: Ident) {
    ps.on_line(ident.line_number());
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident(ident.1);
}

pub fn format_const(ps: &mut ParserState, c: Const) {
    ps.on_line(c.line_number());
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident(c.1);
}

pub fn format_int(ps: &mut ParserState, int: Int) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_int(int.1);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_bare_assoc_hash(ps: &mut ParserState, bah: BareAssocHash) {
    format_assocs(ps, bah.1)
}

pub fn format_alias(ps: &mut ParserState, alias: Alias) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident("alias ".to_string());

    ps.with_start_of_line(false, |ps| {
        format_symbol_literal(ps, alias.1);
        ps.emit_space();
        format_symbol_literal(ps, alias.2);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_op(ps: &mut ParserState, op: Op) {
    match op.1 {
        Operator::Equals(_) => ps.emit_ident("==".to_string()),
        Operator::Dot(_) => ps.emit_dot(),
        Operator::LonelyOperator(_) => ps.emit_lonely_operator(),
    }
}

pub fn format_symbol(ps: &mut ParserState, symbol: Symbol) {
    ps.emit_ident(":".to_string());
    match symbol.1 {
        IdentOrConst::Ident(i) => format_ident(ps, i),
        IdentOrConst::Const(c) => format_const(ps, c),
    }
}

pub fn format_symbol_literal(ps: &mut ParserState, symbol_literal: SymbolLiteral) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| match symbol_literal.1 {
        SymbolOrBare::Ident(ident) => format_ident(ps, ident),
        SymbolOrBare::Op(op) => format_op(ps, op),
        SymbolOrBare::Symbol(symbol) => format_symbol(ps, symbol),
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_assocs(ps: &mut ParserState, assocs: Vec<AssocNewOrAssocSplat>) {
    for assoc in assocs.into_iter() {
        ps.emit_soft_indent();
        ps.with_start_of_line(false, |ps| match assoc {
            AssocNewOrAssocSplat::AssocNew(new) => {
                match new.1 {
                    LabelOrSymbolLiteralOrDynaSymbol::Label(label) => {
                        ps.emit_ident(label.1);
                        ps.emit_space();
                    }
                    LabelOrSymbolLiteralOrDynaSymbol::SymbolLiteral(symbol) => {
                        match symbol.1 {
                            SymbolOrBare::Symbol(symbol) => {
                                format_expression(ps, Expression::Symbol(symbol))
                            }
                            _ => panic!("other symbol variants are not valid in an assoc"),
                        }
                        ps.emit_space();
                        ps.emit_ident("=>".to_string());
                        ps.emit_space();
                    }
                    LabelOrSymbolLiteralOrDynaSymbol::DynaSymbol(dyna_symbol) => {
                        format_expression(ps, Expression::DynaSymbol(dyna_symbol));
                        ps.emit_space();
                        ps.emit_ident("=>".to_string());
                        ps.emit_space();
                    }
                }
                format_expression(ps, new.2);
            }
            AssocNewOrAssocSplat::AssocSplat(splat) => {
                ps.emit_ident("**".to_string());
                ps.emit_ident((splat.1).1);
            }
        });
        ps.emit_comma();
        ps.emit_soft_newline();
    }
}

pub fn format_begin(ps: &mut ParserState, begin: Begin) {
    if ps.at_start_of_line() {
        ps.emit_indent()
    }

    ps.emit_begin();
    ps.emit_newline();
    ps.new_block(|ps| format_bodystmt(ps, begin.1, true));

    ps.with_start_of_line(true, |ps| {
        ps.emit_end();
        ps.emit_newline();
    });
}

trait ToMethodCall {
    fn to_method_call(self) -> MethodCall;
}

impl ToMethodCall for VCall {
    fn to_method_call(self) -> MethodCall {
        MethodCall::new(vec![], self.1, false, vec![])
    }
}

pub fn normalize(e: Expression) -> Expression {
    match e {
        Expression::VCall(v) => Expression::MethodCall(v.to_method_call()),
        Expression::MethodAddArg(maa) => Expression::MethodCall(maa.to_method_call()),
        //"command" => unimplemented!(),
        //"command_call" => unimplemented!(),
        Expression::Call(call) => Expression::MethodCall(call.to_method_call()),
        //"fcall" => unimplemented!(),
        //"zsuper" => unimplemented!(),
        //"super" => unimplemented!(),
        //"return" => unimplemented!(),
        //"yield" => unimplemented!(),
        e => e,
    }
}

pub fn format_void_stmt(_ps: &mut ParserState, _void: VoidStmt) {
    // deliberately does nothing
}

pub fn format_paren(ps: &mut ParserState, paren: ParenExpr) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    ps.emit_ident("(".to_string());

    if paren.1.len() == 1 {
        let p = (paren.1)
            .into_iter()
            .next()
            .expect("we know this isn't empty");
        ps.with_start_of_line(false, |ps| format_expression(ps, p));
    } else {
        ps.emit_newline();
        ps.new_block(|ps| {
            for expr in (paren.1).into_iter() {
                format_expression(ps, expr);
            }
        });
    }
    ps.emit_ident(")".to_string());
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_dot2(ps: &mut ParserState, dot2: Dot2) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        match dot2.1 {
            Some(expr) => format_expression(ps, *expr),
            _ => {}
        }

        ps.emit_ident("..".to_string());

        match dot2.2 {
            Some(expr) => format_expression(ps, *expr),
            _ => {}
        }
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_array(ps: &mut ParserState, array: Array) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    match array.1 {
        SimpleArrayOrPercentArray::SimpleArray(a) => format_array_fast_path(ps, a),
        _ => {
            unimplemented!();
        }
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_array_fast_path(ps: &mut ParserState, a: Option<ArgsAddStarOrExpressionList>) {
    match a {
        None => {
            ps.emit_open_square_bracket();
            ps.emit_close_square_bracket();
        }
        Some(a) => {
            ps.breakable_of("[".to_string(), "]".to_string(), |ps| {
                ps.breakable_entry(|ps| {
                    format_list_like_thing(ps, a, false);
                });
            });
        }
    }
}

pub fn format_list_like_thing(
    ps: &mut ParserState,
    a: ArgsAddStarOrExpressionList,
    single_line: bool,
) -> bool {
    match a {
        ArgsAddStarOrExpressionList::ArgsAddStar(aas) => {
            let left = aas.1;
            let star = aas.2;
            let right = aas.3;
            let mut emitted_args = format_list_like_thing(ps, *left, single_line);
            if single_line {
                // if we're single line, our predecessor didn't emit a trailing comma
                // space because rubyfmt terminates single line arg lists without the
                // trailer so emit one here
                if emitted_args {
                    ps.emit_comma_space();
                }
            } else {
                // similarly if we're multi line, we emit a newline but not an indent
                // at the end our formatting spree, because we might be at a terminator
                // so fix up the indent
                ps.emit_soft_indent();
            }

            emitted_args = true;

            ps.with_start_of_line(false, |ps| {
                ps.emit_ident("*".to_string());
                format_expression(ps, *star);

                for expr in right {
                    emit_intermediate_array_separator(ps);
                    format_expression(ps, expr);
                }

                ps.emit_comma();
                ps.emit_soft_newline();
            });

            emitted_args
        }
        ArgsAddStarOrExpressionList::ExpressionList(el) => {
            format_list_like_thing_items(ps, el, single_line)
        }
    }
}

pub fn emit_intermediate_array_separator(ps: &mut ParserState) {
    ps.emit_comma();
    ps.emit_soft_newline();
    ps.emit_soft_indent();
}

#[derive(PartialEq, Debug)]
pub enum StringType {
    Quoted,
    Heredoc,
}

pub fn format_inner_string(ps: &mut ParserState, parts: Vec<StringContentPart>, tipe: StringType) {
    if tipe == StringType::Heredoc {
        panic!("heredocs aren't supported yet");
    }
    for (idx, part) in parts.into_iter().enumerate() {
        match part {
            StringContentPart::TStringContent(t) => {
                ps.emit_string_content(t.1)
            },
            StringContentPart::StringEmbexpr(e) => {
                ps.emit_string_content("#{".to_string());
                ps.with_start_of_line(false, |ps| {
                    let expr = ((e.1).into_iter()).next().expect("should not be empty");
                    format_expression(ps, expr);
                });
                ps.emit_string_content("}".to_string());
            },
            StringContentPart::StringDVar(dv) => {
                ps.emit_string_content("#{".to_string());
                ps.with_start_of_line(false, |ps| {
                    let expr = *(dv.1);
                    format_expression(ps, expr);
                });
                ps.emit_string_content("}".to_string());
            }
        }
    }
}

pub fn format_string_literal(ps: &mut ParserState, sl: StringLiteral) {
    let parts = (sl.1).1;

    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_double_quote();
    format_inner_string(ps, parts, StringType::Quoted);
    ps.emit_double_quote();

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_assign(ps: &mut ParserState, assign: Assign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        let left = (assign.1).1;
        let right = assign.2;

        format_var_ref_type(ps, left);

        ps.emit_space();
        ps.emit_op("=".to_string());
        ps.emit_space();

        ps.with_formatting_context(FormattingContext::Assign, |ps| {
            format_expression(ps, *right);
        });
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_ref_type(ps: &mut ParserState, vr: VarRefType) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    match vr {
        VarRefType::CVar(c) => ps.emit_ident(c.1),
        VarRefType::GVar(g) => ps.emit_ident(g.1),
        VarRefType::IVar(i) => ps.emit_ident(i.1),
        VarRefType::Ident(i) => ps.emit_ident(i.1),
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_ref(ps: &mut ParserState, vr: VarRef) {
    format_var_ref_type(ps, vr.1);
}

pub fn format_expression(ps: &mut ParserState, expression: Expression) {
    let expression = normalize(expression);
    match expression {
        Expression::Def(def) => format_def(ps, def),
        Expression::MethodCall(mc) => format_method_call(ps, mc),
        Expression::Ident(ident) => format_ident(ps, ident),
        Expression::Int(int) => format_int(ps, int),
        Expression::BareAssocHash(bah) => format_bare_assoc_hash(ps, bah),
        Expression::Begin(begin) => format_begin(ps, begin),
        Expression::VoidStmt(void) => format_void_stmt(ps, void),
        Expression::Paren(paren) => format_paren(ps, paren),
        Expression::Dot2(dot2) => format_dot2(ps, dot2),
        Expression::SymbolLiteral(sl) => format_symbol_literal(ps, sl),
        Expression::Alias(alias) => format_alias(ps, alias),
        Expression::Array(array) => format_array(ps, array),
        Expression::StringLiteral(sl) => format_string_literal(ps, sl),
        Expression::Assign(assign) => format_assign(ps, assign),
        Expression::VarRef(vr) => format_var_ref(ps, vr),
        e => {
            panic!("got unknown token: {:?}", e);
        }
    }
}

pub fn format_program(ps: &mut ParserState, program: Program) {
    for expression in program.1 {
        format_expression(ps, expression);
    }
}
