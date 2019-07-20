use crate::parser_state::{FormattingContext, ParserState};
use crate::types::{FormatStatus, LineNumber};
use serde_json::Value;

pub fn to_array_or_error(v: &Value) -> Result<&Vec<Value>, FormatStatus> {
    match v {
        Value::Array(e) => Ok(e),
        _ => return Err(FormatStatus::BadlyFormedTree),
    }
}

pub fn extract_line_number_from(expr: &Vec<Value>) -> LineNumber {
    to_array_or_error(expr.last().expect("line number"))
        .expect("line number")
        .first()
        .expect("line number")
        .as_i64()
        .expect("line number") as LineNumber
}

pub fn format_def(ps: &mut ParserState, def: &Vec<Value>) -> Result<(), FormatStatus> {
    assert!(def.len() == 3);
    let (def_expression, params, body) = (&def[0], &def[1], &def[2]);
    let def_expression = to_array_or_error(def_expression)?;
    let params = to_array_or_error(params)?;
    let body = to_array_or_error(body)?;

    let line_number = extract_line_number_from(def_expression);
    let def_name = def_expression[1].as_str().expect("def name is a string");
    ps.on_line(line_number);

    ps.emit_indent();
    ps.emit_def(def_name.into());
    format_params(ps, params, "(".into(), ")".into())?;
    ps.emit_newline();
    ps.with_formatting_context(FormattingContext::Def, |ps| {
        ps.new_block(|ps| format_expression(ps, body))
    })?;
    ps.emit_end();
    ps.emit_newline();
    Ok(())
}

pub fn format_params(
    ps: &mut ParserState,
    params: &Vec<Value>,
    open_delim: String,
    close_delim: String,
) -> Result<(), FormatStatus> {
    Ok(())
}

pub fn format_bodystmt(ps: &mut ParserState, rest: &Vec<Value>) -> Result<(), FormatStatus> {
    println!("{}", rest.len());
    assert!(rest.len() == 4);
    let expressions = &rest[0];
    let rescue_part = &rest[1];
    let else_part = &rest[2];
    let ensure_part = &rest[3];

    let expressions = to_array_or_error(expressions)?;
    for expr in expressions {
        format_expression(ps, to_array_or_error(expr)?)?;
    }

    Ok(())
}

pub fn use_parens_for_method_call(
    method: &Vec<Value>,
    chain: &Vec<Value>,
    args: &Vec<Value>,
    original_used_parens: bool,
    context: FormattingContext,
) -> bool {
    false
}

pub fn format_method_call(ps: &mut ParserState, rest: &Vec<Value>) -> Result<(), FormatStatus> {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }
    assert!(rest.len() == 4);
    let (chain, method, original_used_parens, args) = (&rest[0], &rest[1], &rest[2], &rest[3]);
    let chain = to_array_or_error(chain)?;
    let method = to_array_or_error(method)?;
    let original_used_parens = match original_used_parens {
        Value::Bool(x) => x.clone(),
        _ => return Err(FormatStatus::BadlyFormedTree),
    };
    let args = to_array_or_error(args)?;

    let use_parens = use_parens_for_method_call(
        method,
        chain,
        args,
        original_used_parens,
        ps.current_formatting_context(),
    );

    ps.with_start_of_line(false, |ps| {
        for expr in chain {
            format_expression(ps, to_array_or_error(expr)?)?;
        }
        Ok(())
    })
}

pub fn normalize_vcall(expression: &Vec<Value>) -> Vec<Value> {
    vec![
        Value::String("method_call".into()),
        Value::Array(vec![]),
        Value::Array(to_array_or_error(&expression[0]).expect("it's an array").clone()),
        Value::Bool(false),
        Value::Array(vec![]),
    ]
}

pub fn normalize(expression: &Vec<Value>) -> Vec<Value> {
    let (head, rest) = (&expression[0], &expression[1..expression.len()]);
    match head.as_str().expect("it's a str") {
        "vcall" => normalize_vcall(&rest.to_vec()),
        "method_add_arg" => unimplemented!(),
        "command" => unimplemented!(),
        "command_call" => unimplemented!(),
        "call" => unimplemented!(),
        "zsuper" => unimplemented!(),
        "super" => unimplemented!(),
        "return" => unimplemented!(),
        "yield" => unimplemented!(),
        e => expression.clone(),
    }
}

pub fn format_expression(
    ps: &mut ParserState,
    expression: &Vec<Value>,
) -> Result<(), FormatStatus> {
    let expression = normalize(expression);
    let head = expression[0].as_str().expect("it's a str");
    let rest = &expression[1..expression.len()].to_vec();
    match head {
        "def" => format_def(ps, rest),
        "bodystmt" => format_bodystmt(ps, rest),
        "method_call" => format_method_call(ps, rest),
        e => {
            println!("got bad token: {}", e);
            return Err(FormatStatus::UnknownToken)
        },
    }
}

pub fn format_program(ps: &mut ParserState, tree: &Vec<Value>) -> Result<(), FormatStatus> {
    assert!(tree.len() == 2);
    let (program, expressions) = (&tree[0], &tree[1]);
    assert!(program == &Value::String("program".into()));
    let expressions = to_array_or_error(expressions)?;

    for expression in expressions {
        let expression = to_array_or_error(expression)?;
        format_expression(ps, expression)?;
    }
    Ok(())
}
