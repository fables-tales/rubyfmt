use crate::parser_state::{FormattingContext, ParserState};
use crate::ripper_tree_types::*;
use std::borrow::Borrow;

pub fn format_def(ps: &mut ParserState, def: Def) {
    let def_expression = def.1;

    let body = def.3;
    ps.on_line((def_expression.2).0);
    ps.emit_indent();
    ps.emit_def(def_expression.1);
    format_paren_or_params(ps, def.2);
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
        match rest_param {
            Some(rp) => {
                match rp.1 {
                    Some(IdentOrVarField::Ident(i)) => {
                        format_ident(ps, i);
                    }
                    Some(IdentOrVarField::VarField(vf)) => {
                        format_var_field(ps, vf);
                    }
                    None => {
                        // deliberately do nothing
                    }
                }
            }
            None => {
                panic!("format_rest_param was called with none");
            }
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
    args: &ArgsAddStarOrExpressionList,
    original_used_parens: bool,
    context: &FormattingContext,
) -> bool {
    let name = match method.borrow() {
        Expression::DotCall(_) => return true,
        Expression::Ident(Ident(_, name, _)) => name,
        Expression::Const(Const(_, name, _)) => name,
        _ => panic!(
            "method should always be ident or dotcall, got: {:?}",
            method
        ),
    };
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
        DotTypeOrOp::ColonColon(_) => {
            ps.emit_colon_colon();
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
            Expression::Const(c) => format_const(ps, c),
            x => panic!("got unexpecxted struct {:?}", x),
        };
        if use_parens {
            ps.emit_open_paren();
        } else if !args.is_empty() {
            ps.emit_ident(" ".to_string());
        }

        ps.with_formatting_context(FormattingContext::ArgsList, |ps| {
            format_list_like_thing(ps, args, true);
        });

        if use_parens {
            ps.emit_close_paren();
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
                format_expression(ps, splat.1);
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
        MethodCall::new(
            vec![],
            self.1,
            false,
            ArgsAddStarOrExpressionList::ExpressionList(vec![]),
        )
    }
}

pub fn normalize(e: Expression) -> Expression {
    match e {
        Expression::VCall(v) => Expression::MethodCall(v.to_method_call()),
        Expression::MethodAddArg(maa) => Expression::MethodCall(maa.to_method_call()),
        Expression::Command(command) => Expression::MethodCall(command.to_method_call()),
        Expression::CommandCall(call) => Expression::MethodCall(call.to_method_call()),
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
    ps.emit_open_paren();

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

    ps.emit_close_paren();
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_dot2(ps: &mut ParserState, dot2: Dot2) {
    format_dot2_or_3(ps, "..".to_string(), dot2.1, dot2.2);
}

pub fn format_dot3(ps: &mut ParserState, dot3: Dot3) {
    format_dot2_or_3(ps, "...".to_string(), dot3.1, dot3.2);
}

pub fn format_dot2_or_3(
    ps: &mut ParserState,
    dots: String,
    left: Option<Box<Expression>>,
    right: Option<Box<Expression>>,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        match left {
            Some(expr) => format_expression(ps, *expr),
            _ => {}
        }

        ps.emit_ident(dots);

        match right {
            Some(expr) => format_expression(ps, *expr),
            _ => {}
        }
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn percent_symbol_for(tag: String) -> String {
    match tag.as_ref() {
        "qsymbols" => "%i".to_string(),
        "qwords" => "%w".to_string(),
        "symbols" => "%I".to_string(),
        "words" => "%W".to_string(),
        _ => panic!("got invalid percent symbol"),
    }
}

pub fn format_percent_array(ps: &mut ParserState, tag: String, parts: Vec<StringContentPart>) {
    ps.emit_ident(percent_symbol_for(tag));
    ps.emit_open_square_bracket();
    ps.with_start_of_line(false, |ps| {
        let parts_length = parts.len();
        for (idx, part) in parts.into_iter().enumerate() {
            format_inner_string(ps, vec![part], StringType::Array);
            if idx != parts_length - 1 {
                ps.emit_space();
            }
        }
    });
    ps.emit_close_square_bracket();
}

pub fn format_array(ps: &mut ParserState, array: Array) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    match array.1 {
        SimpleArrayOrPercentArray::SimpleArray(a) => format_array_fast_path(ps, a),
        SimpleArrayOrPercentArray::PercentArray(pa) => {
            ps.on_line((pa.2).0);
            format_percent_array(ps, pa.0, pa.1);
        },
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
    Array,
}

pub fn format_inner_string(ps: &mut ParserState, parts: Vec<StringContentPart>, tipe: StringType) {
    let mut peekable = parts.into_iter().peekable();
    while peekable.peek().is_some() {
        let part = peekable.next().expect("we peeked");
        match part {
            StringContentPart::TStringContent(t) => ps.emit_string_content(t.1),
            StringContentPart::StringEmbexpr(e) => {
                ps.emit_string_content("#{".to_string());
                ps.with_start_of_line(false, |ps| {
                    let expr = ((e.1).into_iter()).next().expect("should not be empty");
                    format_expression(ps, expr);
                });
                ps.emit_string_content("}".to_string());

                let on_line_skip = tipe == StringType::Heredoc
                    && match peekable.peek() {
                        Some(StringContentPart::TStringContent(TStringContent(_, s, _))) => {
                            s.starts_with("\n")
                        }
                        _ => false,
                    };
                if on_line_skip {
                    ps.render_heredocs(true)
                }
            }
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

pub fn format_heredoc_string_literal(
    ps: &mut ParserState,
    hd: HeredocStringLiteral,
    parts: Vec<StringContentPart>,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_surpress_comments(true, |ps| {
        let heredoc_type = (hd.1).0;
        let heredoc_symbol = (hd.1).1;
        ps.emit_ident(heredoc_type.clone());
        ps.emit_ident(heredoc_symbol.clone());

        ps.push_heredoc_content(heredoc_symbol, heredoc_type.contains("~"), parts);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_string_literal(ps: &mut ParserState, sl: StringLiteral) {
    let parts = (sl.2).1;
    // some(hd) if we have a heredoc
    match sl.1 {
        Some(hd) => {
            format_heredoc_string_literal(ps, hd, parts);
        }
        None => {
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
    }
}

pub fn format_xstring_literal(ps: &mut ParserState, xsl: XStringLiteral) {
    let parts = xsl.1;

    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident("`".to_string());
    format_inner_string(ps, parts, StringType::Quoted);
    ps.emit_ident("`".to_string());

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_const_path_field(ps: &mut ParserState, cf: ConstPathField) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        format_expression(ps, *cf.1);
        ps.emit_colon_colon();
        format_const(ps, cf.2);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_top_const_field(ps: &mut ParserState, tcf: TopConstField) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_colon_colon();
        format_const(ps, tcf.1);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_field(ps: &mut ParserState, vf: VarField) {
    let left = vf.1;
    format_var_ref_type(ps, left);
}

pub fn format_var_field_or_const_field_or_rest_param(ps: &mut ParserState, v: Assignable) {
    match v {
        Assignable::VarField(vf) => {
            format_var_field(ps, vf);
        }
        Assignable::ConstPathField(cf) => {
            format_const_path_field(ps, cf);
        }
        Assignable::RestParam(rp) => {
            format_rest_param(ps, Some(rp));
        }
        Assignable::TopConstField(tcf) => {
            format_top_const_field(ps, tcf);
        }
    }
}

pub fn format_assign(ps: &mut ParserState, assign: Assign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        format_var_field_or_const_field_or_rest_param(ps, assign.1);
        let right = assign.2;

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

pub fn format_massign(ps: &mut ParserState, massign: MAssign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        let length = massign.1.len();
        for (idx, v) in massign.1.into_iter().enumerate() {
            format_var_field_or_const_field_or_rest_param(ps, v);
            if idx != length - 1 {
                ps.emit_comma_space();
            }
        }
        ps.emit_space();
        ps.emit_ident("=".to_string());
        ps.emit_space();
        format_expression(ps, *massign.2);
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
        VarRefType::Const(c) => ps.emit_ident(c.1),
        VarRefType::Kw(kw) => ps.emit_ident(kw.1),
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_ref(ps: &mut ParserState, vr: VarRef) {
    format_var_ref_type(ps, vr.1);
}

pub fn format_const_path_ref(ps: &mut ParserState, cpr: ConstPathRef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        format_expression(ps, *cpr.1);
        ps.emit_colon_colon();
        format_const(ps, cpr.2);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_top_const_ref(ps: &mut ParserState, tcr: TopConstRef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_colon_colon();
        format_const(ps, tcr.1);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_defined(ps: &mut ParserState, defined: Defined) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_ident("defined?".to_string());
        ps.emit_open_paren();
        format_expression(ps, *defined.1);
        ps.emit_close_paren();
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_rescue_mod(ps: &mut ParserState, rescue_mod: RescueMod) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        format_expression(ps, *rescue_mod.1);
        ps.emit_space();
        ps.emit_rescue();
        ps.emit_space();
        format_expression(ps, *rescue_mod.2);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_mrhs_add_star(ps: &mut ParserState, mrhs: MRHSAddStar) {
    ps.with_start_of_line(false, |ps| {
        match mrhs.1 {
            MRHSNewFromArgsOrEmpty::Empty(e) => {
                if !e.is_empty() {
                    panic!("this should be impossible, got non-empty mrhs empty");
                }
            }
            MRHSNewFromArgsOrEmpty::MRHSNewFromArgs(mnfa) => {
                format_list_like_thing(ps, mnfa.1, true);
            }
        }
        ps.emit_ident("*".to_string());
        format_expression(ps, *mrhs.2);
    });
}

pub fn format_next(ps: &mut ParserState, next: Next) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        ps.emit_ident("next".to_string());
        match next.1 {
            ArgsAddBlockOrExpressionList::ExpressionList(e) => {
                if e.len() != 0 {
                    panic!("got non empty next expression list, should be impossible");
                }
            }
            ArgsAddBlockOrExpressionList::ArgsAddBlock(aab) => match aab.2 {
                MaybeBlock::ToProcExpr(_) => {
                    panic!("got a block in a next, should be impossible");
                }
                MaybeBlock::NoBlock(_) => {
                    ps.emit_space();
                    format_list_like_thing(ps, aab.1, true);
                }
            },
        }
        ps.emit_space();
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_conditional_mod(
    ps: &mut ParserState,
    right: Expression,
    left: Expression,
    conditional: String,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        format_expression(ps, left);
        ps.emit_space();
        ps.emit_ident(conditional);
        ps.emit_space();
        format_expression(ps, right);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_if_mod(ps: &mut ParserState, if_mod: IfMod) {
    format_conditional_mod(ps, *if_mod.1, *if_mod.2, "if".to_string());
}

pub fn format_unary(ps: &mut ParserState, unary: Unary) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(false, |ps| {
        match unary.1 {
            UnaryType::Not => {
                ps.emit_ident("not".to_string());
                ps.emit_space();
            }
        }

        format_expression(ps, *unary.2);
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_string_concat(ps: &mut ParserState, sc: StringConcat) {
    let nested = sc.1;
    let sl = sc.2;

    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    match nested {
        StringConcatOrStringLiteral::StringConcat(sc) => format_string_concat(ps, *sc),
        StringConcatOrStringLiteral::StringLiteral(sl) => format_string_literal(ps, sl),
    }

    ps.emit_space();
    ps.emit_slash();
    ps.emit_newline();

    ps.with_absorbing_indent_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            format_string_literal(ps, sl);
        });
    });

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_undef(ps: &mut ParserState, undef: Undef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident("undef ".to_string());
    let length = undef.1.len();
    for (idx, literal) in undef.1.into_iter().enumerate() {
        ps.with_start_of_line(false, |ps| format_symbol_literal(ps, literal));
        if idx != length-1 {
            ps.emit_comma_space();
        }
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_defs(ps: &mut ParserState, defs: Defs) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let singleton = defs.1;
    let ident = defs.3;
    let paren_or_params = defs.4;
    let bodystmt = defs.5;

    ps.emit_def_keyword();
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        match singleton {
            Singleton::VarRef(vr) => {
                format_var_ref(ps, vr);
            },
            Singleton::Paren(pe) => {
                format_paren(ps, pe);
            }
        }

        ps.emit_dot();
        format_ident(ps, ident);
        format_paren_or_params(ps, paren_or_params);
        ps.emit_newline();
    });

    ps.with_formatting_context(FormattingContext::Def, |ps| {
        ps.new_block(|ps| {
            format_bodystmt(ps, bodystmt, false);
        });
    });

    ps.emit_end();

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_paren_or_params(ps: &mut ParserState, pp: ParenOrParams) {
    let params = match pp {
        ParenOrParams::Paren(p) => p.1,
        ParenOrParams::Params(p) => p,
    };
    format_params(ps, params, "(".to_string(), ")".to_string());
}

pub fn format_class(ps: &mut ParserState, class: Class) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let class_name = class.1;
    let inherit = class.2;
    let bodystmt = class.3;

    ps.emit_class_keyword();
    ps.with_start_of_line(false, |ps| {
        ps.emit_space();

        match class_name {
            ConstPathRefOrConstRef::ConstPathRef(cpr) => {
                format_const_path_ref(ps, cpr);
            },
            ConstPathRefOrConstRef::ConstRef(cr) => {
                ps.on_line(((cr.1).2).0);
                ps.emit_ident((cr.1).1);
            }
        }

        if inherit.is_some() {
            let inherit_expression = *(inherit.expect("We checked it is some"));
            ps.emit_ident(" < ".to_string());
            format_expression(ps, inherit_expression);
        }
    });

    ps.emit_newline();
    ps.new_block(|ps| {
        ps.with_formatting_context(FormattingContext::ClassOrModule, |ps| {
            format_bodystmt(ps, bodystmt, false);
        });
    });

    ps.emit_end();
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_conditional(ps: &mut ParserState, cond_expr: Expression, body: Vec<Expression>, kw: String, tail: Option<ElsifOrElse>) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    ps.emit_keyword(kw);
    ps.emit_space();
    ps.with_start_of_line(false, |ps| {
        format_expression(ps, cond_expr);
    });
    ps.emit_newline();

    ps.with_start_of_line(true, |ps| {
        ps.new_block(|ps| {
            for expr in body.into_iter() {
                format_expression(ps, expr);
            }
        });
    });
    ps.with_start_of_line(true, |ps| {
        match tail {
            None => {},
            Some(ElsifOrElse::Elsif(elsif)) => {
                ps.emit_newline();
                format_conditional(ps, *elsif.1, elsif.2, "elsif".to_string(), (elsif.3).map(|v| *v));
            },
            Some(ElsifOrElse::Else(els)) => {
                ps.emit_newline();
                ps.emit_indent();
                ps.emit_else();
                ps.emit_newline();
                ps.with_start_of_line(true, |ps| {
                    ps.new_block(|ps| {
                        for expr in els.1 {
                            format_expression(ps, expr);
                        }
                    });
                });
            }
        }
    });
}

pub fn format_if(ps: &mut ParserState, ifs: If) {
    format_conditional(ps, *ifs.1, ifs.2, "if".to_string(), ifs.3);
    ps.emit_end();
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
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
        Expression::Dot3(dot3) => format_dot3(ps, dot3),
        Expression::SymbolLiteral(sl) => format_symbol_literal(ps, sl),
        Expression::Alias(alias) => format_alias(ps, alias),
        Expression::Array(array) => format_array(ps, array),
        Expression::StringLiteral(sl) => format_string_literal(ps, sl),
        Expression::XStringLiteral(xsl) => format_xstring_literal(ps, xsl),
        Expression::Assign(assign) => format_assign(ps, assign),
        Expression::VarRef(vr) => format_var_ref(ps, vr),
        Expression::ConstPathRef(cpr) => format_const_path_ref(ps, cpr),
        Expression::TopConstRef(tcr) => format_top_const_ref(ps, tcr),
        Expression::Defined(defined) => format_defined(ps, defined),
        Expression::RescueMod(rescue_mod) => format_rescue_mod(ps, rescue_mod),
        Expression::MRHSAddStar(mrhs) => format_mrhs_add_star(ps, mrhs),
        Expression::MAssign(massign) => format_massign(ps, massign),
        Expression::Next(next) => format_next(ps, next),
        Expression::IfMod(if_mod) => format_if_mod(ps, if_mod),
        Expression::Unary(unary) => format_unary(ps, unary),
        Expression::StringConcat(sc) => format_string_concat(ps, sc),
        Expression::Undef(undef) => format_undef(ps, undef),
        Expression::Class(class) => format_class(ps, class),
        Expression::Defs(defs) => format_defs(ps, defs),
        Expression::If(ifs) => format_if(ps, ifs),
        e => {
            panic!("got unknown token: {:?}", e);
        }
    }
}

pub fn format_program(ps: &mut ParserState, program: Program) {
    println!("{:?}", program);
    for expression in program.1 {
        format_expression(ps, expression);
    }
}
