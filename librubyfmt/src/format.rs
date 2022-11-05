use crate::delimiters::BreakableDelims;
use crate::heredoc_string::HeredocKind;
use crate::parser_state::{BaseParserState, ConcreteParserState, FormattingContext, RenderFunc};
use crate::ripper_tree_types::*;
use crate::types::LineNumber;
use log::debug;

pub fn format_def(ps: &mut dyn ConcreteParserState, def: Def) {
    let def_expression = (def.1).to_def_parts();

    let body = def.3;
    let pp = def.2;
    let end_line = (def.4).1;
    ps.on_line((def_expression.1).0);
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    ps.emit_def(def_expression.0);
    ps.new_scope(Box::new(|ps| {
        format_paren_or_params(ps, pp);

        ps.with_formatting_context(
            FormattingContext::Def,
            Box::new(|ps| {
                ps.new_block(Box::new(|ps| {
                    ps.emit_newline();
                    ps.with_start_of_line(
                        true,
                        Box::new(|ps| {
                            format_bodystmt(ps, body, end_line);
                        }),
                    );
                }));
            }),
        );
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.wind_dumping_comments_until_line(end_line);
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

type ParamFormattingFunc = Box<dyn FnOnce(&mut dyn ConcreteParserState) -> bool>;

pub fn inner_format_params(ps: &mut dyn ConcreteParserState, params: Box<Params>) {
    let non_null_positions = params.non_null_positions();
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
    let required_params = (params.1).unwrap_or_default();
    let optional_params = (params.2).unwrap_or_default();
    let rest_param = params.3;
    let more_required_params = (params.4).unwrap_or_default();
    let kwargs = (params.5).unwrap_or_default();
    let kwrest_params = params.6;
    let block_arg = params.7;

    let formats: Vec<ParamFormattingFunc> = vec![
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            format_required_params(ps, required_params)
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            format_optional_params(ps, optional_params)
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            format_rest_param(ps, rest_param, SpecialCase::NoSpecialCase)
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            format_required_params(ps, more_required_params)
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| format_kwargs(ps, kwargs)),
        Box::new(move |ps: &mut dyn ConcreteParserState| format_kwrest_params(ps, kwrest_params)),
        Box::new(move |ps: &mut dyn ConcreteParserState| format_block_arg(ps, block_arg)),
    ];

    for (idx, format_fn) in formats.into_iter().enumerate() {
        let did_emit = format_fn(ps);
        let have_more = non_null_positions[idx + 1..].iter().any(|&v| v);

        if did_emit && have_more {
            ps.emit_comma();
            ps.emit_soft_newline();
        }
        ps.shift_comments();
    }
}

pub fn format_blockvar(ps: &mut dyn ConcreteParserState, bv: BlockVar) {
    let start_end = bv.3;
    let f_params = match bv.2 {
        BlockLocalVariables::Present(v) => Some(v),
        _ => None,
    };

    let params = bv.1;

    let have_any_params = match &params {
        Some(params) => params.non_null_positions().iter().any(|&v| v) || f_params.is_some(),
        None => f_params.is_some(),
    };

    if !have_any_params {
        return;
    }

    ps.new_block(Box::new(|ps| {
        ps.breakable_of(
            BreakableDelims::for_block_params(),
            Box::new(|ps| {
                if let Some(params) = params {
                    inner_format_params(ps, params);
                }

                match f_params {
                    None => {}
                    Some(f_params) => {
                        if !f_params.is_empty() {
                            ps.emit_ident(";".to_string());

                            ps.with_start_of_line(
                                false,
                                Box::new(|ps| {
                                    format_list_like_thing_items(
                                        ps,
                                        f_params.into_iter().map(Expression::Ident).collect(),
                                        None,
                                        true,
                                    );
                                }),
                            );
                        }
                    }
                }
                ps.emit_collapsing_newline();
            }),
        );
    }));

    ps.on_line(start_end.end_line());
}

pub fn format_params(
    ps: &mut dyn ConcreteParserState,
    params: Box<Params>,
    delims: BreakableDelims,
) {
    let have_any_params = params.non_null_positions().iter().any(|&x| x);
    if !have_any_params {
        return;
    }

    let end_line = params.8.end_line();

    ps.breakable_of(
        delims,
        Box::new(|ps| {
            inner_format_params(ps, params);
            ps.emit_collapsing_newline();
            ps.wind_dumping_comments_until_line(end_line);
        }),
    );
}

pub fn format_kwrest_params(
    ps: &mut dyn ConcreteParserState,
    kwrest_params: Option<KwRestParam>,
) -> bool {
    if kwrest_params.is_none() {
        return false;
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_soft_indent();
            ps.emit_ident("**".to_string());
            let ident = (kwrest_params.unwrap()).1;
            if let Some(ident) = ident {
                bind_ident(ps, &ident);
                format_ident(ps, ident);
            }
        }),
    );
    true
}

pub fn format_block_arg(ps: &mut dyn ConcreteParserState, block_arg: Option<BlockArg>) -> bool {
    match block_arg {
        None => false,
        Some(ba) => {
            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    ps.emit_soft_indent();
                    ps.emit_ident("&".to_string());
                    bind_ident(ps, &ba.1);
                    format_ident(ps, ba.1);
                }),
            );

            true
        }
    }
}

pub fn format_kwargs(
    ps: &mut dyn ConcreteParserState,
    kwargs: Vec<(Label, ExpressionOrFalse)>,
) -> bool {
    if kwargs.is_empty() {
        return false;
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let len = kwargs.len();
            for (idx, (label, expr_or_false)) in kwargs.into_iter().enumerate() {
                ps.emit_soft_indent();
                ps.bind_variable(
                    (label.1)
                        .strip_suffix(':')
                        .expect("Labels are passed through with trailing colons")
                        .to_string(),
                );
                handle_string_and_linecol(ps, label.1, label.2);

                match expr_or_false {
                    ExpressionOrFalse::Expression(e) => {
                        ps.emit_space();
                        format_expression(ps, e);
                    }
                    ExpressionOrFalse::False(_) => {}
                }
                emit_params_separator(ps, idx, len);
            }
        }),
    );

    true
}

pub fn format_rest_param(
    ps: &mut dyn ConcreteParserState,
    rest_param: Option<RestParamOr0OrExcessedCommaOrArgsForward>,
    special_case: SpecialCase,
) -> bool {
    let mut res = false;
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            match rest_param {
                None => {}
                Some(RestParamOr0OrExcessedCommaOrArgsForward::ExcessedComma(_)) => {}
                Some(RestParamOr0OrExcessedCommaOrArgsForward::Zero(_)) => {}
                Some(RestParamOr0OrExcessedCommaOrArgsForward::ArgsForward(_)) => {
                    ps.emit_ellipsis();
                }
                Some(RestParamOr0OrExcessedCommaOrArgsForward::RestParam(rp)) => {
                    if special_case != SpecialCase::RestParamOutsideOfParamDef {
                        ps.emit_soft_indent();
                    }
                    ps.emit_ident("*".to_string());
                    ps.with_start_of_line(
                        false,
                        Box::new(|ps| {
                            match rp.1 {
                                Some(RestParamAssignable::Ident(i)) => {
                                    bind_ident(ps, &i);
                                    format_ident(ps, i);
                                }
                                Some(RestParamAssignable::VarField(vf)) => {
                                    bind_var_field(ps, &vf);
                                    format_var_field(ps, vf);
                                }
                                Some(RestParamAssignable::ArefField(aref_field)) => {
                                    // No need to bind, hash value must have been previously bound
                                    format_aref_field(ps, aref_field);
                                }
                                None => {
                                    // deliberately do nothing
                                }
                            }
                        }),
                    );

                    res = true;
                }
            }
        }),
    );
    res
}

pub fn format_optional_params(
    ps: &mut dyn ConcreteParserState,
    optional_params: Vec<(Ident, Expression)>,
) -> bool {
    if optional_params.is_empty() {
        return false;
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let len = optional_params.len();
            for (idx, (left, right)) in optional_params.into_iter().enumerate() {
                ps.emit_soft_indent();
                bind_ident(ps, &left);
                format_ident(ps, left);
                ps.emit_ident(" = ".to_string());
                format_expression(ps, right);
                emit_params_separator(ps, idx, len);
            }
        }),
    );

    true
}

pub fn format_mlhs(ps: &mut dyn ConcreteParserState, mlhs: MLhs) {
    ps.emit_open_paren();

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let mut first = true;
            for inner in mlhs.0 {
                if !first {
                    ps.emit_comma_space();
                }
                first = false;

                match inner {
                    MLhsInner::Field(f) => format_field(ps, f),
                    MLhsInner::Ident(i) => format_ident(ps, i),
                    MLhsInner::RestParam(rp) => {
                        format_rest_param(
                            ps,
                            Some(RestParamOr0OrExcessedCommaOrArgsForward::RestParam(rp)),
                            SpecialCase::NoSpecialCase,
                        );
                    }
                    MLhsInner::VarField(vf) => format_var_field(ps, vf),
                    MLhsInner::MLhs(mlhs) => format_mlhs(ps, *mlhs),
                }
            }
        }),
    );

    ps.emit_close_paren();
}

fn bind_var_field(ps: &mut dyn ConcreteParserState, vf: &VarField) {
    ps.bind_variable((vf.1).clone().to_local_string())
}

fn bind_ident(ps: &mut dyn ConcreteParserState, id: &Ident) {
    ps.bind_variable((id.1).clone())
}

fn bind_mlhs(ps: &mut dyn ConcreteParserState, mlhs: &MLhs) {
    for value in (mlhs.0).iter() {
        match value {
            MLhsInner::VarField(v) => bind_var_field(ps, v),
            MLhsInner::Field(_) => {
                // TODO(penelopezone) is something missing here?
            }
            MLhsInner::RestParam(v) => match v.1 {
                Some(RestParamAssignable::Ident(ref i)) => bind_ident(ps, i),
                Some(RestParamAssignable::VarField(ref v)) => bind_var_field(ps, v),
                Some(RestParamAssignable::ArefField(..)) | None => {}
            },
            MLhsInner::Ident(i) => bind_ident(ps, i),
            MLhsInner::MLhs(m) => bind_mlhs(ps, m),
        }
    }
}

pub fn format_required_params(
    ps: &mut dyn ConcreteParserState,
    required_params: Vec<IdentOrMLhs>,
) -> bool {
    if required_params.is_empty() {
        return false;
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let len = required_params.len();
            for (idx, ident) in required_params.into_iter().enumerate() {
                ps.emit_soft_indent();
                match ident {
                    IdentOrMLhs::Ident(ident) => {
                        bind_ident(ps, &ident);
                        format_ident(ps, ident);
                    }
                    IdentOrMLhs::MLhs(mlhs) => {
                        bind_mlhs(ps, &mlhs);
                        format_mlhs(ps, mlhs);
                    }
                }
                emit_params_separator(ps, idx, len);
            }
        }),
    );
    true
}

pub fn emit_params_separator(ps: &mut dyn ConcreteParserState, index: usize, length: usize) {
    if index != length - 1 {
        ps.emit_comma();
        ps.emit_soft_newline();
    }
}

pub fn format_bodystmt(
    ps: &mut dyn ConcreteParserState,
    bodystmt: Box<BodyStmt>,
    end_line: LineNumber,
) {
    let expressions = bodystmt.1;
    let rescue_part = bodystmt.2;
    let else_part = bodystmt.3;
    let ensure_part = bodystmt.4;

    for expression in expressions {
        format_expression(ps, expression);
    }

    // Else statements are actually just an array of statements in many cases,
    // which means we don't get an "end point" from the parser. Instead, we need
    // to deduce the end point from other nodes. In this case, there are three options
    // (1) There's no else clause, so what we pass doesn't matter at all
    // (2) There's an else clause but no ensure clause, so we can assume the end of the
    //     else clause is the same as the end of the entire body
    // (3) There's an else clause with an ensure block, in which case the else clause must end
    //     must end wherever the ensure clause begins
    let else_end_line = if let Some(ref ensure) = ensure_part {
        ensure.2.start_line()
    } else {
        end_line
    };

    format_rescue(ps, rescue_part);
    format_else(ps, else_part, else_end_line);
    format_ensure(ps, ensure_part);
}

pub fn format_mrhs(ps: &mut dyn ConcreteParserState, mrhs: Option<MRHS>) {
    match mrhs {
        None => {}
        Some(MRHS::Single(expr)) => {
            format_expression(ps, *expr);
        }
        Some(MRHS::SingleAsArray(exprs)) => {
            if exprs.len() != 1 {
                panic!("this should be impossible, bug in the ruby parser?");
            }
            format_expression(
                ps,
                exprs
                    .into_iter()
                    .next()
                    .expect("we checked there's one item"),
            );
        }
        Some(MRHS::MRHSNewFromArgs(mnfa)) => {
            format_mrhs_new_from_args(ps, mnfa);
        }
        Some(MRHS::MRHSAddStar(mas)) => {
            format_mrhs_add_star(ps, mas);
        }
        Some(MRHS::Array(array)) => {
            format_array(ps, array);
        }
    }
}

pub fn format_rescue_capture(
    ps: &mut dyn ConcreteParserState,
    rescue_capture: Option<Assignable>,
    class_present: bool,
) {
    match rescue_capture {
        None => {}
        Some(expr) => {
            if class_present {
                ps.emit_space();
            }
            ps.emit_ident("=>".to_string());
            ps.emit_space();
            format_assignable(ps, expr);
        }
    }
}

pub fn format_rescue(ps: &mut dyn ConcreteParserState, rescue_part: Option<Rescue>) {
    match rescue_part {
        None => {}
        Some(Rescue(_, class, capture, expressions, more_rescue, start_end)) => {
            ps.on_line(start_end.start_line());

            ps.dedent(Box::new(|ps| {
                ps.emit_indent();
                ps.emit_rescue();
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        if class.is_none()
                            && capture.is_none()
                            && expressions
                                .as_ref()
                                .map(|expr| !is_empty_bodystmt(expr))
                                .unwrap_or(false)
                        {
                            return;
                        }
                        let cs = class.is_some();
                        if cs || capture.is_some() {
                            ps.emit_space();
                        }

                        format_mrhs(ps, class);
                        format_rescue_capture(ps, capture, cs);
                    }),
                );
            }));

            match expressions {
                None => {}
                Some(expressions) => {
                    ps.emit_newline();
                    for expression in expressions {
                        format_expression(ps, expression);
                    }
                }
            }

            format_rescue(ps, more_rescue.map(|v| *v));

            ps.wind_dumping_comments_until_line(start_end.end_line());
        }
    }
}

pub fn format_else(
    ps: &mut dyn ConcreteParserState,
    else_part: Option<RescueElseOrExpressionList>,
    end_line: LineNumber,
) {
    match else_part {
        None => {}
        Some(RescueElseOrExpressionList::ExpressionList(exprs)) => {
            ps.dedent(Box::new(|ps| {
                ps.emit_indent();
                ps.emit_else();
            }));
            ps.emit_newline();
            ps.with_start_of_line(
                true,
                Box::new(|ps| {
                    for expr in exprs {
                        format_expression(ps, expr);
                    }
                }),
            );
            ps.wind_dumping_comments_until_line(end_line);
        }
        Some(RescueElseOrExpressionList::RescueElse(re)) => {
            ps.on_line(re.2.start_line());

            ps.dedent(Box::new(|ps| {
                ps.emit_indent();
                ps.emit_else();
            }));

            match re.1 {
                None => {}
                Some(exprs) => {
                    ps.emit_newline();
                    ps.with_start_of_line(
                        true,
                        Box::new(|ps| {
                            for expr in exprs {
                                format_expression(ps, expr);
                            }
                        }),
                    );
                }
            }

            ps.wind_dumping_comments_until_line(re.2.end_line());
        }
    }
}

pub fn format_ensure(ps: &mut dyn ConcreteParserState, ensure_part: Option<Ensure>) {
    match ensure_part {
        None => {}
        Some(e) => {
            ps.on_line(e.2.start_line());

            ps.dedent(Box::new(|ps| {
                ps.emit_indent();
                ps.emit_ensure();
            }));

            match e.1 {
                None => {}
                Some(exprs) => {
                    ps.emit_newline();
                    ps.with_start_of_line(
                        true,
                        Box::new(|ps| {
                            for expr in exprs {
                                format_expression(ps, expr);
                            }
                        }),
                    );
                }
            }
            ps.wind_dumping_comments_until_line(e.2.end_line());
        }
    }
}

pub fn args_has_single_def_expression(args: &ArgsAddStarOrExpressionListOrArgsForward) -> bool {
    if let ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el) = args {
        if el.len() != 1 {
            return false;
        }

        if let Some(Expression::Def(_) | Expression::Defs(_)) = el.first() {
            return true;
        }
    }

    false
}

pub fn use_parens_for_method_call(
    ps: &dyn ConcreteParserState,
    chain: &[CallChainElement],
    method: &IdentOrOpOrKeywordOrConst,
    args: &ArgsAddStarOrExpressionListOrArgsForward,
    original_used_parens: bool,
    context: FormattingContext,
) -> bool {
    let name = method.get_name();
    debug!("name: {:?}", name);
    if name.starts_with("attr_") && context == FormattingContext::ClassOrModule {
        return original_used_parens;
    }

    if ps.scope_has_variable(&name) {
        match chain.first() {
            None => return original_used_parens,
            Some(CallChainElement::VarRef(VarRef(_, VarRefType::Kw(Kw(_, x, _))))) => {
                if x == "self" {
                    return true;
                }
            }
            _ => {}
        }
    }

    if name == "yield" {
        debug!("yield paren: {:?}", original_used_parens);
        return ps.current_formatting_context_requires_parens() || original_used_parens;
    }

    if name == "return" || name == "raise" || name == "break" {
        if ps.current_formatting_context_requires_parens() {
            return true;
        }
        match args {
            ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(_) => return true,
            _ => return false,
        }
    }

    if name == "super" || name == "require" || name == "require_relative" {
        return original_used_parens;
    }

    if args.is_empty() {
        return false;
    } else {
        // If the first argument to this method call is `def`, we don't want
        // to use parens. Example:
        //
        //   private def foo
        //   end
        if args_has_single_def_expression(args) {
            return false;
        }
    }

    if context == FormattingContext::ClassOrModule && !original_used_parens {
        return false;
    }

    true
}

pub fn format_dot_type(ps: &mut dyn ConcreteParserState, dt: DotType) {
    match dt {
        DotType::Dot(_) => ps.emit_dot(),
        DotType::LonelyOperator(_) => ps.emit_lonely_operator(),
    }
}

pub fn format_dot(ps: &mut dyn ConcreteParserState, dot: DotTypeOrOp) {
    match dot {
        DotTypeOrOp::DotType(dt) => format_dot_type(ps, dt),
        DotTypeOrOp::Op(op) => {
            let lc = op.2;
            ps.on_line(lc.0);
            match op.1 {
                Operator::Dot(dot) => format_dot_type(ps, DotType::Dot(dot)),
                Operator::LonelyOperator(dot) => format_dot_type(ps, DotType::LonelyOperator(dot)),
                Operator::StringOperator(string) => ps.emit_ident(string),
                x => panic!(
                    "should be impossible, dot position operator parsed as not a dot, {:?}",
                    x
                ),
            }
        }
        DotTypeOrOp::Period(p) => {
            ps.on_line(p.2 .0);
            ps.emit_dot();
        }
        DotTypeOrOp::ColonColon(_) => {
            ps.emit_colon_colon();
        }
        DotTypeOrOp::StringDot(s) => {
            ps.emit_ident(s);
        }
    }
}

pub fn format_method_call(ps: &mut dyn ConcreteParserState, method_call: MethodCall) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let should_multiline_call_chain = should_multiline_call_chain(ps, &method_call);
    let MethodCall(_, chain, method, original_used_parens, args, start_end) = method_call;

    debug!("method call!!");
    let use_parens = use_parens_for_method_call(
        ps,
        &chain,
        &method,
        &args,
        original_used_parens,
        ps.current_formatting_context(),
    );

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let is_indented = format_call_chain(ps, chain, should_multiline_call_chain);
            if is_indented {
                ps.start_indent();
            }

            if use_parens && method.get_name() == ".()" {
                ps.emit_ident(".".to_string());
            } else {
                format_ident(ps, method.into_ident());
            }
            let delims = if use_parens {
                BreakableDelims::for_method_call()
            } else {
                BreakableDelims::for_kw()
            };
            if !args.is_empty() {
                if args_has_single_def_expression(&args) {
                    // If we match `def ...` as the first argument, just
                    // emit it without any delimiters.
                    ps.emit_space();

                    if let ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(mut el) = args {
                        let expr = el.pop().expect("checked the list is not empty");

                        if let Expression::Def(def_expression) = expr {
                            format_def(ps, def_expression);
                        } else if let Expression::Defs(defs_expression) = expr {
                            format_defs(ps, defs_expression);
                        }
                    }
                } else {
                    let force_single_line = matches!(
                        args,
                        ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(..)
                    );

                    let maybe_end_line = start_end.map(|se| se.1);

                    ps.breakable_of(
                        delims,
                        Box::new(|ps| {
                            ps.with_formatting_context(
                                FormattingContext::ArgsList,
                                Box::new(|ps| {
                                    format_list_like_thing(
                                        ps,
                                        args,
                                        maybe_end_line,
                                        force_single_line,
                                    );
                                    ps.emit_collapsing_newline();
                                }),
                            );
                            debug!("end of format method call");
                        }),
                    );
                    if let Some(end_line) = maybe_end_line {
                        ps.wind_dumping_comments_until_line(end_line);
                    }
                }
            } else if use_parens {
                ps.emit_open_paren();
                ps.emit_close_paren();
            }

            if is_indented {
                ps.end_indent();
            }
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCase {
    NoSpecialCase,
    NoLeadingTrailingCollectionMarkers,
    RestParamOutsideOfParamDef,
}

pub fn format_list_like_thing_items(
    ps: &mut dyn ConcreteParserState,
    args: Vec<Expression>,
    end_line: Option<LineNumber>,
    single_line: bool,
) -> bool {
    let mut emitted_args = false;
    let skip_magic_comments = args
        .iter()
        .any(|i| matches!(i, Expression::StringConcat(..)));
    let args_count = args.len();
    let cls: RenderFunc = Box::new(|ps| {
        for (idx, expr) in args.into_iter().enumerate() {
            if single_line {
                match expr {
                    Expression::BareAssocHash(bah) => format_assocs_single_line(ps, bah.1),
                    expr => format_expression(ps, expr),
                }
                if idx != args_count - 1 {
                    ps.emit_comma_space();
                }
            } else {
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        match expr {
                            Expression::BareAssocHash(bah) => format_assocs(
                                ps,
                                bah.1,
                                SpecialCase::NoLeadingTrailingCollectionMarkers,
                            ),
                            expr => {
                                ps.emit_soft_indent();
                                format_expression(ps, expr);
                            }
                        }
                        if idx != args_count - 1 {
                            ps.emit_comma();
                            ps.emit_soft_newline();
                        } else {
                            ps.shift_comments();
                        }
                    }),
                );
            };
            emitted_args = true;
        }
    });

    if skip_magic_comments {
        cls(ps)
    } else {
        ps.magic_handle_comments_for_multiline_arrays(end_line, cls);
    }
    emitted_args
}

pub fn format_ident(ps: &mut dyn ConcreteParserState, ident: Ident) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, ident.1, ident.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_const(ps: &mut dyn ConcreteParserState, c: Const) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, c.1, c.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_int(ps: &mut dyn ConcreteParserState, int: Int) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, int.1, int.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_bare_assoc_hash(ps: &mut dyn ConcreteParserState, bah: BareAssocHash) {
    format_assocs(ps, bah.1, SpecialCase::NoSpecialCase)
}

pub fn format_alias(ps: &mut dyn ConcreteParserState, alias: Alias) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident("alias ".to_string());

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_symbol_literal_or_dyna_symbol(ps, alias.1);
            ps.emit_space();
            format_symbol_literal_or_dyna_symbol(ps, alias.2);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_symbol_literal_or_dyna_symbol(
    ps: &mut dyn ConcreteParserState,
    symbol_literal_or_dyna_symbol: SymbolLiteralOrDynaSymbol,
) {
    match symbol_literal_or_dyna_symbol {
        SymbolLiteralOrDynaSymbol::DynaSymbol(dyna_symbol) => format_dyna_symbol(ps, dyna_symbol),
        SymbolLiteralOrDynaSymbol::SymbolLiteral(symbol_literal) => {
            format_symbol_literal(ps, symbol_literal)
        }
    }
}

pub fn format_op(ps: &mut dyn ConcreteParserState, op: Op) {
    match op.1 {
        Operator::Equals(_) => ps.emit_ident("==".to_string()),
        Operator::Dot(_) => ps.emit_dot(),
        Operator::LonelyOperator(_) => ps.emit_lonely_operator(),
        Operator::StringOperator(s) => ps.emit_ident(s),
    }
}

pub fn format_kw(ps: &mut dyn ConcreteParserState, kw: Kw) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, kw.1, kw.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_backtick(ps: &mut dyn ConcreteParserState, backtick: Backtick) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, backtick.1, backtick.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_symbol(ps: &mut dyn ConcreteParserState, symbol: Symbol) {
    ps.emit_ident(":".to_string());
    match symbol.1 {
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Ident(i) => format_ident(ps, i),
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Const(c) => format_const(ps, c),
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Keyword(kw) => format_kw(ps, kw),
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Op(op) => format_op(ps, op),
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::IVar(ivar) => {
            format_var_ref_type(ps, VarRefType::IVar(ivar))
        }
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::GVar(gvar) => {
            format_var_ref_type(ps, VarRefType::GVar(gvar))
        }
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::CVar(cvar) => {
            format_var_ref_type(ps, VarRefType::CVar(cvar))
        }
        IdentOrConstOrKwOrOpOrIvarOrGvarOrCvarOrBacktick::Backtick(backtick) => {
            format_backtick(ps, backtick)
        }
    }
}

pub fn format_symbol_literal(ps: &mut dyn ConcreteParserState, symbol_literal: SymbolLiteral) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.on_line(symbol_literal.2.start_line());

    ps.with_start_of_line(
        false,
        Box::new(|ps| match symbol_literal.1 {
            SymbolOrBare::Ident(ident) => format_ident(ps, ident),
            SymbolOrBare::Kw(kw) => format_kw(ps, kw),
            SymbolOrBare::Op(op) => format_op(ps, op),
            SymbolOrBare::Symbol(symbol) => format_symbol(ps, symbol),
            SymbolOrBare::GVar(gvar) => format_var_ref_type(ps, VarRefType::GVar(gvar)),
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn all_labelish(assocs: &[AssocNewOrAssocSplat]) -> bool {
    assocs.iter().all(|assoc| match assoc {
        AssocNewOrAssocSplat::AssocNew(new) => match new.1 {
            AssocKey::Label(_) => true,
            AssocKey::Expression(_) => false,
        },
        AssocNewOrAssocSplat::AssocSplat(_) => true,
    })
}

pub fn format_assocs(
    ps: &mut dyn ConcreteParserState,
    assocs: Vec<AssocNewOrAssocSplat>,
    sc: SpecialCase,
) {
    let len = assocs.len();
    let all_labelish = all_labelish(&assocs);
    for (idx, assoc) in assocs.into_iter().enumerate() {
        ps.emit_soft_indent();
        format_assoc(ps, assoc, all_labelish);
        if idx != len - 1 {
            ps.emit_comma();
        }
        if !(idx == len - 1 && sc == SpecialCase::NoLeadingTrailingCollectionMarkers) {
            ps.emit_soft_newline();
        }
    }
}

pub fn format_assocs_single_line(
    ps: &mut dyn ConcreteParserState,
    assocs: Vec<AssocNewOrAssocSplat>,
) {
    let len = assocs.len();
    let all_labelish = all_labelish(&assocs);
    for (idx, assoc) in assocs.into_iter().enumerate() {
        format_assoc(ps, assoc, all_labelish);
        if idx != len - 1 {
            ps.emit_comma_space();
        }
    }
}

pub fn format_assoc(
    ps: &mut dyn ConcreteParserState,
    assoc: AssocNewOrAssocSplat,
    all_labelish: bool,
) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| match assoc {
            AssocNewOrAssocSplat::AssocNew(new) => {
                match new.1 {
                    AssocKey::Label(label) => {
                        if all_labelish {
                            handle_string_and_linecol(ps, label.1, label.2);
                        } else {
                            let colonless_label = label
                                .1
                                .strip_suffix(':')
                                .expect("labels end with a colon")
                                .to_owned();
                            format_symbol(ps, Symbol::from_string(colonless_label, label.2));
                            ps.emit_space();
                            ps.emit_ident("=>".to_string());
                        }
                        ps.emit_space();
                    }
                    AssocKey::Expression(expression) => {
                        format_expression(ps, expression);
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
        }),
    );
}

pub fn format_begin(ps: &mut dyn ConcreteParserState, begin: Begin) {
    if ps.at_start_of_line() {
        ps.emit_indent()
    }

    ps.on_line(begin.1 .0);

    ps.emit_begin();

    ps.new_block(Box::new(|ps| {
        ps.emit_newline();
        ps.with_start_of_line(
            true,
            Box::new(|ps| format_bodystmt(ps, begin.2, begin.1.end_line())),
        );
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_begin_block(ps: &mut dyn ConcreteParserState, begin: BeginBlock) {
    if ps.at_start_of_line() {
        ps.emit_indent()
    }

    ps.wind_line_forward();
    ps.emit_begin_block();
    ps.emit_space();
    ps.emit_open_curly_bracket();
    ps.emit_newline();
    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                for expr in begin.1 {
                    format_expression(ps, expr);
                }
            }),
        );
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_close_curly_bracket();
        }),
    );
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_end_block(ps: &mut dyn ConcreteParserState, end: EndBlock) {
    if ps.at_start_of_line() {
        ps.emit_indent()
    }

    ps.wind_line_forward();
    ps.emit_end_block();
    ps.emit_space();
    ps.emit_open_curly_bracket();
    ps.emit_newline();

    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                for expr in end.1 {
                    format_expression(ps, expr);
                }
            }),
        );
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_close_curly_bracket();
        }),
    );
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn normalize(e: Expression) -> Expression {
    match e {
        Expression::VCall(v) => Expression::MethodCall(v.to_method_call()),
        Expression::MethodAddArg(maa) => Expression::MethodCall(maa.to_method_call()),
        Expression::Command(command) => Expression::MethodCall(command.to_method_call()),
        Expression::CommandCall(call) => Expression::MethodCall(call.to_method_call()),
        Expression::Call(call) => Expression::MethodCall(call.to_method_call()),
        Expression::Super(sup) => Expression::MethodCall(sup.to_method_call()),
        e => e,
    }
}

pub fn format_void_stmt(_ps: &mut dyn ConcreteParserState, _void: VoidStmt) {
    // deliberately does nothing
}

pub fn format_paren(ps: &mut dyn ConcreteParserState, paren: ParenExpr) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    let start_end = paren.2;
    ps.emit_open_paren();

    match paren.1 {
        ParenExpressionOrExpressions::Expressions(exps) => {
            if exps.len() == 1 {
                let p = exps.into_iter().next().expect("we know this isn't empty");
                ps.with_start_of_line(false, Box::new(|ps| format_expression(ps, p)));
            } else {
                // We don't have a line for the opening paren, so just wind until we see an expression
                ps.wind_dumping_comments(None);
                ps.emit_newline();
                ps.new_block(Box::new(|ps| {
                    ps.with_start_of_line(
                        true,
                        Box::new(|ps| {
                            for expr in exps.into_iter() {
                                format_expression(ps, expr);
                            }
                        }),
                    );
                }));
                ps.emit_indent();
            }
        }
        ParenExpressionOrExpressions::Expression(expr) => {
            format_expression(ps, *expr);
        }
    }
    ps.emit_close_paren();
    ps.wind_dumping_comments_until_line(start_end.end_line());
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_dot2(ps: &mut dyn ConcreteParserState, dot2: Dot2) {
    format_dot2_or_3(ps, "..".to_string(), dot2.1, dot2.2);
}

pub fn format_dot3(ps: &mut dyn ConcreteParserState, dot3: Dot3) {
    format_dot2_or_3(ps, "...".to_string(), dot3.1, dot3.2);
}

pub fn format_dot2_or_3(
    ps: &mut dyn ConcreteParserState,
    dots: String,
    left: Option<Box<Expression>>,
    right: Option<Box<Expression>>,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if let Some(expr) = left {
                format_expression(ps, *expr)
            }

            ps.emit_ident(dots);

            if let Some(expr) = right {
                format_expression(ps, *expr)
            }
        }),
    );

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

pub fn format_percent_array(
    ps: &mut dyn ConcreteParserState,
    tag: String,
    parts: Vec<Vec<StringContentPart>>,
) {
    ps.emit_ident(percent_symbol_for(tag));
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.breakable_of(
                BreakableDelims::for_array(),
                Box::new(|ps| {
                    let parts_length = parts.len();
                    for (idx, part) in parts.into_iter().enumerate() {
                        ps.emit_soft_indent();
                        format_inner_string(ps, part, StringType::Array);
                        if idx != parts_length - 1 {
                            ps.emit_soft_newline();
                        }
                    }
                    ps.emit_collapsing_newline();
                }),
            );
        }),
    );
}

pub fn format_array(ps: &mut dyn ConcreteParserState, array: Array) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.on_line((array.2).0);

    match array.1 {
        SimpleArrayOrPercentArray::SimpleArray(a) => {
            format_array_fast_path(ps, array.2.end_line(), a)
        }
        SimpleArrayOrPercentArray::LowerPercentArray(pa) => {
            ps.on_line((pa.2).0);
            format_percent_array(
                ps,
                pa.0,
                pa.1.into_iter()
                    .map(|v| vec![StringContentPart::TStringContent(v)])
                    .collect(),
            );
        }
        SimpleArrayOrPercentArray::UpperPercentArray(pa) => {
            ps.on_line((pa.2).0);
            format_percent_array(ps, pa.0, pa.1);
        }
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }

    ps.wind_dumping_comments_until_line((array.2).1);
}

pub fn format_array_fast_path(
    ps: &mut dyn ConcreteParserState,
    end_line: LineNumber,
    a: Option<ArgsAddStarOrExpressionListOrArgsForward>,
) {
    match a {
        None => {
            ps.emit_open_square_bracket();
            ps.emit_close_square_bracket();
        }
        Some(a) => {
            ps.breakable_of(
                BreakableDelims::for_array(),
                Box::new(|ps| {
                    format_list_like_thing(ps, a, Some(end_line), false);
                    ps.emit_collapsing_newline();
                }),
            );
        }
    }
}

pub fn format_list_like_thing(
    ps: &mut dyn ConcreteParserState,
    a: ArgsAddStarOrExpressionListOrArgsForward,
    end_line: Option<LineNumber>,
    single_line: bool,
) -> bool {
    match a {
        ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(aas) => {
            let left = aas.1;
            let star = aas.2;
            let right = aas.3;
            let mut emitted_args = format_list_like_thing(ps, *left, None, single_line);

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
                if emitted_args {
                    ps.emit_comma();
                    ps.emit_soft_newline();
                }
                ps.emit_soft_indent();
            }

            emitted_args = true;

            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    ps.emit_ident("*".to_string());
                    format_expression(ps, *star);

                    for expr in right {
                        match expr {
                            Expression::BareAssocHash(bah) => {
                                ps.emit_comma();
                                ps.emit_soft_newline();
                                format_assocs(
                                    ps,
                                    bah.1,
                                    SpecialCase::NoLeadingTrailingCollectionMarkers,
                                );
                            }
                            e => {
                                emit_intermediate_array_separator(ps, single_line);
                                format_expression(ps, e);
                            }
                        }
                    }
                    if let Some(end_line) = end_line {
                        ps.wind_dumping_comments_until_line(end_line);
                        ps.shift_comments();
                    }
                }),
            );

            emitted_args
        }
        ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el) => {
            format_list_like_thing_items(ps, el, end_line, single_line)
        }
        ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(_) => {
            ps.emit_ellipsis();
            false
        }
    }
}

pub fn emit_intermediate_array_separator(ps: &mut dyn ConcreteParserState, single_line: bool) {
    if single_line {
        ps.emit_comma_space();
    } else {
        ps.emit_comma();
        ps.emit_soft_newline();
        ps.emit_soft_indent();
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum StringType {
    Quoted,
    Heredoc,
    Array,
    Regexp,
}

pub fn format_inner_string(
    ps: &mut dyn ConcreteParserState,
    parts: Vec<StringContentPart>,
    tipe: StringType,
) {
    let mut peekable = parts.into_iter().peekable();
    while peekable.peek().is_some() {
        let part = peekable.next().expect("we peeked");
        match part {
            StringContentPart::TStringContent(t) => match tipe {
                StringType::Heredoc => {
                    let mut contents = t.1;

                    if peekable.peek().is_none() && contents.ends_with('\n') {
                        contents.pop();
                    }
                    ps.emit_string_content(contents);
                }
                _ => {
                    ps.on_line((t.2).0);
                    ps.emit_string_content(t.1);
                }
            },
            StringContentPart::StringEmbexpr(e) => ps.with_formatting_context(
                FormattingContext::StringEmbexpr,
                Box::new(|ps| {
                    ps.emit_string_content("#{".to_string());
                    // Embexpr must have at least one expression.
                    // If they have multiple, render them with an expression per line
                    // just like they are outside of embexprs.
                    if (e.1).len() == 1 {
                        ps.with_start_of_line(
                            false,
                            Box::new(|ps| {
                                format_expression(ps, (e.1).first().unwrap().to_owned());
                            }),
                        )
                    } else {
                        ps.with_start_of_line(
                            true,
                            Box::new(|ps| {
                                ps.new_block(Box::new(|ps| {
                                    ps.emit_newline();
                                    for expression in e.1 {
                                        format_expression(ps, expression);
                                    }
                                }));
                            }),
                        );
                    }
                    ps.emit_string_content("}".to_string());

                    let on_line_skip = tipe == StringType::Heredoc
                        && match peekable.peek() {
                            Some(StringContentPart::TStringContent(TStringContent(_, s, _))) => {
                                s.starts_with('\n')
                            }
                            _ => false,
                        };
                    if on_line_skip {
                        ps.render_heredocs(true)
                    }
                }),
            ),
            StringContentPart::StringDVar(dv) => {
                ps.emit_string_content("#{".to_string());
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        let expr = *(dv.1);
                        format_expression(ps, expr);
                    }),
                );
                ps.emit_string_content("}".to_string());
            }
        }
    }
}

pub fn format_heredoc_string_literal(
    ps: &mut dyn ConcreteParserState,
    hd: HeredocStringLiteral,
    parts: Vec<StringContentPart>,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let end_line = hd.2.end_line();
    ps.on_line(hd.2.start_line());

    ps.with_suppress_comments(
        true,
        Box::new(|ps| {
            let heredoc_type = (hd.1).0;
            let heredoc_symbol = (hd.1).1;
            ps.emit_heredoc_start(heredoc_type.clone(), heredoc_symbol.clone());

            ps.push_heredoc_content(
                heredoc_symbol,
                HeredocKind::from_string(heredoc_type),
                parts,
                end_line,
            );
        }),
    );

    if ps.at_start_of_line() && !ps.is_absorbing_indents() {
        ps.emit_newline();
    }
}

pub fn format_string_literal(ps: &mut dyn ConcreteParserState, sl: StringLiteral) {
    match sl {
        StringLiteral::Heredoc(_, hd, StringContent(_, parts)) => {
            format_heredoc_string_literal(ps, hd, parts)
        }
        StringLiteral::Normal(_, StringContent(_, parts), start_end) => {
            if ps.at_start_of_line() {
                ps.emit_indent();
            }

            ps.on_line(start_end.start_line());

            ps.emit_double_quote();
            format_inner_string(ps, parts, StringType::Quoted);
            ps.emit_double_quote();

            if ps.at_start_of_line() {
                ps.emit_newline();
            }
        }
    }
}

pub fn format_xstring_literal(ps: &mut dyn ConcreteParserState, xsl: XStringLiteral) {
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

pub fn format_const_path_field(ps: &mut dyn ConcreteParserState, cf: ConstPathField) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *cf.1);
            ps.emit_colon_colon();
            format_const(ps, cf.2);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_top_const_field(ps: &mut dyn ConcreteParserState, tcf: TopConstField) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_colon_colon();
            format_const(ps, tcf.1);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_field(ps: &mut dyn ConcreteParserState, vf: VarField) {
    let left = vf.1;
    format_var_ref_type(ps, left);
}

pub fn format_aref_field(ps: &mut dyn ConcreteParserState, af: ArefField) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let end_line = (af.3).0;

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *af.1);
            let aab = af.2;
            match aab.2 {
                ToProcExpr::Present(_) => {
                    panic!("got a to_proc in an aref_field, should be impossible");
                }
                ToProcExpr::NotPresent(_) => {
                    format_array_fast_path(
                        ps,
                        end_line,
                        Some((aab.1).into_args_add_star_or_expression_list()),
                    );
                }
            }
        }),
    );

    ps.wind_dumping_comments_until_line(end_line);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_field(ps: &mut dyn ConcreteParserState, f: Field) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *f.1);
            format_dot(ps, f.2);
            match f.3 {
                IdentOrConst::Const(c) => format_const(ps, c),
                IdentOrConst::Ident(i) => format_ident(ps, i),
            }
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_assignable(ps: &mut dyn ConcreteParserState, v: Assignable) {
    match v {
        Assignable::VarField(vf) => {
            bind_var_field(ps, &vf);
            format_var_field(ps, vf);
        }
        Assignable::ConstPathField(cf) => {
            format_const_path_field(ps, cf);
        }
        Assignable::RestParam(rp) => {
            format_rest_param(
                ps,
                Some(RestParamOr0OrExcessedCommaOrArgsForward::RestParam(rp)),
                SpecialCase::RestParamOutsideOfParamDef,
            );
        }
        Assignable::TopConstField(tcf) => {
            format_top_const_field(ps, tcf);
        }
        Assignable::ArefField(af) => {
            format_aref_field(ps, af);
        }
        Assignable::Field(field) => {
            format_field(ps, field);
        }
        Assignable::MLhs(mlhs) => {
            format_mlhs(ps, mlhs);
        }
        Assignable::Ident(ident) => {
            bind_ident(ps, &ident);
            format_ident(ps, ident);
        }
    }
}

pub fn format_assign(ps: &mut dyn ConcreteParserState, assign: Assign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_assignable(ps, assign.1);
            let right = assign.2;

            ps.emit_space();
            ps.emit_op("=".to_string());
            ps.emit_space();

            ps.with_formatting_context(
                FormattingContext::Assign,
                Box::new(|ps| match right {
                    ExpressionOrMRHSNewFromArgs::Expression(e) => format_expression(ps, *e),
                    ExpressionOrMRHSNewFromArgs::MRHSNewFromArgs(m) => {
                        format_mrhs_new_from_args(ps, m)
                    }
                }),
            );
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_massign(ps: &mut dyn ConcreteParserState, massign: MAssign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            match massign.1 {
                AssignableListOrMLhs::AssignableList(al) => {
                    let length = al.len();
                    for (idx, v) in al.into_iter().enumerate() {
                        format_assignable(ps, v);
                        let last = idx == length - 1;
                        if !last {
                            ps.emit_comma_space();
                        }
                        if length == 1 {
                            ps.emit_comma();
                        }
                    }
                }
                AssignableListOrMLhs::MLhs(mlhs) => format_mlhs(ps, mlhs),
            }
            ps.emit_space();
            ps.emit_ident("=".to_string());
            ps.emit_space();
            match massign.2 {
                MRHSOrArray::MRHS(mrhs) => {
                    format_mrhs(ps, Some(mrhs));
                }
                MRHSOrArray::Array(array) => {
                    format_array(ps, array);
                }
            }
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_var_ref_type(ps: &mut dyn ConcreteParserState, vr: VarRefType) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    match vr {
        VarRefType::CVar(c) => handle_string_and_linecol(ps, c.1, c.2),
        VarRefType::GVar(g) => handle_string_and_linecol(ps, g.1, g.2),
        VarRefType::IVar(i) => handle_string_and_linecol(ps, i.1, i.2),
        VarRefType::Ident(i) => handle_string_and_linecol(ps, i.1, i.2),
        VarRefType::Const(c) => handle_string_and_linecol(ps, c.1, c.2),
        VarRefType::Kw(kw) => handle_string_and_linecol(ps, kw.1, kw.2),
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn handle_string_and_linecol(ps: &mut dyn ConcreteParserState, ident: String, lc: LineCol) {
    ps.on_line(lc.0);
    ps.emit_ident(ident);
}

pub fn format_var_ref(ps: &mut dyn ConcreteParserState, vr: VarRef) {
    format_var_ref_type(ps, vr.1);
}

pub fn format_const_path_ref(ps: &mut dyn ConcreteParserState, cpr: ConstPathRef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *cpr.1);
            ps.emit_colon_colon();
            format_const(ps, cpr.2);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_top_const_ref(ps: &mut dyn ConcreteParserState, tcr: TopConstRef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_colon_colon();
            format_const(ps, tcr.1);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_defined(ps: &mut dyn ConcreteParserState, defined: Defined) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_ident("defined?".to_string());
            ps.emit_open_paren();
            format_expression(ps, *defined.1);
            ps.emit_close_paren();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_rescue_mod(ps: &mut dyn ConcreteParserState, rescue_mod: RescueMod) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *rescue_mod.1);
            ps.emit_space();
            ps.emit_rescue();
            ps.emit_space();
            format_expression(ps, *rescue_mod.2);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_mrhs_new_from_args(ps: &mut dyn ConcreteParserState, mnfa: MRHSNewFromArgs) {
    format_list_like_thing(ps, mnfa.1, None, true);

    if let Some(expr) = mnfa.2 {
        ps.emit_comma_space();
        format_expression(ps, *expr);
    }
}

pub fn format_mrhs_add_star(ps: &mut dyn ConcreteParserState, mrhs: MRHSAddStar) {
    let first = mrhs.1;
    let second = mrhs.2;
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            match first {
                MRHSNewFromArgsOrEmpty::Empty(e) => {
                    if !e.is_empty() {
                        panic!("this should be impossible, got non-empty mrhs empty");
                    }
                }
                MRHSNewFromArgsOrEmpty::MRHSNewFromArgs(mnfa) => {
                    format_mrhs_new_from_args(ps, mnfa);
                    ps.emit_comma_space();
                }
            }
            ps.emit_ident("*".to_string());
            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    format_expression(ps, *second);
                }),
            );
        }),
    );
}

pub fn format_next(ps: &mut dyn ConcreteParserState, next: Next) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.on_line((next.2).0);
            ps.emit_ident("next".to_string());
            match next.1 {
                ArgsAddBlockOrExpressionList::ExpressionList(e) => {
                    if !e.is_empty() {
                        ps.emit_space();
                        format_list_like_thing_items(ps, e, Some((next.2).1), true);
                    }
                }
                ArgsAddBlockOrExpressionList::ArgsAddBlock(aab) => match aab.2 {
                    ToProcExpr::Present(_) => {
                        panic!("got a block in a next, should be impossible");
                    }
                    ToProcExpr::NotPresent(_) => {
                        ps.emit_space();
                        format_list_like_thing(
                            ps,
                            (aab.1).into_args_add_star_or_expression_list(),
                            Some((next.2).1),
                            true,
                        );
                    }
                },
            }
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_unary(ps: &mut dyn ConcreteParserState, unary: Unary) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            match unary.1 {
                UnaryType::Not => {
                    ps.emit_ident("not".to_string());
                    ps.emit_space();
                }
                UnaryType::Positive => {
                    ps.emit_ident("+".to_string());
                }
                UnaryType::Negative => {
                    ps.emit_ident("-".to_string());
                }
                UnaryType::BooleanNot => {
                    ps.emit_ident("!".to_string());
                }
                UnaryType::BitwiseNot => {
                    ps.emit_ident("~".to_string());
                }
            }

            format_expression(ps, *unary.2);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_string_concat(ps: &mut dyn ConcreteParserState, sc: StringConcat) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    ps.with_absorbing_indent_block(Box::new(|ps| {
        let nested = sc.1;
        let sl = sc.2;

        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                match nested {
                    StringConcatOrStringLiteral::StringConcat(sc) => format_string_concat(ps, *sc),
                    StringConcatOrStringLiteral::StringLiteral(sl) => format_string_literal(ps, sl),
                }

                ps.emit_space();
                ps.emit_slash();
                ps.emit_newline();

                ps.emit_indent();
                format_string_literal(ps, sl);
            }),
        );
    }));
    if ps.at_start_of_line() && !ps.is_absorbing_indents() {
        ps.emit_newline();
    }
}

pub fn format_dyna_symbol(ps: &mut dyn ConcreteParserState, ds: DynaSymbol) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident(":".to_string());
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_string_literal(ps, ds.to_string_literal());
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_undef(ps: &mut dyn ConcreteParserState, undef: Undef) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident("undef ".to_string());
    let length = undef.1.len();
    for (idx, literal) in undef.1.into_iter().enumerate() {
        ps.with_start_of_line(
            false,
            Box::new(|ps| format_symbol_literal_or_dyna_symbol(ps, literal)),
        );
        if idx != length - 1 {
            ps.emit_comma_space();
        }
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_defs(ps: &mut dyn ConcreteParserState, defs: Defs) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let singleton = defs.1;
    let ident_or_kw = defs.3;
    let paren_or_params = defs.4;
    let bodystmt = defs.5;
    let end_line = (defs.6).1;

    ps.emit_def_keyword();
    ps.emit_space();

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            match singleton {
                Singleton::VarRef(vr) => {
                    format_var_ref(ps, vr);
                }
                Singleton::Paren(pe) => {
                    format_paren(ps, pe);
                }
                Singleton::VCall(vc) => {
                    format_method_call(ps, vc.to_method_call());
                }
            }

            ps.emit_dot();
            let (ident, linecol) = ident_or_kw.to_def_parts();
            handle_string_and_linecol(ps, ident, linecol);
            format_paren_or_params(ps, paren_or_params);
        }),
    );

    ps.with_formatting_context(
        FormattingContext::Def,
        Box::new(|ps| {
            ps.new_block(Box::new(|ps| {
                ps.emit_newline();
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        format_bodystmt(ps, bodystmt, end_line);
                    }),
                );
            }));
        }),
    );

    ps.wind_dumping_comments_until_line(end_line);
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_paren_or_params(ps: &mut dyn ConcreteParserState, pp: ParenOrParams) {
    let maybe_closing_paren_line = match &pp {
        ParenOrParams::Paren(p) => Some(p.2.end_line()),
        _ => None,
    };
    let params = match pp {
        ParenOrParams::Paren(p) => p.1,
        ParenOrParams::Params(p) => p,
    };

    format_params(ps, params, BreakableDelims::for_method_call());

    if let Some(end_line) = maybe_closing_paren_line {
        ps.wind_dumping_comments_until_line(end_line)
    }
}

// Modules and classes bodies should be treated the same,
// the only real difference is in the module/class name and inheritance
fn format_constant_body(ps: &mut dyn ConcreteParserState, bodystmt: Box<BodyStmt>, end_line: u64) {
    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                ps.with_formatting_context(
                    FormattingContext::ClassOrModule,
                    Box::new(|ps| {
                        ps.emit_newline();
                        format_bodystmt(ps, bodystmt, end_line);
                    }),
                );
            }),
        );
    }));

    ps.on_line(end_line);
    ps.emit_end();
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_class(ps: &mut dyn ConcreteParserState, class: Class) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let class_name = class.1;
    let inherit = class.2;
    let bodystmt = class.3;
    let end_line = (class.4).1;

    ps.emit_class_keyword();
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_space();

            match class_name {
                ConstPathRefOrConstRefOrTopConstRef::ConstPathRef(cpr) => {
                    format_const_path_ref(ps, cpr);
                }
                ConstPathRefOrConstRefOrTopConstRef::ConstRef(cr) => {
                    handle_string_and_linecol(ps, (cr.1).1, (cr.1).2);
                }
                ConstPathRefOrConstRefOrTopConstRef::TopConstRef(tcr) => {
                    format_top_const_ref(ps, tcr)
                }
            }

            if let Some(inherit_expression) = inherit {
                ps.emit_ident(" < ".to_string());
                format_expression(ps, *inherit_expression);
            }
        }),
    );

    format_constant_body(ps, bodystmt, end_line);
}

pub fn format_module(ps: &mut dyn ConcreteParserState, module: Module) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let module_name = module.1;
    let bodystmt = module.2;
    let end_line = (module.3).1;

    ps.emit_module_keyword();
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_space();

            match module_name {
                ConstPathRefOrConstRefOrTopConstRef::ConstPathRef(cpr) => {
                    format_const_path_ref(ps, cpr);
                }
                ConstPathRefOrConstRefOrTopConstRef::ConstRef(cr) => {
                    handle_string_and_linecol(ps, (cr.1).1, (cr.1).2);
                }
                ConstPathRefOrConstRefOrTopConstRef::TopConstRef(tcr) => {
                    format_top_const_ref(ps, tcr)
                }
            }
        }),
    );

    format_constant_body(ps, bodystmt, end_line);
}

pub fn format_conditional(
    ps: &mut dyn ConcreteParserState,
    cond_expr: Expression,
    body: Vec<Expression>,
    kw: String,
    tail: Option<ElsifOrElse>,
    start_end: Option<StartEnd>,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    if let Some(StartEnd(start_line, ..)) = start_end {
        ps.on_line(start_line);
    }
    ps.emit_conditional_keyword(kw);
    ps.emit_space();
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.new_block(Box::new(|ps| {
                format_expression(ps, cond_expr);
            }))
        }),
    );

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.new_block(Box::new(|ps| {
                ps.emit_newline();
                for expr in body.into_iter() {
                    format_expression(ps, expr);
                }
            }));
        }),
    );
    ps.with_start_of_line(
        true,
        Box::new(|ps| match tail {
            None => {}
            Some(ElsifOrElse::Elsif(elsif)) => {
                format_conditional(
                    ps,
                    *elsif.1,
                    elsif.2,
                    "elsif".to_string(),
                    (elsif.3).map(|v| *v),
                    Some(elsif.4),
                );
            }
            Some(ElsifOrElse::Else(els)) => {
                ps.emit_indent();
                ps.emit_else();
                ps.new_block(Box::new(|ps| {
                    ps.on_line(els.2.start_line());
                    ps.emit_newline();
                }));
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        ps.new_block(Box::new(|ps| {
                            for expr in els.1 {
                                format_expression(ps, expr);
                            }
                            ps.wind_dumping_comments_until_line(els.2.end_line());
                        }));
                    }),
                );
            }
        }),
    );

    if let Some(StartEnd(_, end_line)) = start_end {
        ps.wind_dumping_comments_until_line(end_line);
    }
}

pub fn format_if(ps: &mut dyn ConcreteParserState, ifs: If) {
    let vifs = ifs.clone();
    format_conditional(ps, *ifs.1, ifs.2, "if".to_string(), ifs.3, Some(ifs.4));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.wind_dumping_comments_until_line(vifs.4 .1);
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_unless(ps: &mut dyn ConcreteParserState, unless: Unless) {
    format_conditional(
        ps,
        *unless.1,
        unless.2,
        "unless".to_string(),
        (unless.3).map(ElsifOrElse::Else),
        Some(unless.4),
    );
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_binary(ps: &mut dyn ConcreteParserState, binary: Binary, must_be_multiline: bool) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let format_func = |ps: &mut dyn ConcreteParserState, render_multiline: bool| {
        ps.with_formatting_context(
            FormattingContext::Binary,
            Box::new(|ps| {
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        if let Expression::Binary(b) = *binary.1 {
                            format_binary(ps, b, render_multiline);
                        } else {
                            format_expression(ps, *binary.1);
                        }

                        let operator = binary.2;
                        let comparison_operators =
                            vec![">", ">=", "===", "==", "<", "<=", "<=>", "!="];
                        let is_not_comparison =
                            !comparison_operators.iter().any(|o| o == &operator);

                        ps.emit_space();
                        ps.emit_ident(operator);

                        let next_expr = *binary.3;

                        // In some cases, previous expressions changed the space
                        // count but haven't reset it, so we force a reset here in
                        // case we shift comments during the _next_ expression
                        ps.reset_space_count();

                        if render_multiline && is_not_comparison {
                            ps.new_block(Box::new(|ps| {
                                ps.emit_newline();
                                ps.emit_indent();

                                format_expression(ps, next_expr);
                            }));
                        } else {
                            ps.emit_space();
                            format_expression(ps, next_expr);
                        }
                    }),
                );
            }),
        );
    };

    let is_multiline = must_be_multiline
        || ps.will_render_beyond_max_line_length(Box::new(|ps| format_func.clone()(ps, false)));
    format_func(ps, is_multiline);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_float(ps: &mut dyn ConcreteParserState, float: Float) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, float.1, float.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_aref(ps: &mut dyn ConcreteParserState, aref: Aref) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let end_line = (aref.3).0;

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *aref.1);
            match aref.2 {
                None => {
                    ps.emit_open_square_bracket();
                    ps.emit_close_square_bracket();
                }
                Some(arg_node) => {
                    let args_list = normalize_args(arg_node);
                    format_array_fast_path(ps, end_line, Some(args_list));
                }
            }
        }),
    );

    ps.wind_dumping_comments_until_line(end_line);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_char(ps: &mut dyn ConcreteParserState, c: Char) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_double_quote();
    ps.on_line((c.2).0);
    ps.emit_ident(c.1[1..].to_string());
    ps.emit_double_quote();

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_hash(ps: &mut dyn ConcreteParserState, hash: Hash) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    let StartEnd(start_line, end_line) = hash.2;
    ps.on_line(start_line);

    match hash.1 {
        None => {
            let is_multiline = start_line != end_line;
            let has_comments = ps.has_comments_in_line(start_line, end_line);

            if is_multiline && has_comments {
                // Since we already know this is multiline, we can just use
                // a breakable and know that it will always be the multiline form
                // instead of manually inserting all of the newlines/indents for
                // a multiline hash
                ps.breakable_of(
                    BreakableDelims::for_hash(),
                    Box::new(|ps| {
                        ps.wind_dumping_comments_until_line(end_line);
                    }),
                );
            } else {
                ps.emit_ident("{}".to_string());
                ps.wind_dumping_comments_until_line(end_line);
            }
        }
        Some(assoc_list_from_args) => {
            ps.breakable_of(
                BreakableDelims::for_hash(),
                Box::new(|ps| {
                    format_assocs(ps, assoc_list_from_args.1, SpecialCase::NoSpecialCase);
                    ps.wind_dumping_comments_until_line(end_line);
                }),
            );
        }
    };

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_regexp_literal(ps: &mut dyn ConcreteParserState, regexp: RegexpLiteral) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let parts = regexp.1;
    let start_delimiter = (regexp.2).3;
    let end_delimiter = (regexp.2).1;

    ps.emit_ident(start_delimiter);
    format_inner_string(ps, parts, StringType::Regexp);
    handle_string_and_linecol(ps, end_delimiter, (regexp.2).2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_backref(ps: &mut dyn ConcreteParserState, backref: Backref) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, backref.1, backref.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn can_elide_parens_for_rspec_dsl_call(cc: &[CallChainElement]) -> bool {
    if let Some(CallChainElement::Block(Block::BraceBlock(_))) = cc.last() {
        return false;
    };

    let is_bare_it_or_describe = match cc.get(0) {
        Some(CallChainElement::IdentOrOpOrKeywordOrConst(IdentOrOpOrKeywordOrConst::Ident(
            Ident(_, ident, _),
        ))) => ident == "it" || ident == "describe",
        _ => false,
    };

    let is_rspec_describe = match (cc.get(0), cc.get(2)) {
        (
            Some(CallChainElement::VarRef(VarRef(_, VarRefType::Const(Const(_, c, _))))),
            Some(CallChainElement::IdentOrOpOrKeywordOrConst(IdentOrOpOrKeywordOrConst::Ident(
                Ident(_, i, _),
            ))),
        ) => c == "RSpec" && i == "describe",
        _ => false,
    };

    is_bare_it_or_describe || is_rspec_describe
}

/// Returns `true` if the call chain is indented, `false` if not
fn format_call_chain(
    ps: &mut dyn ConcreteParserState,
    cc: Vec<CallChainElement>,
    should_multiline_call_chain: bool,
) -> bool {
    if cc.is_empty() {
        return false;
    }

    format_call_chain_elements(ps, cc, should_multiline_call_chain);

    ps.emit_after_call_chain();
    should_multiline_call_chain
}

fn format_call_chain_elements(
    ps: &mut dyn ConcreteParserState,
    cc: Vec<CallChainElement>,
    render_multiline_chain: bool,
) {
    let elide_parens = can_elide_parens_for_rspec_dsl_call(&cc);
    let mut has_indented = false;
    // When set, force all `CallChainElement::ArgsAddStarOrExpressionListOrArgsForward`
    // to use parens, even when empty. This handles cases like `super()` where parens matter
    let mut next_args_list_must_use_parens = false;
    for cc_elem in cc {
        let mut element_is_super_keyword = false;

        match cc_elem {
            CallChainElement::Paren(p) => format_paren(ps, p),
            CallChainElement::IdentOrOpOrKeywordOrConst(i) => {
                let ident = i.into_ident();
                element_is_super_keyword = ident.1 == "super";

                format_ident(ps, ident)
            }
            CallChainElement::Block(b) => {
                ps.emit_space();
                format_block(ps, b)
            }
            CallChainElement::VarRef(vr) => format_var_ref(ps, vr),
            CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(aas, start_end) => {
                if !aas.is_empty() || next_args_list_must_use_parens {
                    let delims = if elide_parens {
                        BreakableDelims::for_kw()
                    } else {
                        BreakableDelims::for_method_call()
                    };

                    let end_line = start_end.map(|se| se.1);

                    ps.breakable_of(
                        delims,
                        Box::new(|ps| {
                            format_list_like_thing(ps, aas, end_line, false);
                        }),
                    );
                    if let Some(end_line) = end_line {
                        // If we're rendering a single-line chain, force a reset so
                        // that comments end up at the current indentation level
                        if !render_multiline_chain {
                            ps.reset_space_count();
                        }
                        ps.wind_dumping_comments_until_line(end_line);
                    }
                }
            }
            CallChainElement::DotTypeOrOp(d) => {
                if render_multiline_chain && !has_indented {
                    ps.start_indent();
                    has_indented = true;
                }
                if render_multiline_chain {
                    ps.emit_newline();
                    ps.emit_indent();
                }
                format_dot(ps, d);
            }
            CallChainElement::Expression(e) => format_expression(ps, *e),
        }
        next_args_list_must_use_parens = element_is_super_keyword;
    }

    if has_indented {
        ps.end_indent();
    }
}

/// Checks whether a call chain both starts with a heredoc expression
/// *and* contains a call chain element with a breakable.
///
/// In practice, this generally means something like the call chain having something
/// like a method call with args or a block, e.g.
///
/// ```ruby
/// # `|line|` here is the breakable
/// <<~FOO.lines.map { |line| p(line) }
/// FOO
/// ```
///
/// Breakables don't play very nicely with heredoc rendering in call chains,
/// and it would likely be a pretty hefty refactor to properly support this.
fn is_heredoc_call_chain_with_breakables(cc_elements: &[CallChainElement]) -> bool {
    if let Some(CallChainElement::Expression(expr)) = cc_elements.first() {
        if let Expression::StringLiteral(string_literal) = &**expr {
            if matches!(string_literal, StringLiteral::Heredoc(..)) {
                let contains_breakables = cc_elements.iter().any(|cc_elem| match cc_elem {
                    CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(
                        ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(list),
                        ..,
                    ) => !list.is_empty(),
                    CallChainElement::Block(..) => true,
                    _ => false,
                });
                return contains_breakables;
            }
        }
    }

    false
}

/// When to multiling call chains generally relies on a few broadly-applicable rules, but in practice
/// it has *many* special-cases, because multilining them ends up colliding with other language features in awkward ways.
///
/// ## High-level rules
///
/// The two main rules that govern whether or not to multiline a method chain is to split across multiple lines if
/// (1) the whole chain exceeds the maximum line length or
/// (2) the chain contains blocks that are split across multiple lines
///
/// That said, both of these have some *very large* asterisks, since there are a lot of contexts in which these
/// have special cases for various reasons (see below).
///
/// ## Special conditions
///
/// This is a best-effort listing for the exceptional cases and their rationales
///
/// * String embedded expressions
///   * We currently _never_ multiline in string embexprs, mostly because multilining makes it more difficult
///     to grok the final whitespace of the string.
/// * Chains consisting only of var/const refs
///   * It's pretty common to have something of the shape of `Class.method(args)`, and even if it exceeds the max line length,
///     multilining generally makes this only more confusing at first glance.
/// * Chains starting with heredocs
///   * With the current way breakables work, heredocs at the beginning of call chains will will often render really awkwardly
///     (for example, in the middle of the argument parameters), so we default to multilining if the chain has both a heredoc
///     and breakables to work around this.
/// * Long expressions/blocks at the beggining/end
///   * It's not uncommon to have call chains that start with extremely long expressions (e.g. a long array literal) but
///     but have very little following it; similarly, it's fairly common to have a short expression but a *very* long
///     block call following it (e.g. an `each` or `map` call with most of the logic in it). In these cases, we ignore
///     these long CallChainElements when calculating the length of the final expression to make some common idioms render nicely.
fn should_multiline_call_chain(ps: &mut dyn ConcreteParserState, method_call: &MethodCall) -> bool {
    // Never multiline if we're in an embedded expression
    if ps.current_formatting_context() == FormattingContext::StringEmbexpr {
        return false;
    }

    let MethodCall(_, mut call_chain_to_check, ident, _, args, start_end) = method_call.clone();

    // Add the original method as a call chain element purely for the sake of determining multiling
    call_chain_to_check.append(&mut vec![
        CallChainElement::IdentOrOpOrKeywordOrConst(ident),
        CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(args, start_end),
    ]);

    // Ignore chains that are basically only method calls, e.g.
    // ````ruby
    // Thing.foo(args)
    // Thing.foo(args) { block! }
    // ```
    // These should always stay inline
    match call_chain_to_check.as_slice() {
        [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
        | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
        | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
        | [CallChainElement::VarRef(..) | CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] =>
        {
            return false;
        }
        [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..)]
        | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..)]
        | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::Block(..)]
        | [CallChainElement::Expression(maybe_const_ref), CallChainElement::DotTypeOrOp(..), CallChainElement::IdentOrOpOrKeywordOrConst(..), CallChainElement::ArgsAddStarOrExpressionListOrArgsForward(..), CallChainElement::Block(..)] => {
            if matches!(maybe_const_ref.as_ref(), Expression::ConstPathRef(..)) {
                return false;
            }
        }
        _ => {}
    }

    if is_heredoc_call_chain_with_breakables(&call_chain_to_check) {
        return true;
    }

    // If the first item in the chain is a multiline expression (like a hash or array),
    // ignore it when checking line length
    if let Some(CallChainElement::Expression(expr)) = call_chain_to_check.first() {
        let is_multiline_expression = ps.will_render_as_multiline(Box::new(|ps| {
            format_expression(ps, expr.as_ref().clone());
        }));

        if is_multiline_expression {
            call_chain_to_check.remove(0);
        }
    }

    // We don't always want to multiline blocks if their only usage
    // is at the end of a chain, since it's common to have chains
    // that end with long blocks, but those blocks don't mean we should
    // multiline the rest of the chain.
    //
    // example:
    // ```
    // items.get_all.each do
    // end
    // ```
    if let Some(CallChainElement::Block(..)) = call_chain_to_check.last() {
        call_chain_to_check.pop();
    }

    let chain_is_too_long = ps.will_render_beyond_max_line_length(Box::new(|ps| {
        format_call_chain_elements(ps, call_chain_to_check.clone(), false);
    }));
    if chain_is_too_long {
        return true;
    }

    let chain_blocks_are_multilined = call_chain_to_check
        .iter()
        .filter_map(|elem| match elem {
            CallChainElement::Block(block) => Some(block.clone()),
            _ => None,
        })
        .any(|block| {
            ps.will_render_as_multiline(Box::new(|ps| {
                format_block(ps, block);
            }))
        });

    chain_blocks_are_multilined
}

pub fn format_block(ps: &mut dyn ConcreteParserState, b: Block) {
    match b {
        Block::BraceBlock(bb) => format_brace_block(ps, bb),
        Block::DoBlock(db) => format_do_block(ps, db),
    }
}

pub fn format_method_add_block(ps: &mut dyn ConcreteParserState, mab: MethodAddBlock) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let should_multiline_chain = should_multiline_call_chain(ps, &mab.clone().to_method_call());
    let mut chain = (mab.1).into_call_chain();
    chain.push(CallChainElement::Block(mab.2));

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_call_chain(ps, chain, should_multiline_chain);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn is_empty_bodystmt(bodystmt: &Vec<Expression>) -> bool {
    bodystmt.len() == 1 && matches!(bodystmt[0], Expression::VoidStmt(..))
}

pub fn format_brace_block(ps: &mut dyn ConcreteParserState, brace_block: BraceBlock) {
    let bv = brace_block.1;
    let body = brace_block.2;
    let StartEnd(start_line, end_line) = brace_block.3;

    ps.on_line(start_line);

    let new_body = body.clone();

    let will_render_multiline =
        brace_contents_will_render_multiline(ps, start_line, end_line, new_body);

    ps.emit_ident("{".to_string());

    if let Some(bv) = bv {
        format_blockvar(ps, bv);
    }

    render_block_contents(ps, will_render_multiline, body, end_line);
    ps.emit_ident("}".to_string());
}

fn render_block_contents(
    ps: &mut dyn ConcreteParserState,
    will_render_multiline: bool,
    body: Vec<Expression>,
    end_line: u64,
) {
    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            will_render_multiline,
            Box::new(|ps| {
                if will_render_multiline {
                    ps.emit_newline();
                } else {
                    ps.emit_space();
                }
                for expr in body.into_iter() {
                    format_expression(ps, expr);
                }
            }),
        );
    }));
    if will_render_multiline {
        ps.emit_indent();
    } else {
        ps.emit_space();
    }
    ps.wind_dumping_comments_until_line(end_line);
}

fn brace_contents_will_render_multiline(
    ps: &mut dyn ConcreteParserState,
    start_line: u64,
    end_line: u64,
    body: Vec<Expression>,
) -> bool {
    let has_comments = ps.has_comments_in_line(start_line, end_line);
    let is_multiline = ps.will_render_as_multiline(Box::new(|next_ps| {
        next_ps.new_block(Box::new(|next_ps| {
            next_ps.with_start_of_line(
                true,
                Box::new(|next_ps| {
                    next_ps.with_formatting_context(
                        FormattingContext::CurlyBlock,
                        Box::new(|next_ps| {
                            for expr in body.into_iter() {
                                format_expression(next_ps, expr);
                            }
                        }),
                    );
                }),
            );
        }));
    }));

    is_multiline || has_comments
}

pub fn format_do_block(ps: &mut dyn ConcreteParserState, do_block: DoBlock) {
    ps.emit_do_keyword();

    let bv = do_block.1;
    let body = do_block.2;
    let end_line = do_block.3.end_line();

    if let Some(bv) = bv {
        format_blockvar(ps, bv)
    }

    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                ps.emit_newline();
                format_bodystmt(ps, body, end_line);
            }),
        );
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.wind_dumping_comments_until_line(end_line);
            ps.emit_end()
        }),
    );
}

pub fn format_keyword(
    ps: &mut dyn ConcreteParserState,
    args: ParenOrArgsAddBlock,
    kw: String,
    start_end: StartEnd,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_keyword(kw);
    let yield_args = match args {
        ParenOrArgsAddBlock::YieldParen(p) => {
            ps.emit_space();
            let arg = *p.1;
            match arg {
                ArgNode::ArgsAddBlock(aab) => aab,
                _ => panic!("should not have anything other than aab in yield"),
            }
        }
        ParenOrArgsAddBlock::ArgsAddBlock(aab) => {
            ps.emit_space();
            aab
        }
        ParenOrArgsAddBlock::Empty(v) => {
            if !v.is_empty() {
                panic!("got non empty empty in break/yield");
            };
            ArgsAddBlock(
                args_add_block_tag,
                ArgsAddBlockInner::ArgsAddStarOrExpressionListOrArgsForward(
                    ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(vec![]),
                ),
                ToProcExpr::NotPresent(false),
                start_end.clone(),
            )
        }
    };
    ps.on_line(start_end.0);

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_list_like_thing(
                ps,
                (yield_args.1).into_args_add_star_or_expression_list(),
                Some(start_end.1),
                true,
            );
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_while(
    ps: &mut dyn ConcreteParserState,
    conditional: Box<Expression>,
    exprs: Vec<Expression>,
    kw: String,
    start_end: StartEnd,
) {
    format_conditional(ps, *conditional, exprs, kw, None, Some(start_end));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

/// Some mod statements should _always_ be inlined, specifically `while` and `until` blocks,
/// since multilining them has different semantics. For example:
///
/// ```ruby
/// # Will always run at least once
/// begin
///   puts "thing"
/// end while should_run?
///
/// # Won't always run the block
/// while should_run?
///   begin
///     puts "thing"
///   end
/// end
/// ```
pub fn format_inline_mod(
    ps: &mut dyn ConcreteParserState,
    conditional: Box<Expression>,
    body: Box<Expression>,
    name: String,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_expression(ps, *body);

            ps.emit_mod_keyword(format!(" {} ", name));
            format_expression(ps, *conditional);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

/// Some mod statements can be safely converted to their equivalent
/// multiline forms, specifically `if` and `unless` mod statements.
pub fn format_multilinable_mod(
    ps: &mut dyn ConcreteParserState,
    conditional: Box<Expression>,
    body: Box<Expression>,
    name: String,
) {
    let new_body = body.clone();

    let is_multiline = ps.will_render_as_multiline(Box::new(|next_ps| {
        let exprs = match *new_body {
            Expression::Paren(p) => match p.1 {
                ParenExpressionOrExpressions::Expressions(exprs) => exprs,
                ParenExpressionOrExpressions::Expression(e) => vec![*e],
            },
            e => vec![e],
        };

        for expr in exprs {
            format_expression(next_ps, expr);
        }
    }));

    if is_multiline {
        let exps = match *body {
            Expression::Paren(ParenExpr(_, exps, _)) => match exps {
                ParenExpressionOrExpressions::Expressions(exprs) => exprs,
                ParenExpressionOrExpressions::Expression(e) => vec![*e],
            },
            x => vec![x],
        };
        format_conditional(ps, *conditional, exps, name, None, None);

        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                ps.emit_end();
            }),
        );

        if ps.at_start_of_line() {
            ps.emit_newline();
        }
    } else {
        format_inline_mod(ps, conditional, body, name)
    }
}

pub fn format_when_or_else(ps: &mut dyn ConcreteParserState, tail: WhenOrElse) {
    match tail {
        WhenOrElse::When(when) => {
            let conditionals = when.1;
            let body = when.2;
            let tail = when.3;
            let start_end = when.4;
            ps.on_line(start_end.0);
            ps.emit_indent();
            ps.emit_when_keyword();

            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    ps.new_block(Box::new(|ps| {
                        ps.inline_breakable_of(
                            BreakableDelims::for_when(),
                            Box::new(|ps| {
                                format_list_like_thing(ps, conditionals, None, false);
                            }),
                        );
                    }));
                }),
            );

            ps.new_block(Box::new(|ps| {
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        ps.emit_newline();
                        for expr in body {
                            format_expression(ps, expr);
                        }
                    }),
                );
            }));

            if let Some(tail) = tail {
                format_when_or_else(ps, *tail);
            }
        }
        WhenOrElse::Else(e) => {
            ps.emit_indent();
            ps.emit_else();

            ps.new_block(Box::new(|ps| {
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        ps.on_line(e.2.start_line());
                        ps.emit_newline();
                        for expr in e.1 {
                            format_expression(ps, expr);
                        }

                        ps.wind_dumping_comments_until_line(e.2.end_line());
                    }),
                );
            }));
        }
    }
}

pub fn format_case(ps: &mut dyn ConcreteParserState, case: Case) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    let end_line = case.3.end_line();
    ps.on_line((case.3).0);

    ps.emit_case_keyword();

    let case_expr = case.1;
    let tail = case.2;

    if let Some(e) = case_expr {
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                ps.emit_space();
                format_expression(ps, *e)
            }),
        );
    }

    ps.emit_newline();
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            format_when_or_else(ps, WhenOrElse::When(tail));
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.wind_dumping_comments_until_line(end_line);
        ps.emit_newline();
    }
    ps.on_line(case.3 .1);
}

pub fn format_retry(ps: &mut dyn ConcreteParserState, r: Retry) {
    format_keyword(
        ps,
        ParenOrArgsAddBlock::Empty(Vec::new()),
        "retry".to_string(),
        r.1,
    );
}

pub fn format_redo(ps: &mut dyn ConcreteParserState, r: Redo) {
    format_keyword(
        ps,
        ParenOrArgsAddBlock::Empty(Vec::new()),
        "redo".to_string(),
        r.1,
    );
}

pub fn format_sclass(ps: &mut dyn ConcreteParserState, sc: SClass) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let expr = sc.1;
    let body = sc.2;
    let end_line = sc.3.end_line();

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_keyword("class".into());
            ps.emit_space();
            ps.emit_ident("<<".to_string());
            ps.emit_space();
            format_expression(ps, *expr);
            ps.emit_newline();
            ps.new_block(Box::new(|ps| {
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        format_bodystmt(ps, body, end_line);
                    }),
                );
            }));
        }),
    );
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );

    ps.on_line(end_line);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_stabby_lambda(ps: &mut dyn ConcreteParserState, sl: StabbyLambda) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let StartEnd(start_line, end_line) = sl.3;
    ps.on_line(start_line);

    let params = sl.1;
    let body = sl.2;

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_keyword("->".to_string());
            if params.is_present() {
                ps.emit_space();
            }
            format_paren_or_params(ps, params);

            // Curly blocks are always represented as ExpressionLists (stmt_add nodes)
            // while do/end blocks are BodyStmt nodes
            match body {
                ExpressionListOrBodyStmt::ExpressionList(bud) => {
                    let b = bud;
                    let will_render_multiline =
                        brace_contents_will_render_multiline(ps, start_line, end_line, b.clone());
                    ps.emit_space();
                    ps.emit_ident("{".to_string());
                    render_block_contents(ps, will_render_multiline, b, end_line);
                    ps.emit_ident("}".to_string());
                }
                ExpressionListOrBodyStmt::BodyStmt(bs) => {
                    ps.emit_space();
                    ps.emit_do_keyword();
                    ps.emit_newline();
                    ps.new_block(Box::new(|ps| {
                        ps.with_start_of_line(
                            true,
                            Box::new(|ps| {
                                format_bodystmt(ps, bs, end_line);
                            }),
                        );
                    }));
                    ps.with_start_of_line(
                        true,
                        Box::new(|ps| {
                            ps.wind_dumping_comments_until_line(end_line);
                            ps.emit_end()
                        }),
                    );
                }
            }
        }),
    );

    ps.wind_dumping_comments_until_line(end_line);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_imaginary(ps: &mut dyn ConcreteParserState, imaginary: Imaginary) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, imaginary.1, imaginary.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_rational(ps: &mut dyn ConcreteParserState, rational: Rational) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_and_linecol(ps, rational.1, rational.2);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_for(ps: &mut dyn ConcreteParserState, forloop: For) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let variables = forloop.1;
    let collection = forloop.2;
    let body = forloop.3;

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_keyword("for".to_string());
            ps.emit_space();
            match variables {
                VarFieldOrVarFields::VarField(vf) => {
                    format_var_field(ps, vf);
                }
                VarFieldOrVarFields::VarFields(vfs) => {
                    let len = vfs.len();
                    for (idx, expr) in vfs.into_iter().enumerate() {
                        format_var_field(ps, expr);
                        if idx != len - 1 {
                            ps.emit_comma_space();
                        }
                    }
                }
            }

            ps.emit_space();
            ps.emit_keyword("in".to_string());
            ps.emit_space();
            format_expression(ps, *collection);
            ps.emit_newline();
            ps.new_block(Box::new(|ps| {
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        for expr in body.into_iter() {
                            format_expression(ps, expr);
                        }
                    }),
                );
            }));
        }),
    );
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_ifop(ps: &mut dyn ConcreteParserState, ifop: IfOp) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.with_formatting_context(
                FormattingContext::IfOp,
                Box::new(|ps| {
                    format_expression(ps, *ifop.1);
                    ps.emit_space();
                    ps.emit_keyword("?".to_string());
                    ps.emit_space();
                    format_expression(ps, *ifop.2);
                    ps.emit_space();
                    ps.emit_keyword(":".to_string());
                    ps.emit_space();
                    format_expression(ps, *ifop.3);
                }),
            );
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_return0(ps: &mut dyn ConcreteParserState, r: Return0) {
    format_keyword(
        ps,
        ParenOrArgsAddBlock::Empty(Vec::new()),
        "return".to_string(),
        r.1,
    );
}

pub fn format_opassign(ps: &mut dyn ConcreteParserState, opassign: OpAssign) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_assignable(ps, opassign.1);
            ps.emit_space();
            format_op(ps, opassign.2);
            ps.emit_space();
            format_expression(ps, *opassign.3);
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}
pub fn format_to_proc(ps: &mut dyn ConcreteParserState, e: Box<Expression>) {
    ps.emit_ident("&".to_string());
    ps.with_start_of_line(false, Box::new(|ps| format_expression(ps, *e)));
}

pub fn format_zsuper(ps: &mut dyn ConcreteParserState, start_end: StartEnd) {
    format_keyword(
        ps,
        ParenOrArgsAddBlock::Empty(Vec::new()),
        "super".to_string(),
        start_end,
    )
}

pub fn format_yield0(ps: &mut dyn ConcreteParserState, start_end: StartEnd) {
    format_keyword(
        ps,
        ParenOrArgsAddBlock::Empty(Vec::new()),
        "yield".to_string(),
        start_end,
    )
}

pub fn format_yield(ps: &mut dyn ConcreteParserState, y: Yield) {
    format_method_call(ps, y.to_method_call())
}

pub fn format_return(ps: &mut dyn ConcreteParserState, ret: Return) {
    let args = ret.1;
    let line = (ret.2).0;
    ps.on_line(line);

    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let args = normalize_args(args);
    ps.emit_keyword("return".to_string());

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if !args.is_empty() {
                match args {
                    ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(aas) => {
                        format_bare_return_args(
                            ps,
                            ArgsAddStarOrExpressionListOrArgsForward::ArgsAddStar(aas),
                        );
                    }
                    ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(af) => {
                        format_bare_return_args(
                            ps,
                            ArgsAddStarOrExpressionListOrArgsForward::ArgsForward(af),
                        );
                    }
                    ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(mut el) => {
                        if el.len() == 1 {
                            let element = el.remove(0);
                            match element {
                                Expression::Array(Array(array_tag, contents, linecol)) => {
                                    ps.emit_space();
                                    format_array(ps, Array(array_tag, contents, linecol));
                                }
                                elem => {
                                    ps.emit_space();
                                    format_expression(ps, elem);
                                }
                            }
                        } else {
                            format_bare_return_args(
                                ps,
                                ArgsAddStarOrExpressionListOrArgsForward::ExpressionList(el),
                            );
                        }
                    }
                }
            }
        }),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

pub fn format_bare_return_args(
    ps: &mut dyn ConcreteParserState,
    args: ArgsAddStarOrExpressionListOrArgsForward,
) {
    ps.breakable_of(
        BreakableDelims::for_return_kw(),
        Box::new(|ps| {
            ps.with_formatting_context(
                FormattingContext::ArgsList,
                Box::new(|ps| {
                    format_list_like_thing(ps, args, None, false);
                    ps.emit_collapsing_newline();
                }),
            );
        }),
    );
}

pub fn format_expression(ps: &mut dyn ConcreteParserState, expression: Expression) {
    let expression = normalize(expression);
    debug!("normalized expression: {:?}", expression);
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
        Expression::Unary(unary) => format_unary(ps, unary),
        Expression::StringConcat(sc) => format_string_concat(ps, sc),
        Expression::DynaSymbol(ds) => format_dyna_symbol(ps, ds),
        Expression::Undef(undef) => format_undef(ps, undef),
        Expression::Class(class) => format_class(ps, class),
        Expression::Defs(defs) => format_defs(ps, defs),
        Expression::If(ifs) => format_if(ps, ifs),
        Expression::Binary(binary) => format_binary(ps, binary, false),
        Expression::Float(float) => format_float(ps, float),
        Expression::Aref(aref) => format_aref(ps, aref),
        Expression::Char(c) => format_char(ps, c),
        Expression::Module(m) => format_module(ps, m),
        Expression::Hash(h) => format_hash(ps, h),
        Expression::RegexpLiteral(regexp) => format_regexp_literal(ps, regexp),
        Expression::Backref(backref) => format_backref(ps, backref),
        Expression::Yield(y) => format_yield(ps, y),
        Expression::Break(b) => format_keyword(ps, b.1, "break".to_string(), b.2),
        Expression::MethodAddBlock(mab) => format_method_add_block(ps, mab),
        Expression::While(w) => format_while(ps, w.1, w.2, "while".to_string(), w.3),
        Expression::Until(u) => format_while(ps, u.1, u.2, "until".to_string(), u.3),
        Expression::WhileMod(wm) => format_inline_mod(ps, wm.1, wm.2, "while".to_string()),
        Expression::UntilMod(um) => format_inline_mod(ps, um.1, um.2, "until".to_string()),
        Expression::IfMod(wm) => format_multilinable_mod(ps, wm.1, wm.2, "if".to_string()),
        Expression::UnlessMod(um) => format_multilinable_mod(ps, um.1, um.2, "unless".to_string()),
        Expression::Case(c) => format_case(ps, c),
        Expression::Retry(r) => format_retry(ps, r),
        Expression::Redo(r) => format_redo(ps, r),
        Expression::SClass(sc) => format_sclass(ps, sc),
        Expression::StabbyLambda(sl) => format_stabby_lambda(ps, sl),
        Expression::Rational(rational) => format_rational(ps, rational),
        Expression::Imaginary(imaginary) => format_imaginary(ps, imaginary),
        Expression::MLhs(mlhs) => format_mlhs(ps, mlhs),
        Expression::For(forloop) => format_for(ps, forloop),
        Expression::IfOp(ifop) => format_ifop(ps, ifop),
        Expression::Return0(r) => format_return0(ps, r),
        Expression::OpAssign(op) => format_opassign(ps, op),
        Expression::Unless(u) => format_unless(ps, u),
        Expression::ToProc(ToProc(_, e)) => format_to_proc(ps, e),
        Expression::ZSuper(ZSuper(_, se)) => format_zsuper(ps, se),
        Expression::Yield0(Yield0(_, se)) => format_yield0(ps, se),
        Expression::Return(ret) => format_return(ps, ret),
        Expression::BeginBlock(begin) => format_begin_block(ps, begin),
        Expression::EndBlock(end) => format_end_block(ps, end),
        e => {
            panic!("got unknown token: {:?}", e);
        }
    }
}

pub fn format_program(ps: &mut BaseParserState, program: Program, end_data: Option<&str>) {
    ps.flush_start_of_file_comments();
    debug!("{:?}", program);
    for expression in program.1 {
        format_expression(ps, expression);
    }
    ps.emit_newline();
    ps.on_line(10_000_000_000_000_000_000);
    ps.shift_comments();

    if let Some(end_data) = end_data {
        ps.emit_data_end();
        ps.emit_newline();
        ps.emit_data(end_data);
    }
}
