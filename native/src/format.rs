use crate::parser_state::{FormattingContext, ParserState};
use crate::ripper_tree_types::*;
use crate::types::{FormatStatus, LineNumber};
use std::borrow::Borrow;

pub fn format_def(ps: &mut ParserState, def: Def) {
    let line_number = ((def.1).2).0;
    ps.on_line(line_number);
    let def_name = ((def.1).1).clone();
    ps.emit_indent();
    ps.emit_def(def_name);
    //format_params(ps, def.2, "(".into(), ")".into());
    ps.emit_newline();
    ps.with_formatting_context(FormattingContext::Def, |ps| {
        ps.new_block(|ps| format_bodystmt(ps, def.3));
    });

    ps.emit_end();
    ps.emit_newline();
}

pub fn format_bodystmt(ps: &mut ParserState, bodystmt: BodyStmt) {
    for expression in bodystmt.1 {
        format_expression(ps, expression);
    }
}

pub fn use_parens_for_method_call(
    method: &Box<Expression>,
    chain: &Vec<Expression>,
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
            format_expression(ps, expr);
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

pub fn format_int(ps: &mut ParserState, int: Int) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_int(int.1);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
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
        //"call" => unimplemented!(),
        //"zsuper" => unimplemented!(),
        //"super" => unimplemented!(),
        //"return" => unimplemented!(),
        //"yield" => unimplemented!(),
        e => e,
    }
}

pub fn format_expression(ps: &mut ParserState, expression: Expression) {
    let expression = normalize(expression);
    match expression {
        Expression::Def(def) => format_def(ps, def),
        Expression::BodyStmt(bodystmt) => format_bodystmt(ps, bodystmt),
        Expression::MethodCall(mc) => format_method_call(ps, mc),
        Expression::Ident(ident) => format_ident(ps, ident),
        Expression::Int(int) => format_int(ps, int),
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
