use crate::parser_state::{FormattingContext, ParserState};
use crate::ripper_tree_types::*;
use crate::types::{FormatStatus, LineNumber};
use serde_json::Value;

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

pub fn format_method_call(ps: &mut ParserState, method_call: MethodCall) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    let (chain, method, original_used_parens, args) =
        (method_call.1, method_call.2, method_call.3, method_call.4);

    // let use_parens = use_parens_for_method_call(method, chain, args, original_used_parens,
    // ps.formatting_context);

    ps.with_start_of_line(false, |ps| {
        for expr in chain {
            format_expression(ps, expr);
        }

        match *method {
            Expression::Ident(i) => format_ident(ps, i),
            _ => unimplemented!(),
        };
        //if use_parens
        //  ps.emit_ident("(")
        //elsif args.any?
        //  ps.emit_ident(" ")
        //end

        //ps.with_formatting_context(:args_list) do
        //  format_list_like_thing_items(ps, [args], true)
        //end

        //if use_parens
        //  ps.emit_ident(")")
        //end
    });
}

pub fn format_ident(ps: &mut ParserState, ident: Ident) {
    ps.on_line(ident.line_number());
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.emit_ident(ident.1);
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
        //"method_add_arg" => unimplemented!(),
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
        e => {
            println!("got bad token: {:?}", e);
            panic!("got unknown token");
        }
    }
}

pub fn format_program(ps: &mut ParserState, program: Program) {
    for expression in program.1 {
        format_expression(ps, expression);
    }
}
