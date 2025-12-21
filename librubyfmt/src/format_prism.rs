use std::{borrow::Cow, collections::HashSet, sync::LazyLock};

use ruby_prism as prism;

use crate::{
    delimiters::BreakableDelims,
    heredoc_string::HeredocKind,
    parser_state::{FormattingContext, HashType, ParserState},
    render_targets::MultilineHandling,
    types::SourceOffset,
    util::{const_to_str, const_to_string, loc_to_str, loc_to_string},
};

pub fn format_node(ps: &mut ParserState, node: prism::Node) {
    use prism::Node;

    ps.at_offset(node.location().start_offset());
    // StatementsNode is the only real "wrapper" node, meaning it purely contains
    // other statements which themselves would be at the start of a line.
    // We just ignore it here -- the alternative would be callers might need to have
    // `ps.with_start_of_line(false, ...` for statements, which is semantically confusing
    if ps.at_start_of_line() && !matches!(node, Node::StatementsNode { .. }) {
        ps.emit_indent();
    }

    match node {
        Node::AliasGlobalVariableNode { .. } => {
            format_alias_global_variable_node(ps, node.as_alias_global_variable_node().unwrap())
        }
        Node::AliasMethodNode { .. } => {
            format_alias_method_node(ps, node.as_alias_method_node().unwrap())
        }
        Node::AlternationPatternNode { .. } => {
            format_alternation_pattern_node(ps, node.as_alternation_pattern_node().unwrap())
        }
        Node::AndNode { .. } => format_and_node(ps, node.as_and_node().unwrap()),
        Node::ArgumentsNode { .. } => format_arguments_node(ps, node.as_arguments_node().unwrap()),
        Node::ArrayNode { .. } => format_array_node(ps, node.as_array_node().unwrap()),
        Node::ArrayPatternNode { .. } => {
            format_array_pattern_node(ps, node.as_array_pattern_node().unwrap())
        }
        Node::AssocNode { .. } => format_assoc_node(ps, node.as_assoc_node().unwrap()),
        Node::AssocSplatNode { .. } => {
            format_assoc_splat_node(ps, node.as_assoc_splat_node().unwrap())
        }
        Node::BackReferenceReadNode { .. } => {
            format_back_reference_read_node(ps, node.as_back_reference_read_node().unwrap())
        }
        Node::BeginNode { .. } => format_begin_node(ps, node.as_begin_node().unwrap()),
        Node::BlockArgumentNode { .. } => {
            format_block_argument_node(ps, node.as_block_argument_node().unwrap())
        }
        Node::BlockLocalVariableNode { .. } => {
            format_block_local_variable_node(ps, node.as_block_local_variable_node().unwrap())
        }
        Node::BlockNode { .. } => format_block_node(ps, node.as_block_node().unwrap()),
        Node::BlockParameterNode { .. } => {
            format_block_parameter_node(ps, node.as_block_parameter_node().unwrap())
        }
        Node::BlockParametersNode { .. } => {
            format_block_parameters_node(ps, node.as_block_parameters_node().unwrap())
        }
        Node::BreakNode { .. } => format_break_node(ps, node.as_break_node().unwrap()),
        Node::CallAndWriteNode { .. } => {
            format_call_and_write_node(ps, node.as_call_and_write_node().unwrap())
        }
        Node::CallNode { .. } => {
            let call_node = node.as_call_node().unwrap();
            let is_last_call_in_chain = call_node.receiver().is_none();
            format_call_node(ps, call_node, false, is_last_call_in_chain)
        }
        Node::CallOperatorWriteNode { .. } => {
            format_call_operator_write_node(ps, node.as_call_operator_write_node().unwrap())
        }
        Node::CallOrWriteNode { .. } => {
            format_call_or_write_node(ps, node.as_call_or_write_node().unwrap())
        }
        Node::CallTargetNode { .. } => {
            format_call_target_node(ps, node.as_call_target_node().unwrap())
        }
        Node::CapturePatternNode { .. } => {
            format_capture_pattern_node(ps, node.as_capture_pattern_node().unwrap())
        }
        Node::CaseMatchNode { .. } => {
            format_case_match_node(ps, node.as_case_match_node().unwrap())
        }
        Node::CaseNode { .. } => format_case_node(ps, node.as_case_node().unwrap()),
        Node::ClassNode { .. } => format_class_node(ps, node.as_class_node().unwrap()),
        Node::ClassVariableAndWriteNode { .. } => format_class_variable_and_write_node(
            ps,
            node.as_class_variable_and_write_node().unwrap(),
        ),
        Node::ClassVariableOperatorWriteNode { .. } => format_class_variable_operator_write_node(
            ps,
            node.as_class_variable_operator_write_node().unwrap(),
        ),
        Node::ClassVariableOrWriteNode { .. } => {
            format_class_variable_or_write_node(ps, node.as_class_variable_or_write_node().unwrap())
        }
        Node::ClassVariableReadNode { .. } => {
            format_class_variable_read_node(ps, node.as_class_variable_read_node().unwrap())
        }
        Node::ClassVariableTargetNode { .. } => {
            format_class_variable_target_node(ps, node.as_class_variable_target_node().unwrap())
        }
        Node::ClassVariableWriteNode { .. } => {
            format_class_variable_write_node(ps, node.as_class_variable_write_node().unwrap())
        }
        Node::ConstantAndWriteNode { .. } => {
            format_constant_and_write_node(ps, node.as_constant_and_write_node().unwrap())
        }
        Node::ConstantOperatorWriteNode { .. } => {
            format_constant_operator_write_node(ps, node.as_constant_operator_write_node().unwrap())
        }
        Node::ConstantOrWriteNode { .. } => {
            format_constant_or_write_node(ps, node.as_constant_or_write_node().unwrap())
        }
        Node::ConstantPathAndWriteNode { .. } => {
            format_constant_path_and_write_node(ps, node.as_constant_path_and_write_node().unwrap())
        }
        Node::ConstantPathNode { .. } => {
            format_constant_path_node(ps, node.as_constant_path_node().unwrap())
        }
        Node::ConstantPathOperatorWriteNode { .. } => format_constant_path_operator_write_node(
            ps,
            node.as_constant_path_operator_write_node().unwrap(),
        ),
        Node::ConstantPathOrWriteNode { .. } => {
            format_constant_path_or_write_node(ps, node.as_constant_path_or_write_node().unwrap())
        }
        Node::ConstantPathTargetNode { .. } => {
            format_constant_path_target_node(ps, node.as_constant_path_target_node().unwrap())
        }
        Node::ConstantPathWriteNode { .. } => {
            format_constant_path_write_node(ps, node.as_constant_path_write_node().unwrap())
        }
        Node::ConstantReadNode { .. } => {
            format_constant_read_node(ps, node.as_constant_read_node().unwrap())
        }
        Node::ConstantTargetNode { .. } => {
            format_constant_target_node(ps, node.as_constant_target_node().unwrap())
        }
        Node::ConstantWriteNode { .. } => {
            format_constant_write_node(ps, node.as_constant_write_node().unwrap())
        }
        Node::DefNode { .. } => format_def_node(ps, node.as_def_node().unwrap()),
        Node::DefinedNode { .. } => format_defined_node(ps, node.as_defined_node().unwrap()),
        Node::ElseNode { .. } => format_else_node(ps, node.as_else_node().unwrap()),
        Node::EmbeddedStatementsNode { .. } => {
            format_embedded_statements_node(ps, node.as_embedded_statements_node().unwrap())
        }
        Node::EmbeddedVariableNode { .. } => {
            format_embedded_variable_node(ps, node.as_embedded_variable_node().unwrap())
        }
        Node::EnsureNode { .. } => format_ensure_node(ps, node.as_ensure_node().unwrap()),
        Node::FalseNode { .. } => format_false_node(ps, node.as_false_node().unwrap()),
        Node::FindPatternNode { .. } => {
            format_find_pattern_node(ps, node.as_find_pattern_node().unwrap())
        }
        Node::FlipFlopNode { .. } => format_flip_flop_node(ps, node.as_flip_flop_node().unwrap()),
        Node::FloatNode { .. } => format_float_node(ps, node.as_float_node().unwrap()),
        Node::ForNode { .. } => format_for_node(ps, node.as_for_node().unwrap()),
        Node::ForwardingArgumentsNode { .. } => {
            format_forwarding_arguments_node(ps, node.as_forwarding_arguments_node().unwrap())
        }
        Node::ForwardingParameterNode { .. } => {
            format_forwarding_parameter_node(ps, node.as_forwarding_parameter_node().unwrap())
        }
        Node::ForwardingSuperNode { .. } => {
            format_forwarding_super_node(ps, node.as_forwarding_super_node().unwrap())
        }
        Node::GlobalVariableAndWriteNode { .. } => format_global_variable_and_write_node(
            ps,
            node.as_global_variable_and_write_node().unwrap(),
        ),
        Node::GlobalVariableOperatorWriteNode { .. } => format_global_variable_operator_write_node(
            ps,
            node.as_global_variable_operator_write_node().unwrap(),
        ),
        Node::GlobalVariableOrWriteNode { .. } => format_global_variable_or_write_node(
            ps,
            node.as_global_variable_or_write_node().unwrap(),
        ),
        Node::GlobalVariableReadNode { .. } => {
            format_global_variable_read_node(ps, node.as_global_variable_read_node().unwrap())
        }
        Node::GlobalVariableTargetNode { .. } => {
            format_global_variable_target_node(ps, node.as_global_variable_target_node().unwrap())
        }
        Node::GlobalVariableWriteNode { .. } => {
            format_global_variable_write_node(ps, node.as_global_variable_write_node().unwrap())
        }
        Node::HashNode { .. } => format_hash_node(ps, node.as_hash_node().unwrap()),
        Node::HashPatternNode { .. } => {
            format_hash_pattern_node(ps, node.as_hash_pattern_node().unwrap())
        }
        Node::IfNode { .. } => format_if_node(ps, node.as_if_node().unwrap()),
        Node::ImaginaryNode { .. } => format_imaginary_node(ps, node.as_imaginary_node().unwrap()),
        Node::ImplicitNode { .. } => format_implicit_node(),
        Node::ImplicitRestNode { .. } => format_implicit_rest_node(),
        Node::InNode { .. } => format_in_node(ps, node.as_in_node().unwrap()),
        Node::IndexAndWriteNode { .. } => {
            format_index_and_write_node(ps, node.as_index_and_write_node().unwrap())
        }
        Node::IndexOperatorWriteNode { .. } => {
            format_index_operator_write_node(ps, node.as_index_operator_write_node().unwrap())
        }
        Node::IndexOrWriteNode { .. } => {
            format_index_or_write_node(ps, node.as_index_or_write_node().unwrap())
        }
        Node::IndexTargetNode { .. } => {
            format_index_target_node(ps, node.as_index_target_node().unwrap())
        }
        Node::InstanceVariableAndWriteNode { .. } => format_instance_variable_and_write_node(
            ps,
            node.as_instance_variable_and_write_node().unwrap(),
        ),
        Node::InstanceVariableOperatorWriteNode { .. } => {
            format_instance_variable_operator_write_node(
                ps,
                node.as_instance_variable_operator_write_node().unwrap(),
            )
        }
        Node::InstanceVariableOrWriteNode { .. } => format_instance_variable_or_write_node(
            ps,
            node.as_instance_variable_or_write_node().unwrap(),
        ),
        Node::InstanceVariableReadNode { .. } => {
            format_instance_variable_read_node(ps, node.as_instance_variable_read_node().unwrap())
        }
        Node::InstanceVariableTargetNode { .. } => format_instance_variable_target_node(
            ps,
            node.as_instance_variable_target_node().unwrap(),
        ),
        Node::InstanceVariableWriteNode { .. } => {
            format_instance_variable_write_node(ps, node.as_instance_variable_write_node().unwrap())
        }
        Node::IntegerNode { .. } => format_integer_node(ps, node.as_integer_node().unwrap()),
        Node::InterpolatedMatchLastLineNode { .. } => format_interpolated_last_line_node(
            ps,
            node.as_interpolated_match_last_line_node().unwrap(),
        ),
        Node::InterpolatedRegularExpressionNode { .. } => {
            format_interpolated_regular_expression_node(
                ps,
                node.as_interpolated_regular_expression_node().unwrap(),
            )
        }
        Node::InterpolatedStringNode { .. } => {
            format_interpolated_string_node(ps, node.as_interpolated_string_node().unwrap())
        }
        Node::InterpolatedSymbolNode { .. } => {
            format_interpolated_symbol_node(ps, node.as_interpolated_symbol_node().unwrap())
        }
        Node::InterpolatedXStringNode { .. } => {
            format_interpolated_x_string_node(ps, node.as_interpolated_x_string_node().unwrap())
        }
        Node::ItLocalVariableReadNode { .. } => {
            format_it_local_variable_read_node(ps, node.as_it_local_variable_read_node().unwrap())
        }
        Node::ItParametersNode { .. } => format_it_parameters_node(),
        Node::KeywordHashNode { .. } => {
            format_keyword_hash_node(ps, node.as_keyword_hash_node().unwrap())
        }
        Node::KeywordRestParameterNode { .. } => {
            format_keyword_rest_parameter_node(ps, node.as_keyword_rest_parameter_node().unwrap())
        }
        Node::LambdaNode { .. } => format_lambda_node(ps, node.as_lambda_node().unwrap()),
        Node::LocalVariableAndWriteNode { .. } => format_local_variable_and_write_node(
            ps,
            node.as_local_variable_and_write_node().unwrap(),
        ),
        Node::LocalVariableOperatorWriteNode { .. } => format_local_variable_operator_write_node(
            ps,
            node.as_local_variable_operator_write_node().unwrap(),
        ),
        Node::LocalVariableOrWriteNode { .. } => {
            format_local_variable_or_write_node(ps, node.as_local_variable_or_write_node().unwrap())
        }
        Node::LocalVariableReadNode { .. } => {
            format_local_variable_read_node(ps, node.as_local_variable_read_node().unwrap())
        }
        Node::LocalVariableTargetNode { .. } => {
            format_local_variable_target_node(ps, node.as_local_variable_target_node().unwrap())
        }
        Node::LocalVariableWriteNode { .. } => {
            format_local_variable_write_node(ps, node.as_local_variable_write_node().unwrap())
        }
        Node::MatchLastLineNode { .. } => {
            format_match_last_line_node(ps, node.as_match_last_line_node().unwrap())
        }
        Node::MatchPredicateNode { .. } => {
            format_match_predicate_node(ps, node.as_match_predicate_node().unwrap())
        }
        Node::MatchRequiredNode { .. } => {
            format_match_required_node(ps, node.as_match_required_node().unwrap())
        }
        Node::MatchWriteNode { .. } => {
            format_match_write_node(ps, node.as_match_write_node().unwrap())
        }
        Node::MissingNode { .. } => unreachable!(
            "MissingNode should only occur in files with syntax errors, which cannot be formatted"
        ),
        Node::ModuleNode { .. } => format_module_node(ps, node.as_module_node().unwrap()),
        Node::MultiTargetNode { .. } => {
            format_multi_target_node(ps, node.as_multi_target_node().unwrap())
        }
        Node::MultiWriteNode { .. } => {
            format_multi_write_node(ps, node.as_multi_write_node().unwrap())
        }
        Node::NextNode { .. } => format_next_node(ps, node.as_next_node().unwrap()),
        Node::NilNode { .. } => format_nil_node(ps),
        Node::NoKeywordsParameterNode { .. } => {
            format_no_keywords_parameter_node(ps, node.as_no_keywords_parameter_node().unwrap())
        }
        Node::NumberedParametersNode { .. } => format_numbered_parameters_node(),
        Node::NumberedReferenceReadNode { .. } => {
            format_numbered_reference_read_node(ps, node.as_numbered_reference_read_node().unwrap())
        }
        Node::OptionalKeywordParameterNode { .. } => format_optional_keyword_parameter_node(
            ps,
            node.as_optional_keyword_parameter_node().unwrap(),
        ),
        Node::OptionalParameterNode { .. } => {
            format_optional_parameter_node(ps, node.as_optional_parameter_node().unwrap())
        }
        Node::OrNode { .. } => format_or_node(ps, node.as_or_node().unwrap()),
        Node::ParametersNode { .. } => {
            format_parameters_node(ps, node.as_parameters_node().unwrap())
        }
        Node::ParenthesesNode { .. } => {
            format_parentheses_node(ps, node.as_parentheses_node().unwrap())
        }
        Node::PinnedExpressionNode { .. } => {
            format_pinned_expression_node(ps, node.as_pinned_expression_node().unwrap())
        }
        Node::PinnedVariableNode { .. } => {
            format_pinned_variable_node(ps, node.as_pinned_variable_node().unwrap())
        }
        Node::PostExecutionNode { .. } => {
            format_post_execution_node(ps, node.as_post_execution_node().unwrap())
        }
        Node::PreExecutionNode { .. } => {
            format_pre_execution_node(ps, node.as_pre_execution_node().unwrap())
        }
        Node::ProgramNode { .. } => format_program(ps, node.as_program_node().unwrap(), None),
        Node::RangeNode { .. } => format_range_node(ps, node.as_range_node().unwrap()),
        Node::RationalNode { .. } => format_rational_node(ps, node.as_rational_node().unwrap()),
        Node::RedoNode { .. } => format_redo_node(ps),
        Node::RegularExpressionNode { .. } => {
            format_regular_expression_node(ps, node.as_regular_expression_node().unwrap())
        }
        Node::RequiredKeywordParameterNode { .. } => format_required_keyword_parameter_node(
            ps,
            node.as_required_keyword_parameter_node().unwrap(),
        ),
        Node::RequiredParameterNode { .. } => {
            format_required_parameter_node(ps, node.as_required_parameter_node().unwrap())
        }
        Node::RescueModifierNode { .. } => {
            format_rescue_modifier_node(ps, node.as_rescue_modifier_node().unwrap())
        }
        Node::RescueNode { .. } => format_rescue_node(ps, node.as_rescue_node().unwrap()),
        Node::RestParameterNode { .. } => {
            format_rest_parameter_node(ps, node.as_rest_parameter_node().unwrap())
        }
        Node::RetryNode { .. } => format_retry_node(ps),
        Node::ReturnNode { .. } => format_return_node(ps, node.as_return_node().unwrap()),
        Node::SelfNode { .. } => format_self_node(ps),
        Node::ShareableConstantNode { .. } => {
            format_shareable_constant_node(ps, node.as_shareable_constant_node().unwrap())
        }
        Node::SingletonClassNode { .. } => {
            format_singleton_class_node(ps, node.as_singleton_class_node().unwrap())
        }
        Node::SourceEncodingNode { .. } => {
            format_source_encoding_node(ps, node.as_source_encoding_node().unwrap())
        }
        Node::SourceFileNode { .. } => {
            format_source_file_node(ps, node.as_source_file_node().unwrap())
        }
        Node::SourceLineNode { .. } => {
            format_source_line_node(ps, node.as_source_line_node().unwrap())
        }
        Node::SplatNode { .. } => format_splat_node(ps, node.as_splat_node().unwrap()),
        Node::StatementsNode { .. } => format_statements(ps, node.as_statements_node().unwrap()),
        Node::StringNode { .. } => format_string_node(ps, node.as_string_node().unwrap()),
        Node::SuperNode { .. } => format_super_node(ps, node.as_super_node().unwrap()),
        Node::SymbolNode { .. } => format_symbol_node(ps, node.as_symbol_node().unwrap()),
        Node::TrueNode { .. } => format_true_node(ps, node.as_true_node().unwrap()),
        Node::UndefNode { .. } => format_undef_node(ps, node.as_undef_node().unwrap()),
        Node::UnlessNode { .. } => format_unless_node(ps, node.as_unless_node().unwrap()),
        Node::UntilNode { .. } => format_until_node(ps, node.as_until_node().unwrap()),
        Node::WhenNode { .. } => format_when_node(ps, node.as_when_node().unwrap()),
        Node::WhileNode { .. } => format_while_node(ps, node.as_while_node().unwrap()),
        Node::XStringNode { .. } => format_x_string_node(ps, node.as_x_string_node().unwrap()),
        Node::YieldNode { .. } => format_yield_node(ps, node.as_yield_node().unwrap()),
    }

    ps.at_offset(node.location().end_offset());
    if ps.at_start_of_line() && !matches!(node, Node::StatementsNode { .. }) {
        ps.emit_newline();
    }
}

fn format_alias_global_variable_node(
    ps: &mut ParserState,
    alias_global_variable_node: prism::AliasGlobalVariableNode,
) {
    ps.emit_ident("alias".to_string());
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        format_node(ps, alias_global_variable_node.new_name());
        ps.emit_space();
        format_node(ps, alias_global_variable_node.old_name());
    });
}

fn format_alias_method_node(ps: &mut ParserState, alias_method_node: prism::AliasMethodNode) {
    ps.emit_ident("alias ".to_string());

    ps.with_start_of_line(false, |ps| {
        format_node(ps, alias_method_node.new_name());
        ps.emit_space();
        format_node(ps, alias_method_node.old_name());
    });
}

fn format_alternation_pattern_node(
    _ps: &mut ParserState,
    _alternation_pattern_node: prism::AlternationPatternNode,
) {
    todo!()
}

fn format_and_node(ps: &mut ParserState, and_node: prism::AndNode) {
    ps.inline_breakable_of(BreakableDelims::for_binary_op(), |ps| {
        ps.with_start_of_line(false, |ps| {
            format_infix_operator(
                ps,
                and_node.left(),
                loc_to_string(and_node.operator_loc()),
                and_node.right(),
            );
        });
    });
}

fn format_back_reference_read_node(
    ps: &mut ParserState,
    back_reference_read_node: prism::BackReferenceReadNode,
) {
    let back_reference_loc = back_reference_read_node.location();
    let end_offset = back_reference_loc.end_offset();

    handle_string_at_offset(ps, loc_to_string(back_reference_loc), end_offset);
}

fn format_begin_node(ps: &mut ParserState, begin_node: prism::BeginNode) {
    // If there's no `begin` keyword loc, this is probably an "implicit" begin node,
    // like a rescue/ensure in a def without a `begin` keyword:
    // ```ruby
    //   def foo
    //     raise "Ahh!"
    //   rescue
    //   end
    // ```
    let is_implicit_begin_node = begin_node.begin_keyword_loc().is_none();

    // Double check that these offsets are correct, since begin/rescue/ensure/else
    // aren't always handled with `format_node`, which usually handles this
    ps.at_offset(begin_node.location().start_offset());

    if is_implicit_begin_node {
        // We assume we're in a context that's already been indented, e.g.
        // the body of a `def`
        ps.end_indent();
    } else {
        ps.emit_keyword("begin");
    }
    ps.new_block(|ps| {
        // For implicit nodes, this newline was already emitted by the caller
        if !is_implicit_begin_node {
            ps.emit_newline();
        }
        if let Some(statements_node) = begin_node.statements() {
            format_statements(ps, statements_node);
        }
    });

    ps.with_start_of_line(true, |ps| {
        if let Some(rescue_node) = begin_node.rescue_clause() {
            ps.emit_indent();
            format_rescue_node(ps, rescue_node);
        }

        if let Some(else_node) = begin_node.else_clause() {
            ps.emit_indent();
            format_else_node(ps, else_node);
        }

        if let Some(ensure_node) = begin_node.ensure_clause() {
            ps.emit_indent();
            format_ensure_node(ps, ensure_node);
        }

        if !is_implicit_begin_node {
            ps.emit_end();
        }
    });

    if is_implicit_begin_node {
        ps.start_indent();
    }

    ps.at_offset(begin_node.location().end_offset());
}

fn format_break_node(ps: &mut ParserState, break_node: prism::BreakNode) {
    ps.emit_ident("break".to_string());
    if let Some(arguments_node) = break_node.arguments() {
        ps.with_start_of_line(false, |ps| {
            ps.breakable_of(BreakableDelims::for_kw(), |ps| {
                format_arguments_node(ps, arguments_node);
            });
        });
    }
}

fn format_capture_pattern_node(
    _ps: &mut ParserState,
    _capture_pattern_node: prism::CapturePatternNode,
) {
    todo!()
}

fn format_case_match_node(_ps: &mut ParserState, _case_match_node: prism::CaseMatchNode) {
    todo!()
}

fn format_case_node(ps: &mut ParserState, case_node: prism::CaseNode) {
    ps.emit_case_keyword();

    if let Some(predicate) = case_node.predicate() {
        ps.with_start_of_line(false, |ps| {
            ps.emit_space();
            format_node(ps, predicate);
        });
    }

    ps.emit_newline();
    ps.with_start_of_line(true, |ps| {
        for condition in case_node.conditions().iter() {
            format_when_node(ps, condition.as_when_node().unwrap());
        }

        if let Some(else_node) = case_node.else_clause() {
            ps.emit_indent();
            format_else_node(ps, else_node);
        }

        ps.emit_end();
    });
}

pub fn format_program(
    ps: &mut ParserState,
    program_node: prism::ProgramNode,
    data_loc: Option<prism::Location>,
) {
    ps.with_start_of_line(true, |ps| {
        format_statements(ps, program_node.statements());
    });
    ps.emit_newline();
    ps.on_line(10000000000);
    ps.shift_comments();

    if let Some(data) = data_loc {
        ps.emit_data(loc_to_str(data));
    }
}

fn format_statements(ps: &mut ParserState, statements_node: prism::StatementsNode) {
    ps.with_start_of_line(true, |ps| {
        for node in statements_node.body().iter() {
            format_node(ps, node);
        }
    });
}

fn format_string_node(ps: &mut ParserState, string_node: prism::StringNode) {
    ps.at_offset(string_node.location().start_offset());

    // `opening_loc()` is only `None` in the case of the inner parts of multiline strings
    // (e.g. the inner contents of a heredoc)
    let opener = string_node
        .opening_loc()
        .map(|s| loc_to_str(s).trim().to_string());
    let closer = string_node
        .closing_loc()
        .map(|s| loc_to_str(s).trim().to_string());
    let is_heredoc = opener.as_ref().map(|s| s.starts_with("<")).unwrap_or(false);

    if is_heredoc {
        format_heredoc(
            ps,
            HeredocNodeType::Plain(string_node),
            opener
                .expect("Heredocs must have an opening loc for the opening tag (<<FOO etc.)")
                .to_string(),
        );
        return;
    }

    ps.with_start_of_line(false, |ps| {
        // Always use double quotes over single quotes/percent literals
        if opener.is_some() {
            ps.emit_double_quote();
        }

        // If opener is nil, we must be in some kind of interpolated string context, which
        // means the contents must already be appropriately escaped -- hence we default to `true` here
        let in_escaped_context =
            is_heredoc || opener.as_ref().map(|s| s.starts_with("\"")).unwrap_or(true);
        let string_content = if in_escaped_context {
            loc_to_string(string_node.content_loc())
        } else {
            // For character literals (`?a`), there can be an opening loc without
            // a closing loc. In that case, fall back to a double quote, since
            // we render character literals as double-quoted string literals
            let end_delim = if let Some(ref closer) = closer {
                closer.as_str()
            } else {
                "\""
            };

            crate::string_escape::single_to_double_quoted(
                loc_to_str(string_node.content_loc()),
                opener.as_ref().unwrap(),
                end_delim,
            )
        };

        ps.emit_string_content(string_content);
        ps.wind_dumping_comments_until_offset(string_node.content_loc().end_offset());

        if opener.is_some() {
            ps.emit_double_quote();
        }
    });

    ps.wind_dumping_comments_until_offset(string_node.location().end_offset());
}

fn format_interpolated_string_node(
    ps: &mut ParserState,
    interpolated_string_node: prism::InterpolatedStringNode,
) {
    let opener = interpolated_string_node
        .opening_loc()
        .map(|s| loc_to_str(s).trim().to_string());
    let closer = interpolated_string_node
        .closing_loc()
        .map(|s| loc_to_str(s).trim().to_string());
    let is_heredoc = opener.as_ref().map(|s| s.starts_with("<")).unwrap_or(false);
    let needs_escape = opener
        .as_ref()
        .map(|s| !s.starts_with("\"") && !is_heredoc)
        .unwrap_or(false);

    // Prism actually handles string concatenation when using "\", so it treats
    // ```ruby
    // "foo" \
    //   "bar"
    // ```
    // as an interpolated node with the contents `"foobar"` (in two `parts` of "foo" and "bar").
    // To detect this, we can look for any `InterpolatedStringNode` that has multiple `parts`
    // and isn't a heredoc.
    //
    // Note that Prism itself essentially scrubs any traces of heredocs, and so we have to detect it ourselves.
    // The logic for this is taken straight from Prism's Ripper translator:
    // https://github.com/ruby/prism/blob/609c80c91e146d9ac8f70deedfadca729e8a3e4f/lib/prism/translation/ripper.rb#L2183-L2189
    let is_backslash_string_interpolation = !is_heredoc
        && interpolated_string_node.parts().len() > 1
        && interpolated_string_node.parts().iter().any(|node| {
            node.as_string_node()
                .map(|node| node.opening_loc().is_some())
                .unwrap_or(false)
                || node
                    .as_interpolated_string_node()
                    .map(|node| node.opening_loc().is_some())
                    .unwrap_or(false)
        });

    ps.at_offset(interpolated_string_node.location().start_offset());

    if is_heredoc {
        format_heredoc(
            ps,
            HeredocNodeType::Interpolated(interpolated_string_node),
            opener
                .expect("Heredocs must have an opening loc for the opening tag (<<FOO etc.)")
                .to_string(),
        );
        // The rest of this machinery is handled in format_inner_string
        // From here on out, assume we're not in a heredoc
        return;
    }

    if let Some(s) = &opener {
        if needs_escape {
            ps.emit_double_quote();
        } else {
            ps.emit_string_content(s.clone());
        }
    }

    ps.with_start_of_line(false, |ps| {
        let string_parts_count = interpolated_string_node.parts().len();
        for (i, part) in interpolated_string_node.parts().iter().enumerate() {
            let start_offset = part.location().start_offset();
            let end_offset = part.location().end_offset();

            ps.at_offset(start_offset);
            let indent_for_consecutive_strings = is_backslash_string_interpolation && i > 0;

            if indent_for_consecutive_strings {
                ps.start_indent();
                ps.emit_newline();
                ps.emit_indent();
            }

            if needs_escape && let Some(string_node) = part.as_string_node() {
                let content = loc_to_str(string_node.content_loc());
                let escaped = crate::string_escape::single_to_double_quoted(
                    content,
                    opener.as_ref().unwrap(),
                    closer.as_deref().unwrap_or("\""),
                );
                ps.emit_string_content(escaped);
            } else {
                format_node(ps, part);
            }

            // For non-backslash-concatenated multiline strings, `part` contains newlines and indentation,
            // so we don't need to handle that ourselves.
            if is_backslash_string_interpolation && i < string_parts_count - 1 {
                if let Some(s) = &closer {
                    if needs_escape {
                        ps.emit_double_quote();
                    } else {
                        ps.emit_string_content(s.clone());
                    }
                }
                ps.emit_space();
                ps.emit_slash();
            }

            ps.at_offset(end_offset);
            if indent_for_consecutive_strings {
                ps.end_indent();
            }
        }
    });

    if let Some(closer) = &closer {
        if needs_escape {
            ps.emit_double_quote();
        } else {
            ps.emit_string_content(closer.clone());
        }
    }
}

enum HeredocNodeType<'h> {
    Plain(prism::StringNode<'h>),
    Interpolated(prism::InterpolatedStringNode<'h>),
}

impl HeredocNodeType<'_> {
    fn parts(&self) -> Vec<prism::Node<'_>> {
        match self {
            HeredocNodeType::Plain(string_node) => vec![string_node.as_node()],
            HeredocNodeType::Interpolated(interpolated_string_node) => {
                interpolated_string_node.parts().iter().collect::<Vec<_>>()
            }
        }
    }

    fn closing_loc(&self) -> prism::Location<'_> {
        match self {
            HeredocNodeType::Plain(string_node) => string_node
                .closing_loc()
                .expect("This is a heredoc, it must have a loc for the closing tag"),
            HeredocNodeType::Interpolated(interpolated_string_node) => interpolated_string_node
                .closing_loc()
                .expect("This is a heredoc, it must have a loc for the closing tag"),
        }
    }
}

fn format_heredoc(ps: &mut ParserState, heredoc: HeredocNodeType, heredoc_symbol: String) {
    let heredoc_kind = HeredocKind::from_string(&heredoc_symbol);
    ps.emit_heredoc_start(heredoc_symbol, heredoc_kind);

    ps.push_heredoc_content(
        loc_to_str(heredoc.closing_loc()).trim().to_string(),
        heredoc_kind,
        ps.get_line_number_for_offset(heredoc.closing_loc().start_offset()),
        Box::new(|n: &mut ParserState| {
            n.disable_user_newlines();
            format_inner_string(n, heredoc.parts(), heredoc_kind);
        }),
    );
    ps.wind_dumping_comments_until_offset(heredoc.closing_loc().start_offset());
}

fn maybe_render_heredocs_in_string<'a>(
    ps: &mut ParserState,
    peekable: &mut std::iter::Peekable<impl Iterator<Item = (usize, &'a prism::Node<'a>)>>,
) {
    let should_render = peekable
        .peek()
        .and_then(|(_, node)| {
            node.as_string_node()
                .map(|sn| loc_to_str(sn.content_loc()).starts_with('\n'))
        })
        .unwrap_or(false);
    if should_render {
        ps.render_heredocs(true)
    }
}

fn format_inner_string(ps: &mut ParserState, parts: Vec<prism::Node>, heredoc_kind: HeredocKind) {
    // For squiggly heredocs, calculate the common indentation to strip.
    // We only look at lines that start at the beginning of a StringNode part
    // (not continuations after an interpolation on the same line).
    let common_indent = if heredoc_kind.is_squiggly() {
        parts
            .iter()
            .enumerate()
            .filter_map(|(i, part)| {
                if let Some(node) = part.as_string_node() {
                    let content = loc_to_str(node.content_loc());
                    // Only consider the first line of this part if it follows a newline
                    // (i.e., if the previous part ended with a newline, or this is the first part)
                    let prev_ends_with_newline = if i == 0 {
                        true
                    } else {
                        // After interpolation, might not be at line start
                        let default_value = false;
                        parts[i - 1]
                            .as_string_node()
                            .map(|node| loc_to_str(node.content_loc()).ends_with('\n'))
                            .unwrap_or(default_value)
                    };

                    // Find minimum indent, but skip the first line if it doesn't start
                    // at a line boundary (i.e., it follows an interpolation)
                    content
                        .lines()
                        .enumerate()
                        .filter(|(line_idx, line)| {
                            !line.trim().is_empty() && (*line_idx > 0 || prev_ends_with_newline)
                        })
                        .map(|(_, line)| line.len() - line.trim_start().len())
                        .min()
                } else {
                    None
                }
            })
            .min()
            .unwrap_or(0)
    } else {
        0
    };

    let mut peekable = parts.iter().enumerate().peekable();
    let mut prev_ended_with_newline = true;

    while let Some((idx, part)) = peekable.next() {
        match part {
            prism::Node::StringNode { .. } => {
                let part = part.as_string_node().unwrap();
                // For heredocs, use raw `content_loc` to preserve escape sequences like `\n`
                let mut contents = {
                    let raw = loc_to_str(part.content_loc());
                    if common_indent > 0 {
                        // Track whether this part's first line is at a true line boundary
                        let first_line_is_at_boundary = if idx == 0 {
                            true
                        } else {
                            prev_ended_with_newline
                        };

                        raw.split('\n')
                            .enumerate()
                            .map(|(line_idx, line)| {
                                // Only strip from lines that:
                                // 1. Are at a true line boundary
                                // 2. Have enough characters to strip
                                let should_strip = (line_idx > 0 || first_line_is_at_boundary)
                                    && !line.is_empty()
                                    && line.len() >= common_indent;
                                if should_strip {
                                    &line[common_indent..]
                                } else {
                                    line
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        raw.to_string()
                    }
                };

                prev_ended_with_newline = contents.ends_with('\n');

                if peekable.peek().is_none() && contents.ends_with('\n') {
                    contents.pop();
                }

                ps.at_offset(part.location().end_offset());
                ps.emit_string_content(contents);
            }
            prism::Node::InterpolatedStringNode { .. } => {
                ps.at_offset(part.location().start_offset());
                format_interpolated_string_node(ps, part.as_interpolated_string_node().unwrap());
                maybe_render_heredocs_in_string(ps, &mut peekable);
                prev_ended_with_newline = false;
            }
            prism::Node::EmbeddedStatementsNode { .. } => {
                format_embedded_statements_node(ps, part.as_embedded_statements_node().unwrap());
                maybe_render_heredocs_in_string(ps, &mut peekable);
                prev_ended_with_newline = false;
            }
            prism::Node::EmbeddedVariableNode { .. } => {
                format_embedded_variable_node(ps, part.as_embedded_variable_node().unwrap());
                prev_ended_with_newline = false;
            }
            x => unreachable!("Unexpected Node type in heredoc: {:?}", x),
        }
    }
}

fn format_interpolated_symbol_node(
    ps: &mut ParserState,
    interpolated_symbol_node: prism::InterpolatedSymbolNode,
) {
    ps.emit_ident(":".to_string());
    ps.emit_double_quote();

    ps.with_start_of_line(false, |ps| {
        for part in interpolated_symbol_node.parts().iter() {
            let start_offset = part.location().start_offset();
            let end_offset = part.location().end_offset();

            ps.at_offset(start_offset);

            format_node(ps, part);

            ps.at_offset(end_offset);
        }
    });

    ps.emit_double_quote();
}

fn format_interpolated_x_string_node(
    ps: &mut ParserState,
    interpolated_x_string_node: prism::InterpolatedXStringNode,
) {
    ps.emit_ident("`".to_string());

    ps.with_start_of_line(false, |ps| {
        for part in interpolated_x_string_node.parts().iter() {
            let start_offset = part.location().start_offset();
            let end_offset = part.location().end_offset();

            ps.at_offset(start_offset);
            format_node(ps, part);
            ps.at_offset(end_offset);
        }
    });

    ps.emit_ident("`".to_string());
}

fn format_it_local_variable_read_node(
    ps: &mut ParserState,
    it_local_variable_read_node: prism::ItLocalVariableReadNode,
) {
    handle_string_at_offset(
        ps,
        loc_to_string(it_local_variable_read_node.location()),
        it_local_variable_read_node.location().start_offset(),
    );
}

fn format_it_parameters_node() {
    // No-op. This node represents the implicit 'it' parameter,
    // and the actual parameter references are rendered separately.
}

fn format_interpolated_last_line_node(
    ps: &mut ParserState,
    interpolated_match_last_line_node: prism::InterpolatedMatchLastLineNode,
) {
    ps.emit_ident(loc_to_string(
        interpolated_match_last_line_node.opening_loc(),
    ));

    ps.with_start_of_line(false, |ps| {
        for part in interpolated_match_last_line_node.parts().iter() {
            let start_offset = part.location().start_offset();
            let end_offset = part.location().end_offset();

            ps.at_offset(start_offset);
            format_node(ps, part);
            ps.at_offset(end_offset);
        }
    });

    ps.emit_ident(loc_to_string(
        interpolated_match_last_line_node.closing_loc(),
    ));
}

fn format_interpolated_regular_expression_node(
    ps: &mut ParserState,
    interpolated_regular_expression_node: prism::InterpolatedRegularExpressionNode,
) {
    ps.emit_ident(loc_to_string(
        interpolated_regular_expression_node.opening_loc(),
    ));

    ps.with_start_of_line(false, |ps| {
        for part in interpolated_regular_expression_node.parts().iter() {
            let start_offset = part.location().start_offset();
            let end_offset = part.location().end_offset();

            ps.at_offset(start_offset);
            format_node(ps, part);
            ps.at_offset(end_offset);
        }
    });

    ps.emit_ident(loc_to_string(
        interpolated_regular_expression_node.closing_loc(),
    ));
}

fn format_embedded_statements_node(
    ps: &mut ParserState,
    embedded_statements_node: prism::EmbeddedStatementsNode,
) {
    ps.emit_string_content("#{".to_string());
    if let Some(statements) = embedded_statements_node.statements() {
        ps.with_formatting_context(FormattingContext::StringEmbexpr, |ps| {
            let has_multiple_statements = statements.body().len() > 1;
            ps.with_start_of_line(has_multiple_statements, |ps| {
                if has_multiple_statements {
                    ps.emit_newline();
                    ps.new_block(|ps| format_node(ps, statements.as_node()));
                    ps.emit_indent();
                } else if let Some(statement) = statements.body().iter().next() {
                    format_node(ps, statement);
                }
            });
        });
    }
    ps.emit_string_content("}".to_string());
}

fn format_embedded_variable_node(
    ps: &mut ParserState,
    embedded_variable_node: prism::EmbeddedVariableNode,
) {
    ps.emit_string_content("#{".to_string());
    format_node(ps, embedded_variable_node.variable());
    ps.emit_string_content("}".to_string());
}

fn format_ensure_node(ps: &mut ParserState, ensure_node: prism::EnsureNode) {
    // Double check that these offsets are correct, since begin/rescue/ensure/else
    // aren't always handled with `format_node`, which usually handles this
    ps.at_offset(ensure_node.location().start_offset());

    ps.emit_keyword("ensure");
    ps.new_block(|ps| {
        ps.emit_newline();
        if let Some(statements) = ensure_node.statements() {
            format_statements(ps, statements);
        }
    });

    ps.at_offset(ensure_node.location().end_offset());
}

fn format_false_node(ps: &mut ParserState, false_node: prism::FalseNode) {
    handle_string_at_offset(
        ps,
        "false".to_string(),
        false_node.location().start_offset(),
    );
}

fn format_find_pattern_node(_ps: &mut ParserState, _find_pattern_node: prism::FindPatternNode) {
    todo!()
}

fn format_flip_flop_node(ps: &mut ParserState, flip_flop_node: prism::FlipFlopNode) {
    ps.with_start_of_line(false, |ps| {
        if let Some(left) = flip_flop_node.left() {
            format_node(ps, left);
        }
        ps.emit_op(Cow::Owned(loc_to_string(flip_flop_node.operator_loc())));
        if let Some(right) = flip_flop_node.right() {
            format_node(ps, right);
        }
    });
}

fn format_class_node(ps: &mut ParserState, class_node: prism::ClassNode) {
    ps.emit_class_keyword();
    ps.emit_space();
    ps.with_start_of_line(false, |ps| format_node(ps, class_node.constant_path()));

    if let Some(superclass) = class_node.superclass() {
        ps.emit_ident(" < ".to_string());
        ps.with_start_of_line(false, |ps| {
            format_node(ps, superclass);
        });
    }

    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.with_formatting_context(FormattingContext::ClassOrModule, |ps| {
                ps.emit_newline();
                if let Some(body) = class_node.body() {
                    format_node(ps, body);
                }
            });
        })
    });

    ps.with_start_of_line(true, |ps| ps.emit_end());
}

fn format_class_variable_and_write_node(
    ps: &mut ParserState,
    class_variable_and_write_node: prism::ClassVariableAndWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(class_variable_and_write_node.name()),
        Cow::Borrowed("&&="),
        class_variable_and_write_node.value(),
    );
}

fn format_class_variable_operator_write_node(
    ps: &mut ParserState,
    class_variable_operator_write_node: prism::ClassVariableOperatorWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(class_variable_operator_write_node.name()),
        Cow::Owned(loc_to_string(
            class_variable_operator_write_node.binary_operator_loc(),
        )),
        class_variable_operator_write_node.value(),
    );
}

fn format_class_variable_or_write_node(
    ps: &mut ParserState,
    class_variable_or_write_node: prism::ClassVariableOrWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(class_variable_or_write_node.name()),
        Cow::Borrowed("||="),
        class_variable_or_write_node.value(),
    );
}

fn format_class_variable_read_node(
    ps: &mut ParserState,
    class_variable_read_node: prism::ClassVariableReadNode,
) {
    ps.emit_ident(const_to_string(class_variable_read_node.name()));
}

fn format_class_variable_target_node(
    ps: &mut ParserState,
    class_variable_target_node: prism::ClassVariableTargetNode,
) {
    ps.emit_ident(const_to_string(class_variable_target_node.name()));
}

fn format_class_variable_write_node(
    ps: &mut ParserState,
    class_variable_write_node: prism::ClassVariableWriteNode,
) {
    ps.at_offset(class_variable_write_node.location().start_offset());
    format_write_node(
        ps,
        const_to_string(class_variable_write_node.name()),
        Cow::Borrowed("="),
        class_variable_write_node.value(),
    );
}

fn format_module_node(ps: &mut ParserState, module_node: prism::ModuleNode) {
    ps.emit_module_keyword();
    ps.emit_space();
    ps.with_start_of_line(false, |ps| format_node(ps, module_node.constant_path()));

    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.with_formatting_context(FormattingContext::ClassOrModule, |ps| {
                ps.emit_newline();
                if let Some(body) = module_node.body() {
                    format_node(ps, body);
                }
            });
        })
    });

    ps.with_start_of_line(true, |ps| {
        ps.emit_end();
    });
}

fn format_def_node(ps: &mut ParserState, def_node: prism::DefNode) {
    ps.emit_def_keyword();
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        if let Some(receiver) = def_node.receiver() {
            format_node(ps, receiver);
            ps.emit_dot();
        }

        handle_string_at_offset(
            ps,
            const_to_string(def_node.name()),
            def_node.name_loc().end_offset(),
        );
    });

    format_def_body(ps, def_node);
}

fn format_def_body(ps: &mut ParserState, def_node: prism::DefNode) {
    ps.new_scope(|ps| {
        if let Some(parameters_node) = def_node.parameters() {
            ps.breakable_of(
                BreakableDelims::for_method_call(),
                |ps| {
                    ps.with_start_of_line(
                        false,
                        |ps| {
                            format_parameters_node(ps, parameters_node);
                            // If the parameters have parens, wind to the closing paren, since it may
                            // be on its own line past the end of the params
                            if let Some(rparen_loc) = def_node.rparen_loc() {
                                ps.at_offset(rparen_loc.end_offset());
                            }
                        },
                    );
                },
            );
        }

        ps.with_formatting_context(
            FormattingContext::Def,
            |ps| {
                if def_node.end_keyword_loc().is_some() {
                    ps.new_block(|ps| {
                        ps.emit_newline();
                        ps.with_start_of_line(
                            true,
                            |ps| {
                                if let Some(body) = def_node.body() {
                                    // Begin nodes are special because they could be "implicit" begins,
                                    // e.g. `def foo; rescue Foo; end`, which aren't indented the same way
                                    // as other nodes and thus shouldn't go through the usual `format_node` machinery
                                    // that handles indentation and newlines
                                    if let Some(begin_node) = body.as_begin_node() {
                                        if begin_node.begin_keyword_loc().is_none() {
                                            format_begin_node(ps, begin_node);
                                        } else {
                                            format_node(ps, body);
                                        }
                                    } else {
                                        format_node(ps, body);
                                    }
                                }
                            },
                        );
                    });
                } else {
                    ps.emit_space();
                    ps.emit_op(Cow::Borrowed("="));
                    ps.emit_space();

                    ps.with_start_of_line(
                        false,
                        |ps| {
                            if let Some(body) = def_node.body() {
                                let mut body_node_list = body.as_statements_node()
                                    .expect("Endless methods must have a body, and method definitions are always a Statements node")
                                    .body()
                                    .iter();
                                let body_expression = body_node_list.next().expect("Endless methods must have exactly one expression in their body");

                                debug_assert!(body_node_list.next().is_none(), "Expected endless method body to have exactly one node.");

                                format_node(ps, body_expression);
                            }
                        },
                    )
                }
            },
        );
    });

    if let Some(end_keyword_loc) = def_node.end_keyword_loc() {
        ps.with_start_of_line(true, |ps| {
            ps.wind_dumping_comments_until_offset(end_keyword_loc.end_offset());
            ps.emit_end();
        });
    }
}

fn format_defined_node(ps: &mut ParserState, defined_node: prism::DefinedNode) {
    ps.with_start_of_line(false, |ps| {
        ps.emit_ident("defined?(".to_string());
        format_node(ps, defined_node.value());
        ps.emit_close_paren();
    });
}

fn format_else_node(ps: &mut ParserState, else_node: prism::ElseNode) {
    // Double check that these offsets are correct, since begin/rescue/ensure/else
    // aren't always handled with `format_node`, which usually handles this
    ps.at_offset(else_node.location().start_offset());

    // `else_keyword_loc` is somewhat misleading, since this can be either the `else`
    // keyword or the `:` separator in a ternary
    // Given those two options, we can check the length to avoid copying the loc.
    let else_keyword_loc = else_node.else_keyword_loc();
    let keyword_len = else_keyword_loc.end_offset() - else_keyword_loc.start_offset();
    if keyword_len == 4 {
        ps.emit_else();

        ps.new_block(|ps| {
            ps.emit_newline();
            if let Some(statements) = else_node.statements() {
                format_node(ps, statements.as_node())
            }
        });
    } else {
        // In a ternary
        ps.emit_space();
        ps.emit_conditional_keyword(":");
        ps.emit_space();
        ps.with_start_of_line(false, |ps| {
            format_node(
                ps,
                else_node
                    .statements()
                    .expect("Statements must be present in a ternary")
                    .body()
                    .iter()
                    .next()
                    .expect("Ternaries cannot have multiple statements"),
            );
        });
    }

    ps.at_offset(else_node.location().end_offset());
}

fn format_parameters_node(ps: &mut ParserState, params: prism::ParametersNode) {
    let non_null_positions = non_null_positions(&params);

    //def foo(a, b=nil, *args, d, e:, **kwargs, &blk)
    //        ^  ^___^  ^___^  ^  ^    ^_____^   ^
    //        |    |      |    |  |      |       |
    //        |    |      |    |  |      |     block
    //        |    |      |    |  |      |
    //        |    |      |    |  |  keyword_rest
    //        |    |      |    |  |
    //        |    |      |    | keywords
    //        |    |      |    |
    //        |    |      |  posts
    //        |    |      |
    //        |    |     rest
    //        |    |
    //        | optionals
    //        |
    //    requireds
    let formats: &'static [fn(&mut ParserState, prism::ParametersNode)] = &[
        fmt_requireds,
        fmt_optionals,
        fmt_rest,
        fmt_posts,
        fmt_keywords,
        fmt_keyword_rest,
        fmt_block,
    ];

    for (idx, format_fn) in formats.iter().enumerate() {
        format_fn(ps, params.as_node().as_parameters_node().unwrap());
        let did_emit = non_null_positions[idx];
        let have_more = non_null_positions[idx + 1..].iter().any(|&v| v);

        if did_emit && have_more {
            ps.emit_comma();
            ps.emit_soft_newline();
        }
        ps.shift_comments();
    }

    fn fmt_requireds(ps: &mut ParserState, params: prism::ParametersNode) {
        let requireds = params.requireds();
        if requireds.is_empty() {
            return;
        }
        let end_offset = requireds.iter().last().unwrap().location().end_offset();
        format_list_like_thing(ps, requireds, end_offset, false);
    }
    fn fmt_optionals(ps: &mut ParserState, params: prism::ParametersNode) {
        let optionals = params.optionals();
        if optionals.is_empty() {
            return;
        }
        let end_offset = optionals.iter().last().unwrap().location().end_offset();
        format_list_like_thing(ps, optionals, end_offset, false);
    }
    fn fmt_rest(ps: &mut ParserState, params: prism::ParametersNode) {
        let rest = params.rest();
        if let Some(rest) = rest {
            format_node(ps, rest);
        }
    }
    fn fmt_posts(ps: &mut ParserState, params: prism::ParametersNode) {
        let posts = params.posts();
        if posts.is_empty() {
            return;
        }
        let end_offset = posts.iter().last().unwrap().location().end_offset();
        format_list_like_thing(ps, posts, end_offset, false);
    }
    fn fmt_keywords(ps: &mut ParserState, params: prism::ParametersNode) {
        let keywords = params.keywords();
        if keywords.is_empty() {
            return;
        }
        let end_offset = keywords.iter().last().unwrap().location().end_offset();
        format_list_like_thing(ps, keywords, end_offset, false);
    }
    fn fmt_keyword_rest(ps: &mut ParserState, params: prism::ParametersNode) {
        let keyword_rest = params.keyword_rest();
        if let Some(keyword_rest) = keyword_rest {
            format_node(ps, keyword_rest);
        }
    }
    fn fmt_block(ps: &mut ParserState, params: prism::ParametersNode) {
        let block = params.block();
        if let Some(block) = block {
            format_block_parameter_node(ps, block);
        }
    }
}

fn format_block_parameter_node(ps: &mut ParserState, block_arg: prism::BlockParameterNode) {
    ps.with_start_of_line(false, |ps| {
        ps.emit_soft_indent();
        ps.emit_ident("&".to_string());
        if let Some(ident) = block_arg.name() {
            let ident_str = const_to_string(ident);
            ps.bind_variable(ident_str.clone());
            format_ident(ps, ident_str, block_arg.name_loc().unwrap().end_offset());
        }
    });
}

fn format_block_argument_node(ps: &mut ParserState, block_argument_node: prism::BlockArgumentNode) {
    ps.emit_ident("&".to_string());
    if let Some(expression_node) = block_argument_node.expression() {
        ps.with_start_of_line(false, |ps| {
            format_node(ps, expression_node);
        });
    }
}

pub static RSPEC_METHODS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| vec!["it", "describe"].into_iter().collect());

pub static GEMFILE_METHODS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| vec!["gem", "source", "ruby", "group"].into_iter().collect());

pub static OPTIONALLY_PARENTHESIZED_METHODS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| {
        vec!["super", "require", "require_relative"]
            .into_iter()
            .collect::<HashSet<_>>()
    });

fn use_parens_for_call_node(
    ps: &ParserState,
    call_node: &prism::CallNode,
    method_name: &str,
    // Whether we're the final call in a chain, e.g.
    // foo.bar.baz
    //         ^ terminal call
    is_terminal_call: bool,
    context: FormattingContext,
) -> bool {
    let original_used_parens = call_node.opening_loc().is_some();

    // If the calling method is a const, the parens become
    // semantically important, e.g.
    // ```
    // class Foo; end
    // def Foo; end
    // Foo # class reference
    // Foo() # method call
    // ```
    if is_terminal_call && method_name.chars().next().is_some_and(|c| c.is_uppercase()) {
        return true;
    }

    if method_name.starts_with("attr_") && context == FormattingContext::ClassOrModule {
        return original_used_parens;
    }

    if ps.scope_has_variable(method_name) {
        if call_node.receiver().is_none() {
            return original_used_parens;
        } else if let Some(receiver) = call_node.receiver()
            && receiver.as_self_node().is_some()
        {
            return true;
        }
    }

    if method_name == "raise" {
        if ps.current_formatting_context_requires_parens() {
            return true;
        }

        if let Some(arguments) = call_node.arguments()
            && arguments
                .arguments()
                .iter()
                .any(|arg| arg.as_splat_node().is_some())
        {
            return true;
        }
        return false;
    }

    if OPTIONALLY_PARENTHESIZED_METHODS.contains(method_name)
        || GEMFILE_METHODS.contains(method_name)
    {
        return original_used_parens;
    }

    let has_arguments = call_node
        .arguments()
        .map(|args| {
            !(args.arguments().is_empty()
                || (args.arguments().len() == 1
                    && is_empty_parentheses_node(&args.arguments().iter().next().unwrap())))
        })
        .unwrap_or(false);

    if !has_arguments {
        return false;
    }

    let has_brace_block = call_node
        .block()
        .and_then(|b| b.as_block_node())
        .map(|block| loc_to_str(block.opening_loc()) != "do")
        .unwrap_or(false);

    if has_arguments && has_brace_block {
        // Brace blocks require parens, eliding is a syntax error
        return true;
    }

    if RSPEC_METHODS.contains(method_name) && call_node.receiver().is_none() {
        return false;
    }

    // Check for `RSpec.describe`
    if let Some(receiver) = call_node.receiver()
        && let Some(const_read) = receiver.as_constant_read_node()
    {
        let const_name = const_to_str(const_read.name());
        if const_name == "RSpec" && method_name == "describe" {
            return false;
        }
    }

    if context == FormattingContext::ClassOrModule && !original_used_parens {
        return false;
    }

    true
}

fn format_call_node(
    ps: &mut ParserState,
    call_node: prism::CallNode,
    skip_receiver: bool,
    is_final_call_in_chain: bool,
) {
    let method_name = const_to_string(call_node.name());
    let end_offset = call_node.location().end_offset();
    let is_dot_call = &method_name == "call" && call_node.message_loc().is_none(); // e.g. `a.()`

    // Only treat [] and []= as aref syntax when there's no explicit call operator.
    let has_call_operator = call_node.call_operator_loc().is_some();
    let is_aref = &method_name == "[]" && !has_call_operator;
    let is_aref_write = &method_name == "[]=" && !has_call_operator;

    if skip_receiver || call_node.receiver().is_none() {
        if !is_aref && !is_aref_write {
            let method_ident = if call_node.is_attribute_write() {
                loc_to_string(
                    call_node
                        .message_loc()
                        .expect("Attribute writes must have a message"),
                )
            } else if is_dot_call {
                String::new()
            } else {
                method_name.clone()
            };
            handle_string_at_offset(
                ps,
                method_ident,
                start_loc_for_call_node_in_chain(&call_node),
            );
        }

        // Calls with only empty parens (`foo ()`) are treated as calls without args
        let has_only_empty_paren_arg = !is_aref
            && !is_aref_write
            && call_node.arguments().is_some_and(|args| {
                args.arguments().len() == 1
                    && is_empty_parentheses_node(&args.arguments().iter().next().unwrap())
            });

        if let Some(arguments) = call_node.arguments()
            && !has_only_empty_paren_arg
        {
            // For callers where the only arg is a def node,
            // we assume that's a `public def` style modifier and don't use parens
            if arguments.arguments().len() == 1
                && arguments
                    .arguments()
                    .iter()
                    .next()
                    .unwrap()
                    .as_def_node()
                    .is_some()
            {
                ps.emit_space();

                let def_node = arguments
                    .arguments()
                    .iter()
                    .next()
                    .unwrap()
                    .as_def_node()
                    .unwrap();
                format_def_node(ps, def_node);
            } else if is_aref_write {
                let arg_count = arguments.arguments().len();

                ps.with_start_of_line(false, |ps| {
                    ps.breakable_of(BreakableDelims::for_array(), |ps| {
                        // All arguments except the last are index arguments
                        for (i, arg) in arguments.arguments().iter().take(arg_count - 1).enumerate()
                        {
                            if i > 0 {
                                ps.emit_comma();
                                ps.emit_soft_newline();
                            }
                            ps.emit_soft_indent();
                            format_node(ps, arg);
                        }
                    });
                });

                ps.emit_ident(" = ".to_string());

                let last_arg =
                    arguments.arguments().iter().last().expect(
                        "The last argument is the value being assigned and must be present",
                    );
                ps.with_start_of_line(false, |ps| format_node(ps, last_arg));
            } else if call_node.is_attribute_write() {
                ps.emit_ident(" = ".to_string());
                let value = arguments
                    .arguments()
                    .iter()
                    .next()
                    .expect("Attribute writes must have a value");
                ps.with_start_of_line(false, |ps| format_node(ps, value));
            } else {
                let should_use_parens = use_parens_for_call_node(
                    ps,
                    &call_node,
                    &method_name,
                    is_final_call_in_chain,
                    ps.current_formatting_context(),
                );

                let delims = if is_aref {
                    BreakableDelims::for_array()
                } else if should_use_parens {
                    BreakableDelims::for_method_call()
                } else {
                    BreakableDelims::for_kw()
                };

                let maybe_closing_line = call_node
                    .closing_loc()
                    .map(|closing_loc| ps.get_line_number_for_offset(closing_loc.start_offset()));

                ps.with_start_of_line(false, |ps| {
                    ps.breakable_of(delims, |ps| {
                        let has_arguments = !arguments.arguments().is_empty();

                        format_arguments_node(ps, arguments);

                        // Somewhat confusingly, the block argument node (&blk) is
                        // separate from the rest of the arguments node. If it's present,
                        // we want it to be a part of the comma-separated list
                        if let Some(block_argument_node) = call_node
                            .block()
                            .and_then(|block_node| block_node.as_block_argument_node())
                        {
                            if has_arguments {
                                ps.emit_comma();
                                ps.emit_soft_newline();
                                ps.emit_soft_indent();
                            }
                            format_block_argument_node(ps, block_argument_node);
                        }

                        // Ensure that we render comments between the last argument and
                        // closing parens. We wind to one line before the closing paren
                        // to pick up comments that are inside the args but not trailing
                        // comments on the same line as the closing delim (like `) # comment`).
                        if let Some(closing_line) = maybe_closing_line {
                            ps.wind_dumping_comments_until_line(closing_line - 1);
                        }
                    });
                });

                if let Some(closing_line) = maybe_closing_line {
                    ps.on_line(closing_line);
                }
            };
        } else if is_aref {
            // For a[] or a[]= with no arguments, we still need to emit the brackets
            ps.emit_open_square_bracket();
            ps.emit_close_square_bracket();
        } else if !has_only_empty_paren_arg
            && (!skip_receiver
                || call_node
                    .receiver()
                    .map(|r| r.as_self_node().is_some())
                    .unwrap_or(false))
        {
            // Check if we need parens in two cases: we're the
            // first/only item in a call chain (thus `!skip_receiver`)
            // or we're in a chain but the receiver is `self`
            let should_use_parens = use_parens_for_call_node(
                ps,
                &call_node,
                &method_name,
                is_final_call_in_chain,
                ps.current_formatting_context(),
            );

            if should_use_parens {
                ps.emit_open_paren();
                ps.emit_close_paren();
            }
        } else {
            // There's no arguments, but we may still need parens
            let should_use_parens = is_dot_call
                || use_parens_for_call_node(
                    ps,
                    &call_node,
                    &method_name,
                    is_final_call_in_chain,
                    ps.current_formatting_context(),
                );
            if should_use_parens {
                ps.emit_open_paren();
                ps.emit_close_paren();
            }
        };

        if let Some(block) = call_node.block() {
            if block.as_block_argument_node().is_none() {
                ps.emit_space();
                ps.with_start_of_line(false, |ps| {
                    format_node(ps, block);
                });
                // Only emit this when we're not inside a call chain (`skip_receiver` is false),
                // since call chains emit AfterCallChain after the entire chain is done.
                if !skip_receiver {
                    ps.emit_after_call_chain();
                }
            // If there's an arguments node, we've handled this block arg with
            // the rest of the args (since it's included in the comma-separated
            // args list), otherwise the only argument is the &blk node, so we
            // have to handle that here separately
            } else if call_node.arguments().is_none() && block.as_block_argument_node().is_some() {
                ps.breakable_of(BreakableDelims::for_method_call(), |ps| {
                    ps.emit_soft_indent();
                    format_block_argument_node(ps, block.as_block_argument_node().unwrap());
                });
            }
        }
    } else {
        let is_unary_operator = call_node.arguments().is_none()
            && call_node.call_operator_loc().is_none()
            && matches!(method_name.as_str(), "-@" | "+@" | "!" | "~");

        if is_unary_operator {
            format_unary_operator(ps, call_node, method_name);
        } else {
            // Note: infix operators *can* be called with dots, e.g. `1.<=(2)`,
            // but in those cases we render them as regular method calls.
            let is_infix_operator =
                !is_aref && !is_aref_write && call_node.call_operator_loc().is_none();

            if is_infix_operator {
                ps.inline_breakable_of(BreakableDelims::for_binary_op(), |ps| {
                    format_infix_operator(
                        ps,
                        call_node.receiver().unwrap(),
                        method_name,
                        // For infix operators, we still get an ArgumentsNode, but it will
                        // always be an argument list of a single node.
                        call_node
                            .arguments()
                            .unwrap()
                            .arguments()
                            .iter()
                            .next()
                            .unwrap(),
                    );
                });
            } else {
                format_call_chain(ps, call_node);
            }
            ps.emit_after_call_chain();
        }
    }
    // We've been manually handling line winding while rendering the chain,
    // so we need to manually check that we wind to the closing loc
    ps.wind_dumping_comments_until_offset(end_offset);
}

fn format_unary_operator(ps: &mut ParserState, call_node: prism::CallNode, method_name: String) {
    let operator_symbol = match method_name.as_str() {
        "!" => {
            // `not` and `!` both have a `name` of `!` but different messages
            if let Some(message_loc) = call_node.message_loc() {
                let message_text = loc_to_string(message_loc);
                if message_text == "not" {
                    "not ".to_string()
                } else {
                    "!".to_string()
                }
            } else {
                "!".to_string()
            }
        }
        "-@" => "-".to_string(),
        "+@" => "+".to_string(),
        "~" => "~".to_string(),
        _ => {
            if cfg!(debug_assertions) {
                unreachable!("Received unexpected unary operator: {}", method_name);
            }

            // Try to render the message loc as a fallback in unexpected cases, but
            // panic if we don't find one, otherwise we're rendering a total guess.
            loc_to_string(
                call_node
                    .message_loc()
                    .expect("Expected unary operator to have a message loc"),
            )
        }
    };

    ps.with_start_of_line(false, |ps| {
        ps.emit_ident(operator_symbol);
        format_node(
            ps,
            call_node
                .receiver()
                .expect("Unary operators must have a receiver"),
        );
    });
}

fn format_infix_operator(
    ps: &mut ParserState,
    left: prism::Node,
    operator: String,
    right: prism::Node,
) {
    ps.with_formatting_context(FormattingContext::Binary, |ps| {
        ps.with_start_of_line(false, |ps| {
            // Check if left and right are also binary operators so we recurse back and handle it here.
            // This is so that chained binary operations get indented correctly as one big chain.
            // ```ruby
            // foo &&
            //   bar &&
            //   baz
            // ```
            if let Some((inner_left, inner_op, inner_right)) = as_binary_op(&left) {
                format_infix_operator(ps, inner_left, inner_op, inner_right);
            } else {
                ps.dedent(|ps| format_node(ps, left));
            }

            let comparison_operators = [">", ">=", "===", "==", "<", "<=", "<=>", "!="];
            let is_comparison = comparison_operators.iter().any(|o| o == &operator);

            ps.emit_space();
            ps.emit_ident(operator);

            if is_comparison {
                // For comparison operators, we always put the right-hand side
                // on the same line as the left-hand side.
                ps.emit_space();
            } else {
                ps.emit_soft_newline();
                ps.emit_soft_indent();
            }
            ps.reset_space_count();

            if let Some((inner_left, inner_op, inner_right)) = as_binary_op(&right) {
                format_infix_operator(ps, inner_left, inner_op, inner_right);
            } else {
                format_node(ps, right);
            }
        });
    });
}

/// Check if a node is a binary operator (and/or nodes, or infix CallNodes like `+`, `-`, etc.)
/// and return its components (left, operator, right) if so.
fn as_binary_op<'a>(
    node: &'a prism::Node<'a>,
) -> Option<(prism::Node<'a>, String, prism::Node<'a>)> {
    if let Some(and_node) = node.as_and_node() {
        return Some((
            and_node.left(),
            loc_to_string(and_node.operator_loc()),
            and_node.right(),
        ));
    }

    if let Some(or_node) = node.as_or_node() {
        return Some((
            or_node.left(),
            loc_to_string(or_node.operator_loc()),
            or_node.right(),
        ));
    }

    let call_node = node.as_call_node()?;

    if call_node.call_operator_loc().is_some() {
        return None;
    }

    let left = call_node.receiver()?;

    let arguments = call_node.arguments()?.arguments();
    if arguments.len() != 1 {
        return None;
    }

    let right = arguments.iter().next().unwrap();
    let method_name = const_to_string(call_node.name());

    if method_name == "[]" || method_name == "[]=" {
        return None;
    }

    Some((left, method_name, right))
}

fn format_call_chain(ps: &mut ParserState, call_node: ruby_prism::CallNode<'_>) {
    ps.with_start_of_line(false, |ps| {
        let segments = split_node_into_call_chains(call_node.as_node());

        let is_attribute_write = segments
            .last()
            .and_then(|seg| seg.last())
            .and_then(|n| n.as_call_node())
            .map(|c| c.is_attribute_write())
            .unwrap_or(false);

        if is_attribute_write {
            if cfg!(debug_assertions) {
                assert!(
                    segments.len() == 1,
                    "attribute_write call chain had multiple segments"
                );
            }
            let chain = segments.into_iter().next().unwrap();
            format_call_body(ps, chain, true, false);
        } else {
            format_call_chain_segments(ps, segments);
        }
    });
}

fn format_call_chain_segments(ps: &mut ParserState, mut segments: Vec<Vec<prism::Node>>) {
    if let Some(current) = segments.pop() {
        let has_inner = !segments.is_empty();

        let (chain_elements, trailing_arefs) = extract_trailing_arefs(current);

        let is_user_multilined = call_chain_elements_are_user_multilined(ps, &chain_elements);

        ps.breakable_call_chain_of(MultilineHandling::Prism(is_user_multilined), |ps| {
            // Recurse and format previous segments inside this breakable
            format_call_chain_segments(ps, segments);

            format_call_body(ps, chain_elements, false, has_inner);
        });

        // Trailing arefs are formatted after the breakable
        for aref in trailing_arefs {
            let aref = aref.as_call_node().unwrap();
            format_call_node(ps, aref, true, false);
        }
    }
}

fn extract_trailing_arefs(mut elements: Vec<prism::Node>) -> (Vec<prism::Node>, Vec<prism::Node>) {
    let mut trailing_arefs = Vec::new();
    let has_dot_calls = elements.iter().any(|elem| {
        elem.as_call_node()
            .map(|c| c.call_operator_loc().is_some())
            .unwrap_or(false)
    });

    if has_dot_calls {
        while let Some(last_node) = elements.pop_if(|last_node| {
            last_node
                .as_call_node()
                .is_some_and(|call_node| call_node.call_operator_loc().is_none())
        }) {
            trailing_arefs.insert(0, last_node);
        }
    }

    (elements, trailing_arefs)
}

fn format_call_body(
    ps: &mut ParserState,
    mut call_chain_elements: Vec<prism::Node>,
    is_attr_write: bool,
    is_continuation: bool,
) {
    if call_chain_elements.is_empty() {
        return;
    }

    if !is_continuation {
        // The first node can be *any* expression, whereas following receivers
        // must be additional calls -- you cannot insert literals into call chains
        let first_expression = call_chain_elements.remove(0);
        format_node(ps, first_expression);

        // Arefs like `Array[1, 2]` are represented as a call chain where
        // the receiver is the constant name `Array` and the aref is a separate call node.
        // However, we don't want to start the call chain indent until after the aref, since the
        // aref is "part of" the first expression.
        // We loop here to handle chained arefs like `matrix[0][1].foo`.
        while let Some(next) = call_chain_elements.first() {
            if let Some(call_node) = next.as_call_node()
                && call_node.call_operator_loc().is_none()
            {
                call_chain_elements.remove(0);
                format_call_node(ps, call_node, true, false);
                continue;
            }
            break;
        }

        // Eagerly render heredocs if they're in the first expression.
        // We want the full heredoc to get rendered _before_ we emit the
        // BeginCallChainIndent token so that it gets correctly indented
        // (or in the case of it being the first expression, _not_ indented).
        ps.render_heredocs(true);
    }

    // Attribute writes like `self.foo = bar` should have both sides
    // rendered separately so they can break independently
    if !is_continuation
        && call_chain_elements.len() == 1
        && let Some(attr_write) = call_chain_elements
            .first()
            .and_then(|n| n.as_call_node())
            .filter(|c| c.is_attribute_write())
    {
        // Format `.attr_name = ` directly without call chain indent
        let call_operator = attr_write.call_operator_loc().map(|loc| loc_to_str(loc));
        if let Some(call_operator) = call_operator {
            match call_operator {
                "." => ps.emit_dot(),
                "&." => ps.emit_lonely_operator(),
                "::" => ps.emit_colon_colon(),
                _ => ps.emit_ident(call_operator.to_string()),
            }
        }

        ps.at_offset(start_loc_for_call_node_in_chain(&attr_write));
        ps.shift_comments();

        // Format just the attribute name and ` = `, then the value separately
        format_call_node(ps, attr_write, true, true);
    } else if !call_chain_elements.is_empty() {
        // attr_writes are not wrapped in a breakable, so they
        // cannot emit abstract token types
        if !is_attr_write {
            ps.start_indent_for_call_chain();
        }

        ps.with_start_of_line(false, |ps| {
            let call_chain_element_count = call_chain_elements.len();
            for (idx, element) in call_chain_elements.iter().enumerate() {
                let element = element.as_call_node().unwrap();
                let is_final_call = idx == call_chain_element_count - 1;

                // `call_operator_loc` is the `.`/`::`/`&.` etc.
                // it may be None in the case of arefs, e.g. foo[bar]
                let call_operator = element.call_operator_loc().map(|loc| loc_to_str(loc));
                if let Some(call_operator) = call_operator {
                    if call_operator != "::" && !is_attr_write {
                        ps.emit_collapsing_newline();
                        ps.emit_soft_indent();
                    }

                    // Emit the proper token type so that call_count is computed correctly
                    // in single_line_string_length (which is used to determine whether to
                    // break the call chain or just the arguments)
                    match call_operator {
                        "." => ps.emit_dot(),
                        "&." => ps.emit_lonely_operator(),
                        "::" => ps.emit_colon_colon(),
                        _ => ps.emit_ident(call_operator.to_string()),
                    }
                }

                ps.at_offset(start_loc_for_call_node_in_chain(&element));
                ps.shift_comments();

                format_call_node(ps, element, true, is_final_call);
            }
        });
        if !is_attr_write {
            ps.end_indent_for_call_chain();
        }
    }
}

fn call_chain_elements_are_user_multilined(
    ps: &ParserState,
    call_chain_elements: &[prism::Node],
) -> bool {
    // Making a mutable copy since we may pop some items off later
    let mut call_chain_elements = call_chain_elements;

    if call_chain_elements.len() > 1 {
        // If the first item in the chain is a multiline expression (like a hash or array),
        // ignore it when checking line length.
        let is_literal_expression = !matches!(
            call_chain_elements.first().unwrap(),
            prism::Node::CallNode { .. }
                | prism::Node::ConstantReadNode { .. }
                | prism::Node::ConstantPathNode { .. }
                | prism::Node::LocalVariableReadNode { .. }
                | prism::Node::GlobalVariableReadNode { .. }
                | prism::Node::InstanceVariableReadNode { .. }
                | prism::Node::ClassVariableReadNode { .. }
                | prism::Node::ItLocalVariableReadNode { .. }
                | prism::Node::NumberedReferenceReadNode { .. }
                | prism::Node::ParenthesesNode { .. }
        );

        // _However_, don't ignore this if there are comments in the call chain though; this check may
        // cause it to single-lined, which breaks comment rendering. Specifically, we're checking
        // for comments in between the receiver expression and the following message, e.g.
        // ```ruby
        // [stuff]
        //   # spooky comment
        //   .freeze
        // ```
        // For cases without the comment, we'd usually put this all on one line, but if we force
        // it all on one line, this will break the comment insertion logic, and given the comment's
        // placement, the user probably intended to break this onto multiple lines anyways.
        let has_comment = ps.has_comment_in_offset_span(
            call_chain_elements[0].location().end_offset(),
            start_loc_for_call_node_in_chain(&call_chain_elements[1].as_call_node().unwrap()),
        );
        if is_literal_expression && !has_comment {
            call_chain_elements = &call_chain_elements[1..];
        }
    }

    // The first element can be any expression (constant, local var, etc.), so we need
    // to first skip it if it's not a CallNode and there's an aref following it.
    let first_is_not_call = call_chain_elements
        .first()
        .map(|n| n.as_call_node().is_none())
        .unwrap_or(false);
    let second_is_aref = call_chain_elements.get(1).is_some_and(|n| {
        n.as_call_node()
            .map(|c| c.call_operator_loc().is_none())
            .unwrap_or(false)
    });

    if first_is_not_call && second_is_aref {
        call_chain_elements = &call_chain_elements[1..];
        while call_chain_elements.len() > 1
            && let Some(call_node) = call_chain_elements.first().and_then(|n| n.as_call_node())
        {
            // Pop all leading arefs
            if call_node.call_operator_loc().is_none() {
                call_chain_elements = &call_chain_elements[1..];
                continue;
            }
            break;
        }
    }

    let start_line = {
        let start_node = call_chain_elements.first().unwrap();
        if let Some(call_node) = start_node.as_call_node() {
            ps.get_line_number_for_offset(start_loc_for_call_node_in_chain(&call_node))
        } else {
            ps.get_line_number_for_offset(start_node.location().start_offset())
        }
    };

    call_chain_elements[1..].iter().any(|cce| {
        start_line
            != cce
                .as_call_node()
                .unwrap()
                .call_operator_loc()
                .map(|loc| ps.get_line_number_for_offset(loc.start_offset()))
                .unwrap_or(start_line)
    })
}

/// Finds an appropriate starting loc for for a call node inside a call chain.
/// In the middle of a chain, the node's `location().start_offset()` is always the
/// beginning of the chain, since `receiver()` is the entirety of the chain so far.
/// To find the loc in the middle of the chain, we need to use something else to approximate that,
/// which in this case is either the name of the method or, in the case of method calls without
/// names (e.g. `.()`), we use the call operator loc.
fn start_loc_for_call_node_in_chain(call_chain_element: &prism::CallNode<'_>) -> usize {
    call_chain_element
        .message_loc()
        .unwrap_or_else(|| {
            call_chain_element.call_operator_loc().expect(
                "If we're in a call chain and there's no message loc, we must be in a dot-call (`.()`), so there must be a call operator loc",
            )
        })
        .start_offset()
}

fn format_call_and_write_node(ps: &mut ParserState, call_and_write_node: prism::CallAndWriteNode) {
    if let Some(receiver) = call_and_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(call_operator_loc) = call_and_write_node.call_operator_loc() {
        ps.emit_ident(loc_to_string(call_operator_loc));
    }

    if let Some(message_loc) = call_and_write_node.message_loc() {
        ps.emit_ident(loc_to_string(message_loc));
    }

    ps.emit_space();
    ps.emit_op(Cow::Borrowed("&&="));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| format_node(ps, call_and_write_node.value()));
}

fn format_call_operator_write_node(
    ps: &mut ParserState,
    call_operator_write_node: prism::CallOperatorWriteNode,
) {
    if let Some(receiver) = call_operator_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(call_operator_loc) = call_operator_write_node.call_operator_loc() {
        ps.emit_ident(loc_to_string(call_operator_loc));
    }

    if let Some(message_loc) = call_operator_write_node.message_loc() {
        ps.emit_ident(loc_to_string(message_loc));
    }

    ps.emit_space();
    ps.emit_op(Cow::Owned(loc_to_string(
        call_operator_write_node.binary_operator_loc(),
    )));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        format_node(ps, call_operator_write_node.value())
    });
}

fn format_call_or_write_node(ps: &mut ParserState, call_or_write_node: prism::CallOrWriteNode) {
    if let Some(receiver) = call_or_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(call_operator_loc) = call_or_write_node.call_operator_loc() {
        ps.emit_ident(loc_to_string(call_operator_loc));
    }

    if let Some(message_loc) = call_or_write_node.message_loc() {
        ps.emit_ident(loc_to_string(message_loc));
    }

    ps.emit_space();
    ps.emit_op(Cow::Borrowed("||="));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| format_node(ps, call_or_write_node.value()));
}

fn format_call_target_node(ps: &mut ParserState, call_target_node: prism::CallTargetNode) {
    ps.with_start_of_line(false, |ps| format_node(ps, call_target_node.receiver()));
    ps.emit_ident(loc_to_string(call_target_node.call_operator_loc()));
    ps.emit_ident(loc_to_string(call_target_node.message_loc()));
}

fn format_symbol_node(ps: &mut ParserState, symbol_node: prism::SymbolNode) {
    let opener = symbol_node.opening_loc().map(|s| loc_to_string(s));
    let closer = symbol_node.closing_loc().map(|s| loc_to_string(s));

    // Check if this is a quoted symbol that needs normalization to double quotes
    // Symbols like :'"foo"' (single-quoted) should become :"\"foo\""
    let is_single_quoted = opener.as_ref().map(|s| s == ":'").unwrap_or(false);

    if is_single_quoted {
        ps.emit_ident(":".to_string());
        ps.emit_double_quote();

        if let Some(value_loc) = symbol_node.value_loc() {
            let content = loc_to_str(value_loc);
            let escaped = crate::string_escape::single_to_double_quoted(
                content,
                opener
                    .expect("We must have an opener to know we're single-quoted")
                    .as_str(),
                closer
                    .expect("We must have a closer when wrapped in single quotes")
                    .as_str(),
            );
            ps.emit_string_content(escaped);
        }

        ps.emit_double_quote();
    } else {
        // For other symbols, emit as-is
        if let Some(ref opening) = opener {
            ps.emit_ident(opening.clone());
        }
        if let Some(value_loc) = symbol_node.value_loc() {
            ps.emit_ident(loc_to_string(value_loc));
        }
        if let Some(closing_str) = closer {
            let mut closing_str = closing_str;
            // String symbols, such as the key in `{ "a": b }` will have
            // a closing_str of `"\":"`, so we have to trim instead of
            // dropping the entire closing item.
            if closing_str.ends_with(":") {
                closing_str.pop();
            }
            if !closing_str.is_empty() {
                ps.emit_ident(closing_str);
            }
        }
    }
}

fn format_assoc_node(ps: &mut ParserState, assoc_node: prism::AssocNode) {
    let as_symbol = if let Some(hash_type) = ps.hash_type_from_formatting_context() {
        matches!(hash_type, HashType::SymbolKey)
    } else {
        // `operator_loc` is only present for hash rockets, not for symbol keys
        assoc_node.operator_loc().is_none()
    };

    ps.with_start_of_line(false, |ps| {
        // Check if we're rendering a symbol key as a rocket,
        // in which case we need to add back the leading colon
        if !as_symbol && assoc_node.operator_loc().is_none() {
            ps.emit_ident(":".to_string());
        }

        format_node(ps, assoc_node.key());
        if as_symbol {
            ps.emit_ident(":".to_string());
        } else {
            ps.emit_space();
            ps.emit_ident("=>".to_string());
        }
        // For assoc nodes, skip the space so it renders as `{ a:, b:, c: }`
        if assoc_node.value().as_implicit_node().is_none() {
            ps.emit_space();
        }
        format_node(ps, assoc_node.value());
    });
}

fn format_assoc_splat_node(ps: &mut ParserState, assoc_splat_node: prism::AssocSplatNode) {
    ps.with_start_of_line(false, |ps| {
        ps.emit_ident("**".to_string());
        if let Some(value) = assoc_splat_node.value() {
            format_node(ps, value);
        }
    });
}

fn format_block_node(ps: &mut ParserState, block_node: prism::BlockNode) {
    if loc_to_str(block_node.opening_loc()) == "do" {
        ps.new_block(|ps| {
            ps.emit_do_keyword();
            if let Some(block_parameters) = block_node.parameters()
                // These node types are implicit types -- they don't represent anything
                // and have misleading locs that we don't want to use
                && block_parameters.as_numbered_parameters_node().is_none()
                && block_parameters.as_it_parameters_node().is_none()
            {
                format_node(ps, block_parameters);
            }

            // Even if there's no body, we still need a newline for
            // comments to render appropriately. `ps.emit_end` will handle
            // checking for this newline and deduping it appropriately.
            ps.emit_newline();

            if let Some(body) = block_node.body() {
                ps.with_start_of_line(true, |ps| {
                    format_node(ps, body);
                });
            }
        });

        ps.with_start_of_line(true, |ps| {
            ps.wind_dumping_comments_until_offset(block_node.location().end_offset());
            ps.emit_end();
            ps.shift_comments();
        });
    } else {
        ps.inline_breakable_of(BreakableDelims::for_brace_block(), |ps| {
            if let Some(parameters) = block_node.parameters()
                // These node types are implicit types -- they don't represent anything
                // and have misleading locs that we don't want to use
                && parameters.as_numbered_parameters_node().is_none()
                && parameters.as_it_parameters_node().is_none()
            {
                format_node(ps, parameters);
            }

            if let Some(body) = block_node.body() {
                let has_multiple_statements = body
                    .as_statements_node()
                    .map(|statements_node| statements_node.body().len() > 1)
                    .unwrap_or(false);
                if has_multiple_statements {
                    ps.emit_newline();
                    ps.emit_indent();
                    ps.with_start_of_line(false, |ps| {
                        let statements = body.as_statements_node().unwrap().body();
                        let mut peekable = statements.iter().peekable();
                        while peekable.peek().is_some() {
                            format_node(ps, peekable.next().unwrap());
                            ps.emit_soft_newline();
                            if peekable.peek().is_some() {
                                ps.emit_soft_indent();
                            }
                        }
                        ps.shift_comments();
                    });
                } else {
                    ps.with_start_of_line(false, |ps| {
                        if let Some(node) = body.as_statements_node().unwrap().body().iter().next()
                        {
                            ps.emit_soft_newline();
                            ps.emit_soft_indent();
                            format_node(ps, node);
                            ps.emit_soft_newline();
                        }
                    });
                }
            } else if ps.has_comment_in_offset_span(
                block_node.opening_loc().start_offset(),
                block_node.closing_loc().end_offset(),
            ) {
                // Even if there's no `body` node -- that is, there are no statements in the block --
                // we still need to look for comments and multiline if they're present.
                // Note that this is a soft newline, which are special-cased in breakables to correctly handle
                // comments, so we use one here instead of a hard newline.
                ps.emit_soft_newline();
            }

            // `inline_breakable_of` doesn't handle the indentation for the closing delimeter for us.
            ps.dedent(|ps| ps.emit_soft_indent());
            ps.wind_dumping_comments_until_offset(block_node.location().end_offset());
            ps.shift_comments();
        });
    }
}

fn format_block_parameters_node(
    ps: &mut ParserState,
    block_parameters_node: prism::BlockParametersNode,
) {
    // Exit early if there's no params
    if block_parameters_node.locals().is_empty() && block_parameters_node.parameters().is_none() {
        return;
    }

    ps.breakable_of(BreakableDelims::for_block_params(), |ps| {
        format_block_parameters_names(
            ps,
            block_parameters_node.locals(),
            block_parameters_node.parameters(),
            block_parameters_node.location().end_offset(),
        );
    });
}

fn format_block_parameters_names(
    ps: &mut ParserState,
    locals: prism::NodeList,
    parameters: Option<prism::ParametersNode>,
    end_offset: usize,
) {
    let has_locals = !locals.is_empty();

    if let Some(parameters) = parameters {
        format_parameters_node(ps, parameters);
    }
    if has_locals {
        ps.emit_ident(";".to_string());
        ps.with_start_of_line(false, |ps| {
            format_list_like_thing(ps, locals, end_offset, true);
        });
    }
}

fn format_block_local_variable_node(
    ps: &mut ParserState,
    block_local_variable_node: prism::BlockLocalVariableNode,
) {
    handle_string_at_offset(
        ps,
        const_to_string(block_local_variable_node.name()),
        block_local_variable_node.location().start_offset(),
    );
}

fn format_array_node(ps: &mut ParserState, array_node: prism::ArrayNode) {
    let opening = array_node
        .opening_loc()
        .map(|loc| loc_to_string(loc).trim().to_string());
    let is_word_array = opening
        .as_ref()
        .map(|s| s.starts_with("%"))
        .unwrap_or(false);

    let orig_delim = opening
        .as_ref()
        .and_then(|s| s.chars().nth(2))
        .unwrap_or('[');

    if is_word_array {
        ps.emit_ident(opening.unwrap().split_at(2).0.to_string());
    }

    if array_node.elements().is_empty() {
        if ps.has_comment_in_offset_span(
            array_node.location().start_offset(),
            array_node.location().end_offset(),
        ) {
            ps.with_start_of_line(false, |ps| {
                ps.breakable_of(BreakableDelims::for_array(), |ps| {
                    ps.wind_dumping_comments_until_offset(array_node.location().end_offset());
                })
            })
        } else {
            ps.emit_open_square_bracket();
            ps.emit_close_square_bracket();
        }
    } else if is_word_array {
        // For word arrays, preserve the original syntax
        ps.breakable_of(BreakableDelims::for_array(), |ps| {
            ps.with_start_of_line(false, |ps| {
                format_word_array_elements(
                    ps,
                    array_node.elements(),
                    array_node.location().end_offset(),
                    orig_delim,
                );
                ps.wind_dumping_comments_until_offset(array_node.location().end_offset());
            });
        });
    } else {
        ps.with_start_of_line(false, |ps| {
            if array_node.opening_loc().is_none() {
                // Array node is an implicit array, e.g. `a = 1, 2`
                format_list_like_thing(
                    ps,
                    array_node.elements(),
                    array_node.location().end_offset(),
                    true,
                );
            } else {
                ps.breakable_of(BreakableDelims::for_array(), |ps| {
                    format_list_like_thing(
                        ps,
                        array_node.elements(),
                        array_node.location().end_offset(),
                        false,
                    );
                    ps.wind_dumping_comments_until_offset(array_node.location().end_offset());
                });
            }
        });
    }
}

fn format_word_array_elements(
    ps: &mut ParserState,
    node_list: prism::NodeList,
    end_offset: SourceOffset,
    orig_open_delim: char,
) {
    let args_count = node_list.len();
    let orig_close_delim = matching_delimiter(orig_open_delim);

    ps.magic_handle_comments_for_multiline_arrays(
        Some(ps.get_line_number_for_offset(end_offset)),
        |ps| {
            for (idx, expr) in node_list.iter().enumerate() {
                ps.emit_soft_indent();

                if let Some(string_node) = expr.as_string_node() {
                    ps.at_offset(string_node.location().start_offset());

                    let content = loc_to_str(string_node.content_loc());
                    let escaped = crate::string_escape::escape_word_array_content(
                        content,
                        orig_open_delim,
                        orig_close_delim,
                    );
                    ps.emit_string_content(escaped);
                } else if let Some(symbol_node) = expr.as_symbol_node() {
                    ps.at_offset(symbol_node.location().start_offset());

                    if let Some(value_loc) = symbol_node.value_loc() {
                        let content = loc_to_str(value_loc);
                        let escaped = crate::string_escape::escape_word_array_content(
                            content,
                            orig_open_delim,
                            orig_close_delim,
                        );
                        ps.emit_string_content(escaped);
                    }
                } else if let Some(interpolated_symbol_node) = expr.as_interpolated_symbol_node() {
                    ps.at_offset(interpolated_symbol_node.location().start_offset());
                    format_word_array_interpolated_parts(
                        ps,
                        interpolated_symbol_node.parts(),
                        orig_open_delim,
                        orig_close_delim,
                    );
                } else if let Some(interpolated_string_node) = expr.as_interpolated_string_node() {
                    ps.at_offset(interpolated_string_node.location().start_offset());
                    format_word_array_interpolated_parts(
                        ps,
                        interpolated_string_node.parts(),
                        orig_open_delim,
                        orig_close_delim,
                    );
                } else {
                    // This branch shouldn't happen, but we'll have a fallback just in case
                    if cfg!(debug_assertions) {
                        unreachable!("Received unexpected node in word array: {:?}", expr);
                    }
                    format_node(ps, expr);
                }

                if idx != args_count - 1 {
                    ps.emit_soft_newline();
                }
            }
        },
    );
}

fn matching_delimiter(open: char) -> char {
    match open {
        '[' => ']',
        '(' => ')',
        '{' => '}',
        '<' => '>',
        // For non-paired delimiters (like ^, |, etc.), the closing is the same
        c => c,
    }
}

fn format_word_array_interpolated_parts(
    ps: &mut ParserState,
    parts: prism::NodeList,
    orig_open_delim: char,
    orig_close_delim: char,
) {
    for part in parts.iter() {
        let start_offset = part.location().start_offset();
        let end_offset = part.location().end_offset();

        ps.at_offset(start_offset);

        if let Some(string_node) = part.as_string_node() {
            let content = loc_to_str(string_node.content_loc());
            let escaped = crate::string_escape::escape_word_array_content(
                content,
                orig_open_delim,
                orig_close_delim,
            );
            ps.emit_string_content(escaped);
        } else if let Some(embedded_statements_node) = part.as_embedded_statements_node() {
            format_embedded_statements_node(ps, embedded_statements_node);
        } else {
            format_node(ps, part);
        }

        ps.at_offset(end_offset);
    }
}

fn format_array_pattern_node(_ps: &mut ParserState, _array_pattern_node: prism::ArrayPatternNode) {
    todo!()
}

fn format_parentheses_node(ps: &mut ParserState, parentheses_node: prism::ParenthesesNode) {
    ps.emit_open_paren();

    let is_multiline = if let Some(body) = parentheses_node.body() {
        if let Some(statements_node) = body.as_statements_node() {
            statements_node.body().len() > 1
        } else {
            true
        }
    } else {
        false
    };

    if let Some(body) = parentheses_node.body() {
        ps.with_start_of_line(false, |ps| {
            if let Some(statements_node) = body.as_statements_node() {
                if statements_node.body().len() == 1 {
                    ps.with_start_of_line(false, |ps| {
                        format_node(ps, statements_node.body().iter().next().unwrap())
                    });
                } else {
                    ps.emit_newline();
                    ps.new_block(|ps| {
                        ps.with_start_of_line(true, |ps| {
                            format_node(ps, body);
                        });
                    });
                }
            } else {
                // I'm *pretty* sure this should always be a StatementsNode, but this is here
                // just to be defensive
                ps.emit_newline();
                ps.new_block(|ps| {
                    ps.with_start_of_line(true, |ps| {
                        format_node(ps, body);
                    });
                });
            }
        });
    }

    if is_multiline {
        ps.emit_indent();
        ps.emit_close_paren();
    } else {
        ps.emit_close_paren();
    }
}

fn split_node_into_call_chains(node: prism::Node) -> Vec<Vec<prism::Node>> {
    let mut elements = vec![];
    let mut maybe_receiver = Some(node);
    while let Some(receiver) = maybe_receiver {
        maybe_receiver = receiver
            .as_call_node()
            .and_then(|call_node| call_node.receiver());
        elements.insert(0, receiver);
    }

    // Precompute which elements have dots anywhere in the chain after them.
    // Later, when we see an aref, we'll use this to decide whether to cut a new segment
    let mut has_dot_after: Vec<bool> = vec![false; elements.len()];
    let mut seen_dot = false;
    for i in (0..elements.len()).rev() {
        has_dot_after[i] = seen_dot;
        if let Some(call_node) = elements[i].as_call_node()
            && call_node.call_operator_loc().is_some()
        {
            seen_dot = true;
        }
    }

    let mut split_before: Vec<usize> = vec![];
    let mut seen_dot_call = false;

    for (i, node) in elements.iter().enumerate() {
        if let Some(call_node) = node.as_call_node() {
            let node_is_dot_call = call_node.call_operator_loc().is_some();
            // Other calls without dots (unary/infix operators) would not be in a chain
            let is_aref: bool = !node_is_dot_call;

            if is_aref && seen_dot_call && has_dot_after[i] {
                split_before.push(i);
                seen_dot_call = false;
                continue;
            }

            if node_is_dot_call {
                seen_dot_call = true;
            }
        }
    }

    if split_before.is_empty() {
        return vec![elements];
    }

    // Split the vec at the indices (in reverse order so indices remain valid)
    let mut segments: Vec<Vec<prism::Node>> = vec![];
    let mut remaining = elements;

    for split_idx in split_before.into_iter().rev() {
        let after = remaining.split_off(split_idx);
        segments.insert(0, after);
    }
    segments.insert(0, remaining);

    segments
}

fn format_rest_parameter_node(ps: &mut ParserState, rest_param: prism::RestParameterNode) {
    ps.with_start_of_line(false, |ps| {
        ps.emit_soft_indent();
        ps.emit_ident("*".to_string());
        ps.with_start_of_line(false, |ps| {
            if let Some(name) = rest_param.name() {
                let name_str = const_to_string(name);
                ps.bind_variable(name_str.clone());
                format_ident(ps, name_str, rest_param.name_loc().unwrap().end_offset());
            }
        });
    });
}

fn format_arguments_node(ps: &mut ParserState, arguments_node: prism::ArgumentsNode) {
    ps.with_start_of_line(false, |ps| {
        format_list_like_thing(
            ps,
            arguments_node.arguments(),
            arguments_node.location().end_offset(),
            false,
        );
    });
}

fn format_keyword_hash_node(ps: &mut ParserState, keyword_hash_node: prism::KeywordHashNode) {
    let all_symbol_keys = keyword_hash_node
        .elements()
        .iter()
        .filter_map(|node| node.as_assoc_node())
        // The operator loc is empty for symbol keys
        .all(|assoc| assoc.operator_loc().is_none());
    let hash_type = if all_symbol_keys {
        HashType::SymbolKey
    } else {
        HashType::HashRocket
    };

    ps.with_start_of_line(false, |ps| {
        ps.with_formatting_context(FormattingContext::HashType(hash_type), |ps| {
            format_list_like_thing(
                ps,
                keyword_hash_node.elements(),
                keyword_hash_node.location().end_offset(),
                false,
            );
        });
    });
}

fn format_keyword_rest_parameter_node(
    ps: &mut ParserState,
    keyword_rest_parameter_node: prism::KeywordRestParameterNode,
) {
    ps.emit_soft_indent();
    ps.emit_ident("**".to_string());
    if let Some(constant_id) = keyword_rest_parameter_node.name() {
        let name = const_to_string(constant_id);
        ps.bind_variable(name.clone());
        ps.emit_ident(name);
    }
}

fn format_required_keyword_parameter_node(
    ps: &mut ParserState,
    required_keyword_parameter_node: prism::RequiredKeywordParameterNode,
) {
    let name = const_to_string(required_keyword_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
    ps.emit_ident(":".to_string());
}

fn format_required_parameter_node(
    ps: &mut ParserState,
    required_parameter_node: prism::RequiredParameterNode,
) {
    let name = const_to_string(required_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
}

fn format_local_variable_and_write_node(
    ps: &mut ParserState,
    local_variable_and_write_node: prism::LocalVariableAndWriteNode,
) {
    let variable_name = const_to_string(local_variable_and_write_node.name());
    ps.bind_variable(variable_name.clone());
    format_write_node(
        ps,
        variable_name,
        Cow::Borrowed("&&="),
        local_variable_and_write_node.value(),
    );
}

fn format_local_variable_operator_write_node(
    ps: &mut ParserState,
    local_variable_operator_write_node: prism::LocalVariableOperatorWriteNode,
) {
    let variable_name = const_to_string(local_variable_operator_write_node.name());
    ps.bind_variable(variable_name.clone());
    format_write_node(
        ps,
        variable_name,
        Cow::Owned(loc_to_string(
            local_variable_operator_write_node.binary_operator_loc(),
        )),
        local_variable_operator_write_node.value(),
    );
}

fn format_local_variable_or_write_node(
    ps: &mut ParserState,
    local_variable_or_write_node: prism::LocalVariableOrWriteNode,
) {
    let variable_name = const_to_string(local_variable_or_write_node.name());
    ps.bind_variable(variable_name.clone());
    format_write_node(
        ps,
        variable_name,
        Cow::Borrowed("||="),
        local_variable_or_write_node.value(),
    );
}

fn format_local_variable_target_node(
    ps: &mut ParserState,
    local_variable_target_node: prism::LocalVariableTargetNode,
) {
    let variable_name = const_to_string(local_variable_target_node.name());
    ps.bind_variable(variable_name.clone());
    ps.emit_ident(variable_name);
}

fn format_local_variable_read_node(
    ps: &mut ParserState,
    local_variable_read_node: prism::LocalVariableReadNode,
) {
    let name = const_to_string(local_variable_read_node.name());
    ps.emit_ident(name);
}

fn format_local_variable_write_node(
    ps: &mut ParserState,
    local_variable_write_node: prism::LocalVariableWriteNode,
) {
    let name = const_to_string(local_variable_write_node.name());
    ps.bind_variable(name.clone());
    format_write_node(
        ps,
        name,
        Cow::Borrowed("="),
        local_variable_write_node.value(),
    );
}

fn format_splat_node(ps: &mut ParserState, splat_node: prism::SplatNode) {
    ps.emit_ident("*".to_string());
    if let Some(node) = splat_node.expression() {
        ps.with_start_of_line(false, |ps| {
            format_node(ps, node);
        });
    }
}

fn format_ident(ps: &mut ParserState, ident: String, offset: usize) {
    handle_string_at_offset(ps, ident, offset);
}

fn format_instance_variable_write_node(
    ps: &mut ParserState,
    instance_variable_write_node: prism::InstanceVariableWriteNode,
) {
    ps.at_offset(instance_variable_write_node.location().start_offset());
    format_write_node(
        ps,
        const_to_string(instance_variable_write_node.name()),
        Cow::Borrowed("="),
        instance_variable_write_node.value(),
    );
}

fn format_integer_node(ps: &mut ParserState, integer_node: prism::IntegerNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(integer_node.location()),
        integer_node.location().start_offset(),
    );
}

fn format_float_node(ps: &mut ParserState, float_node: prism::FloatNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(float_node.location()),
        float_node.location().start_offset(),
    );
}

fn format_for_node(ps: &mut ParserState, for_node: prism::ForNode) {
    ps.emit_keyword("for");
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        format_node(ps, for_node.index());

        ps.emit_space();
        ps.emit_keyword("in");
        ps.emit_space();

        format_node(ps, for_node.collection());
    });

    ps.new_block(|ps| {
        ps.emit_newline();
        if let Some(statements_node) = for_node.statements() {
            format_statements(ps, statements_node);
        }
    });

    ps.with_start_of_line(true, |ps| ps.emit_end());
}

fn format_forwarding_arguments_node(
    ps: &mut ParserState,
    forwarding_arguments_node: prism::ForwardingArgumentsNode,
) {
    handle_string_at_offset(
        ps,
        "...".to_string(),
        forwarding_arguments_node.location().start_offset(),
    );
}

fn format_forwarding_parameter_node(
    ps: &mut ParserState,
    forwarding_parameter_node: prism::ForwardingParameterNode,
) {
    handle_string_at_offset(
        ps,
        "...".to_string(),
        forwarding_parameter_node.location().start_offset(),
    );
}

fn format_forwarding_super_node(
    ps: &mut ParserState,
    forwarding_super_node: prism::ForwardingSuperNode,
) {
    ps.emit_ident("super".to_string());
    if let Some(block) = forwarding_super_node.block() {
        ps.emit_space();
        format_block_node(ps, block);
    }
}

fn format_super_node(ps: &mut ParserState, super_node: prism::SuperNode) {
    ps.emit_ident("super".to_string());
    // Note that we always emit parens for SuperNodes,
    // since they're distinct from ForwardingSuperNode which never use parens
    ps.with_start_of_line(false, |ps| {
        ps.breakable_of(BreakableDelims::for_method_call(), |ps| {
            if let Some(arguments) = super_node.arguments() {
                format_arguments_node(ps, arguments);
            }
            if let Some(block) = super_node.block()
                && let Some(block_arg) = block.as_block_argument_node()
            {
                if super_node.arguments().is_some() {
                    ps.emit_comma();
                    ps.emit_soft_newline();
                }
                ps.with_start_of_line(false, |ps| format_block_argument_node(ps, block_arg));
            }
        });

        if let Some(block) = super_node.block()
            && let Some(block_node) = block.as_block_node()
        {
            ps.emit_space();
            ps.with_start_of_line(false, |ps| format_block_node(ps, block_node));
        }
    });
}

fn format_global_variable_and_write_node(
    ps: &mut ParserState,
    global_variable_and_write_node: prism::GlobalVariableAndWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(global_variable_and_write_node.name()),
        Cow::Borrowed("&&="),
        global_variable_and_write_node.value(),
    );
}

fn format_global_variable_operator_write_node(
    ps: &mut ParserState,
    global_variable_operator_write_node: prism::GlobalVariableOperatorWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(global_variable_operator_write_node.name()),
        Cow::Owned(loc_to_string(
            global_variable_operator_write_node.binary_operator_loc(),
        )),
        global_variable_operator_write_node.value(),
    );
}

fn format_global_variable_or_write_node(
    ps: &mut ParserState,
    global_variable_or_write_node: prism::GlobalVariableOrWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(global_variable_or_write_node.name()),
        Cow::Borrowed("||="),
        global_variable_or_write_node.value(),
    );
}

fn format_global_variable_read_node(
    ps: &mut ParserState,
    global_variable_read_node: prism::GlobalVariableReadNode,
) {
    ps.emit_ident(loc_to_string(global_variable_read_node.location()));
}

fn format_global_variable_target_node(
    ps: &mut ParserState,
    global_variable_target_node: prism::GlobalVariableTargetNode,
) {
    ps.emit_ident(const_to_string(global_variable_target_node.name()));
}

fn format_global_variable_write_node(
    ps: &mut ParserState,
    global_variable_write_node: prism::GlobalVariableWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(global_variable_write_node.name()),
        Cow::Borrowed("="),
        global_variable_write_node.value(),
    );
}

fn format_hash_node(ps: &mut ParserState, hash_node: prism::HashNode) {
    ps.with_start_of_line(false, |ps| {
        if hash_node.elements().is_empty() {
            let start_offset = hash_node.location().start_offset();
            let end_offset = hash_node.location().end_offset();
            let is_multiline = ps.get_line_number_for_offset(start_offset)
                != ps.get_line_number_for_offset(end_offset);

            let has_comments = ps.has_comment_in_offset_span(start_offset, end_offset);

            if is_multiline && has_comments {
                // Since we already know this is multiline, we can just use
                // a breakable and know that it will always be the multiline form
                // instead of manually inserting all of the newlines/indents for
                // a multiline hash
                ps.breakable_of(BreakableDelims::for_hash(), |ps| {
                    ps.wind_dumping_comments_until_offset(end_offset);
                });
            } else {
                ps.emit_ident("{}".to_string());
                ps.wind_dumping_comments_until_offset(end_offset);
            }
        } else {
            let all_symbol_keys = hash_node
                .elements()
                .iter()
                .filter_map(|node| node.as_assoc_node())
                // The operator loc is empty for symbol keys
                .all(|assoc| assoc.operator_loc().is_none());
            let hash_type = if all_symbol_keys {
                HashType::SymbolKey
            } else {
                HashType::HashRocket
            };

            ps.with_formatting_context(FormattingContext::HashType(hash_type), |ps| {
                ps.breakable_of(BreakableDelims::for_hash(), |ps| {
                    ps.emit_soft_indent();
                    format_list_like_thing(
                        ps,
                        hash_node.elements(),
                        hash_node.closing_loc().end_offset(),
                        false,
                    );
                    ps.wind_dumping_comments_until_offset(hash_node.closing_loc().end_offset());
                });
            });
        }
    });
}

fn format_hash_pattern_node(_ps: &mut ParserState, _hash_pattern_node: prism::HashPatternNode) {
    todo!()
}

fn format_inline_conditional(
    ps: &mut ParserState,
    predicate: prism::Node,
    statements: Option<prism::StatementsNode>,
    keyword: &'static str,
) {
    if let Some(statements) = statements {
        // There can only be a single statement in modifier form.
        // Format it directly to skip the StatementsNode machinery
        if let Some(first_statement) = statements.body().iter().next() {
            ps.with_start_of_line(false, |ps| format_node(ps, first_statement));
        }
        ps.emit_space();
    }
    ps.emit_conditional_keyword(keyword);
    ps.emit_space();
    ps.with_start_of_line(false, |ps| format_node(ps, predicate));
}

enum Conditional<'pr> {
    If(prism::IfNode<'pr>),
    Unless(prism::UnlessNode<'pr>),
    While(prism::WhileNode<'pr>),
    Until(prism::UntilNode<'pr>),
}

impl<'pr> Conditional<'pr> {
    fn predicate(&self) -> prism::Node<'pr> {
        match self {
            Conditional::If(node) => node.predicate(),
            Conditional::Unless(node) => node.predicate(),
            Conditional::While(node) => node.predicate(),
            Conditional::Until(node) => node.predicate(),
        }
    }

    fn statements(&self) -> Option<prism::StatementsNode<'pr>> {
        match self {
            Conditional::If(node) => node.statements(),
            Conditional::Unless(node) => node.statements(),
            Conditional::While(node) => node.statements(),
            Conditional::Until(node) => node.statements(),
        }
    }

    fn subsequent_or_else(&self) -> Option<prism::Node<'pr>> {
        match self {
            Conditional::If(node) => node.subsequent(),
            Conditional::Unless(node) => node.else_clause().map(|ec| ec.as_node()),
            Conditional::While(_) | Conditional::Until(_) => None,
        }
    }

    fn is_begin_modifier(&self) -> bool {
        match self {
            Conditional::If(_) | Conditional::Unless(_) => false,
            Conditional::While(node) => node.is_begin_modifier(),
            Conditional::Until(node) => node.is_begin_modifier(),
        }
    }

    fn keyword_start_offset(&self) -> usize {
        match self {
            Conditional::If(node) => node
                .if_keyword_loc()
                .map(|loc| loc.start_offset())
                .unwrap_or(node.location().start_offset()),
            Conditional::Unless(node) => node.keyword_loc().start_offset(),
            Conditional::While(node) => node.keyword_loc().start_offset(),
            Conditional::Until(node) => node.keyword_loc().start_offset(),
        }
    }
}

fn format_conditional_node(
    ps: &mut ParserState,
    conditional_keyword: &'static str,
    requires_end_keyword: bool,
    conditional: &Conditional,
) {
    // while/until nodes have special behavior if they're modifiers on `begin` nodes.
    // This is because `begin; end while false` will always run the begin block once, whereas
    // `while false; begin; end; end` will not run it at all, so it is unsafe to convert to block
    if conditional.is_begin_modifier() {
        let begin_node = conditional
            .statements()
            .expect("Begin modifiers must have a StatementsNode")
            .body()
            .iter()
            .next()
            .expect("Begin modifiers must have a single statement")
            .as_begin_node()
            .expect("Statement in a begin modifier must be a BeginNode");
        ps.with_start_of_line(false, |ps| {
            format_begin_node(ps, begin_node);
            ps.emit_space();
            ps.emit_keyword(conditional_keyword);
            ps.emit_space();
            format_node(ps, conditional.predicate());
        });
        return;
    }

    let is_modifier = if let Some(statements) = conditional.statements() {
        statements.location().start_offset() < conditional.keyword_start_offset()
    } else {
        false
    };

    if is_modifier {
        // For while/until, always use the inline format for modifiers, since
        //   some transformations are unsafe.
        // For if/unless, check if we should convert to block form.
        let should_convert_to_block = match conditional {
            Conditional::While(_) | Conditional::Until(_) => false,
            Conditional::If(_) | Conditional::Unless(_) => {
                let predicate = conditional.predicate();
                let statements = conditional.statements();
                let predicate_start_line =
                    ps.get_line_number_for_offset(predicate.location().start_offset());
                let predicate_end_line =
                    ps.get_line_number_for_offset(predicate.location().end_offset());
                if predicate_start_line != predicate_end_line {
                    true
                } else {
                    // Check if it renders multiline due to length
                    ps.will_render_as_multiline(|next_ps| {
                        format_inline_conditional(
                            next_ps,
                            predicate,
                            statements,
                            conditional_keyword,
                        )
                    })
                }
            }
        };

        if should_convert_to_block {
            format_conditional_block_form(
                ps,
                conditional_keyword,
                conditional.predicate(),
                conditional.statements(),
                conditional.subsequent_or_else(),
                requires_end_keyword,
            );
        } else {
            format_inline_conditional(
                ps,
                conditional.predicate(),
                conditional.statements(),
                conditional_keyword,
            );
        }
    } else {
        format_conditional_block_form(
            ps,
            conditional_keyword,
            conditional.predicate(),
            conditional.statements(),
            conditional.subsequent_or_else(),
            requires_end_keyword,
        );
    }
}

fn format_conditional_block_form<'pr>(
    ps: &mut ParserState,
    conditional_keyword: &'static str,
    predicate: prism::Node<'pr>,
    statements: Option<prism::StatementsNode<'pr>>,
    subsequent_or_else: Option<prism::Node<'pr>>,
    requires_end_keyword: bool,
) {
    ps.emit_conditional_keyword(conditional_keyword);
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        ps.new_block(|ps| {
            format_node(ps, predicate);
        });
    });

    ps.new_block(|ps| {
        ps.emit_newline();
        if let Some(statements) = statements {
            format_node(ps, statements.as_node());
        }
    });

    if let Some(subsequent_or_else) = subsequent_or_else {
        ps.with_start_of_line(false, |ps| {
            ps.emit_indent();
            format_node(ps, subsequent_or_else);
        });
    }
    if requires_end_keyword {
        ps.with_start_of_line(true, |ps| ps.emit_end());
    }
}

fn format_if_node(ps: &mut ParserState, if_node: prism::IfNode) {
    // If a keyword is present, we're in an `if/elsif` block.
    // If it's not there, this is actually a ternary, which is sufficiently
    // different that we handle it in its own branch
    if let Some(if_loc) = if_node.if_keyword_loc() {
        let is_if_keyword = (if_loc.end_offset() - if_loc.start_offset()) == 2;
        let conditional_keyword = if is_if_keyword { "if" } else { "elsif" };

        format_conditional_node(
            ps,
            conditional_keyword,
            // `elsif` nodes don't need an `else` keyword, that's handled
            // by the parent `if` node.
            is_if_keyword,
            &Conditional::If(if_node),
        );
    } else {
        // No keyword, so this is a ternary
        ps.with_formatting_context(FormattingContext::IfOp, |ps| {
            ps.with_start_of_line(false, |ps| {
                format_node(ps, if_node.predicate());
                ps.emit_ident(" ? ".to_string());

                format_node(
                    ps,
                    if_node
                        .statements()
                        .expect("Ternaries must have a `statements` branch")
                        .body()
                        .iter()
                        .next()
                        .expect("There must be exactly one statement inside a ternary branch"),
                );
                format_node(
                    ps,
                    if_node
                        .subsequent()
                        .expect("Ternaries must have a subsequent branch"),
                )
            });
        });
    }
}

fn format_imaginary_node(ps: &mut ParserState, imaginary_node: prism::ImaginaryNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(imaginary_node.location()),
        imaginary_node.location().start_offset(),
    );
}

fn format_implicit_node() {
    // Do nothing!
    // This implicit node represents an implicit value in hash shorthands,
    // e.g. `{ a: }`, so we don't actually need to do anything to format it
}

fn format_implicit_rest_node() {
    // Intentionally do nothing.
    //
    // prism::ImplicitRestNode is essentially a placeholder for some variable declaration like
    // func { |x,| }, where the trailing comma is the "implicit rest node". This doesn't actually require us
    // to do anything, since this node will basically be listed as a node in the arguments list, so we'll
    // treat it like any other argument: by emitting a comma and a space before it.
    // Since other machinery actually handles all of this, we don't really need to do anything if we end up here.
}

fn format_in_node(_ps: &mut ParserState, _in_node: prism::InNode) {
    todo!()
}

fn format_index_and_write_node(
    ps: &mut ParserState,
    index_and_write_node: prism::IndexAndWriteNode,
) {
    if let Some(receiver) = index_and_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(arguments) = index_and_write_node.arguments() {
        ps.breakable_of(BreakableDelims::for_array(), |ps| {
            format_arguments_node(ps, arguments)
        });
    }

    ps.emit_space();
    ps.emit_op(Cow::Borrowed("&&="));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| format_node(ps, index_and_write_node.value()));
}

fn format_index_operator_write_node(
    ps: &mut ParserState,
    index_operator_write_node: prism::IndexOperatorWriteNode,
) {
    if let Some(receiver) = index_operator_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(arguments) = index_operator_write_node.arguments() {
        ps.breakable_of(BreakableDelims::for_array(), |ps| {
            format_arguments_node(ps, arguments)
        });
    }

    ps.emit_space();
    ps.emit_op(Cow::Owned(loc_to_string(
        index_operator_write_node.binary_operator_loc(),
    )));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| {
        format_node(ps, index_operator_write_node.value())
    });
}

fn format_index_or_write_node(ps: &mut ParserState, index_or_write_node: prism::IndexOrWriteNode) {
    if let Some(receiver) = index_or_write_node.receiver() {
        ps.with_start_of_line(false, |ps| format_node(ps, receiver));
    }

    if let Some(arguments) = index_or_write_node.arguments() {
        ps.breakable_of(BreakableDelims::for_array(), |ps| {
            format_arguments_node(ps, arguments)
        });
    }

    ps.emit_space();
    ps.emit_op(Cow::Borrowed("||="));
    ps.emit_space();

    ps.with_start_of_line(false, |ps| format_node(ps, index_or_write_node.value()));
}

fn format_index_target_node(ps: &mut ParserState, index_target_node: prism::IndexTargetNode) {
    ps.with_start_of_line(false, |ps| format_node(ps, index_target_node.receiver()));

    if let Some(arguments) = index_target_node.arguments() {
        ps.breakable_of(BreakableDelims::for_array(), |ps| {
            format_arguments_node(ps, arguments)
        });
    }
}

fn format_instance_variable_and_write_node(
    ps: &mut ParserState,
    instance_variable_and_write_node: prism::InstanceVariableAndWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(instance_variable_and_write_node.name()),
        Cow::Borrowed("&&="),
        instance_variable_and_write_node.value(),
    );
}

fn format_instance_variable_operator_write_node(
    ps: &mut ParserState,
    instance_variable_operator_write_node: prism::InstanceVariableOperatorWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(instance_variable_operator_write_node.name()),
        Cow::Owned(loc_to_string(
            instance_variable_operator_write_node.binary_operator_loc(),
        )),
        instance_variable_operator_write_node.value(),
    );
}

fn format_instance_variable_or_write_node(
    ps: &mut ParserState,
    instance_variable_or_write_node: prism::InstanceVariableOrWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(instance_variable_or_write_node.name()),
        Cow::Borrowed("||="),
        instance_variable_or_write_node.value(),
    );
}

fn format_instance_variable_read_node(
    ps: &mut ParserState,
    instance_variable_read_node: prism::InstanceVariableReadNode,
) {
    ps.emit_ident(const_to_string(instance_variable_read_node.name()));
}

fn format_instance_variable_target_node(
    ps: &mut ParserState,
    instance_variable_target_node: prism::InstanceVariableTargetNode,
) {
    ps.emit_ident(const_to_string(instance_variable_target_node.name()));
}

fn format_constant_read_node(ps: &mut ParserState, constant_read_node: prism::ConstantReadNode) {
    handle_string_at_offset(
        ps,
        const_to_string(constant_read_node.name()),
        constant_read_node.location().start_offset(),
    );
}

fn format_constant_path_node(ps: &mut ParserState, constant_path_node: prism::ConstantPathNode) {
    ps.with_start_of_line(false, |ps| {
        if let Some(parent) = constant_path_node.parent() {
            format_node(ps, parent);
        }
        // Emit :: regardless of if there's a parent
        // since it could be a top reference
        ps.emit_colon_colon();

        handle_string_at_offset(
            ps,
            const_to_string(constant_path_node.name().unwrap()),
            constant_path_node.name_loc().start_offset(),
        );
    });
}

fn format_constant_and_write_node(
    ps: &mut ParserState,
    constant_and_write_node: prism::ConstantAndWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(constant_and_write_node.name()),
        Cow::Borrowed("&&="),
        constant_and_write_node.value(),
    );
}

fn format_constant_operator_write_node(
    ps: &mut ParserState,
    constant_operator_write_node: prism::ConstantOperatorWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(constant_operator_write_node.name()),
        Cow::Owned(loc_to_string(
            constant_operator_write_node.binary_operator_loc(),
        )),
        constant_operator_write_node.value(),
    );
}

fn format_constant_or_write_node(
    ps: &mut ParserState,
    constant_or_write_node: prism::ConstantOrWriteNode,
) {
    format_write_node(
        ps,
        const_to_string(constant_or_write_node.name()),
        Cow::Borrowed("||="),
        constant_or_write_node.value(),
    );
}

fn format_constant_path_and_write_node(
    ps: &mut ParserState,
    constant_path_and_write_node: prism::ConstantPathAndWriteNode,
) {
    format_constant_path_write(
        ps,
        constant_path_and_write_node.target(),
        Cow::Borrowed("&&="),
        constant_path_and_write_node.value(),
    );
}

fn format_constant_path_operator_write_node(
    ps: &mut ParserState,
    constant_path_operator_write_node: prism::ConstantPathOperatorWriteNode,
) {
    format_constant_path_write(
        ps,
        constant_path_operator_write_node.target(),
        Cow::Owned(loc_to_string(
            constant_path_operator_write_node.binary_operator_loc(),
        )),
        constant_path_operator_write_node.value(),
    );
}

fn format_constant_path_or_write_node(
    ps: &mut ParserState,
    constant_path_or_write_node: prism::ConstantPathOrWriteNode,
) {
    format_constant_path_write(
        ps,
        constant_path_or_write_node.target(),
        Cow::Borrowed("||="),
        constant_path_or_write_node.value(),
    );
}

fn format_constant_path_target_node(
    ps: &mut ParserState,
    constant_path_target_node: prism::ConstantPathTargetNode,
) {
    ps.with_start_of_line(false, |ps| {
        if let Some(parent) = constant_path_target_node.parent() {
            format_node(ps, parent);
        }
        // Emit :: regardless of if there's a parent
        // since it could be a top reference
        ps.emit_colon_colon();

        handle_string_at_offset(
            ps,
            const_to_string(constant_path_target_node.name().unwrap()),
            constant_path_target_node.name_loc().start_offset(),
        );
    });
}

fn format_constant_path_write_node(
    ps: &mut ParserState,
    constant_path_write_node: prism::ConstantPathWriteNode,
) {
    format_constant_path_write(
        ps,
        constant_path_write_node.target(),
        Cow::Borrowed("="),
        constant_path_write_node.value(),
    );
}

fn format_constant_target_node(
    ps: &mut ParserState,
    constant_target_node: prism::ConstantTargetNode,
) {
    handle_string_at_offset(
        ps,
        const_to_string(constant_target_node.name()),
        constant_target_node.location().start_offset(),
    );
}

fn format_constant_path_write(
    ps: &mut ParserState,
    target: prism::ConstantPathNode,
    op: Cow<'static, str>,
    value: prism::Node,
) {
    format_constant_path_node(ps, target);
    ps.emit_space();
    ps.emit_op(op);
    ps.emit_space();
    ps.with_start_of_line(false, |ps| format_node(ps, value));
}

fn format_constant_write_node(ps: &mut ParserState, constant_write_node: prism::ConstantWriteNode) {
    format_write_node(
        ps,
        const_to_string(constant_write_node.name()),
        Cow::Borrowed("="),
        constant_write_node.value(),
    );
}

fn format_lambda_node(ps: &mut ParserState, lambda_node: prism::LambdaNode) {
    let operator = loc_to_string(lambda_node.operator_loc());

    ps.with_start_of_line(false, |ps| {
        ps.emit_ident(operator.clone());

        if let Some(parameters_node) = lambda_node.parameters() {
            if &operator == "->"
                && let Some(block_parameters) = parameters_node.as_block_parameters_node()
            {
                if block_parameters.parameters().is_some() || !block_parameters.locals().is_empty()
                {
                    ps.emit_space();
                    ps.breakable_of(BreakableDelims::for_method_call(), |ps| {
                        format_block_parameters_names(
                            ps,
                            block_parameters.locals(),
                            block_parameters.parameters(),
                            block_parameters.location().end_offset(),
                        );
                    });
                }
            } else {
                format_node(ps, parameters_node);
            }
        }

        let opening = loc_to_str(lambda_node.opening_loc());

        if opening == "do" {
            ps.emit_space();
            ps.new_block(|ps| {
                ps.emit_do_keyword();

                ps.emit_newline();

                if let Some(body) = lambda_node.body() {
                    ps.with_start_of_line(true, |ps| {
                        format_node(ps, body);
                    });
                }
            });

            ps.with_start_of_line(true, |ps| {
                ps.wind_dumping_comments_until_offset(lambda_node.location().end_offset());
                ps.emit_end();
                ps.shift_comments();
            });
        } else {
            ps.emit_space();
            ps.inline_breakable_of(BreakableDelims::for_brace_block(), |ps| {
                if let Some(body) = lambda_node.body() {
                    let has_multiple_statements = body
                        .as_statements_node()
                        .map(|statements_node| statements_node.body().len() > 1)
                        .unwrap_or(false);
                    if has_multiple_statements {
                        ps.emit_soft_newline();
                        ps.with_start_of_line(true, |ps| {
                            format_node(ps, body);
                        });
                    } else {
                        ps.with_start_of_line(false, |ps| {
                            if let Some(node) =
                                body.as_statements_node().unwrap().body().iter().next()
                            {
                                ps.emit_soft_newline();
                                ps.emit_soft_indent();
                                format_node(ps, node);
                                ps.emit_soft_newline();
                            }
                        });
                    }
                } else if ps.has_comment_in_offset_span(
                    lambda_node.opening_loc().start_offset(),
                    lambda_node.closing_loc().end_offset(),
                ) {
                    ps.emit_soft_newline();
                }

                ps.dedent(|ps| ps.emit_soft_indent());
                ps.wind_dumping_comments_until_offset(lambda_node.location().end_offset());
                ps.shift_comments();
            });
        }
    });
}

fn format_match_last_line_node(
    ps: &mut ParserState,
    match_last_line_node: prism::MatchLastLineNode,
) {
    ps.emit_ident(loc_to_string(match_last_line_node.opening_loc()));
    ps.emit_string_content(loc_to_string(match_last_line_node.content_loc()));
    ps.emit_ident(loc_to_string(match_last_line_node.closing_loc()));
}

fn format_match_predicate_node(
    _ps: &mut ParserState,
    _match_predicate_node: prism::MatchPredicateNode,
) {
    todo!()
}

fn format_match_required_node(
    _ps: &mut ParserState,
    _match_required_node: prism::MatchRequiredNode,
) {
    todo!()
}

fn format_match_write_node(ps: &mut ParserState, match_write_node: prism::MatchWriteNode) {
    format_call_node(ps, match_write_node.call(), false, true);
}

fn format_multi_target_node(ps: &mut ParserState, multi_target_node: prism::MultiTargetNode) {
    let has_parens = multi_target_node.lparen_loc().is_some();

    if has_parens {
        ps.emit_open_paren();
    }

    format_multi_targets(
        ps,
        multi_target_node.lefts(),
        multi_target_node.rest(),
        multi_target_node.rights(),
    );

    if has_parens {
        ps.emit_close_paren();
    }
}

fn format_multi_targets(
    ps: &mut ParserState,
    lefts: prism::NodeList,
    rest: Option<prism::Node>,
    rights: prism::NodeList,
) {
    let has_lefts = !lefts.is_empty();
    let has_rest = rest.is_some();
    let has_rights = !rights.is_empty();

    ps.with_start_of_line(false, |ps| {
        if has_lefts {
            let lefts_offset = lefts.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, lefts, lefts_offset, true);
        }

        if let Some(rest) = rest {
            if rest.as_implicit_rest_node().is_some() {
                ps.emit_comma();
            } else {
                if has_lefts {
                    ps.emit_comma_space();
                }
                format_node(ps, rest);
            }
        }

        if has_rights {
            if has_lefts || has_rest {
                ps.emit_comma_space();
            }
            let rights_offset = rights.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, rights, rights_offset, true);
        }
    });
}

fn format_multi_write_node(ps: &mut ParserState, multi_write_node: prism::MultiWriteNode) {
    format_multi_targets(
        ps,
        multi_write_node.lefts(),
        multi_write_node.rest(),
        multi_write_node.rights(),
    );

    ps.emit_ident(" = ".to_string());

    ps.with_start_of_line(false, |ps| format_node(ps, multi_write_node.value()));
}

fn format_next_node(ps: &mut ParserState, next_node: prism::NextNode) {
    ps.emit_ident("next".to_string());
    if let Some(arguments_node) = next_node.arguments() {
        ps.with_start_of_line(false, |ps| {
            ps.breakable_of(BreakableDelims::for_kw(), |ps| {
                format_arguments_node(ps, arguments_node);
            });
        });
    }
}

fn format_nil_node(ps: &mut ParserState) {
    ps.emit_ident("nil".to_string());
}

fn format_no_keywords_parameter_node(
    ps: &mut ParserState,
    no_keywords_parameter_node: prism::NoKeywordsParameterNode,
) {
    ps.emit_soft_indent();
    handle_string_at_offset(
        ps,
        "**nil".to_string(),
        no_keywords_parameter_node.location().start_offset(),
    );
}

fn format_numbered_parameters_node() {
    // No-op. This node represents the implicit set of numbered parameters,
    // and the actual parameter references are rendered separately.
}

fn format_numbered_reference_read_node(
    ps: &mut ParserState,
    numbered_reference_read_node: prism::NumberedReferenceReadNode,
) {
    ps.emit_ident(loc_to_string(numbered_reference_read_node.location()));
}

fn format_optional_keyword_parameter_node(
    ps: &mut ParserState,
    optional_keyword_parameter_node: prism::OptionalKeywordParameterNode,
) {
    ps.emit_ident(const_to_string(optional_keyword_parameter_node.name()));
    ps.emit_op(Cow::Borrowed(":"));
    ps.emit_space();
    ps.with_start_of_line(false, |ps| {
        format_node(ps, optional_keyword_parameter_node.value());
    });
}

fn format_optional_parameter_node(
    ps: &mut ParserState,
    optional_parameter_node: prism::OptionalParameterNode,
) {
    let name = const_to_string(optional_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
    ps.emit_space();
    ps.emit_op(Cow::Borrowed("="));
    ps.emit_space();
    format_node(ps, optional_parameter_node.value());
}

fn format_or_node(ps: &mut ParserState, or_node: prism::OrNode) {
    ps.inline_breakable_of(BreakableDelims::for_binary_op(), |ps| {
        ps.with_start_of_line(false, |ps| {
            format_infix_operator(
                ps,
                or_node.left(),
                loc_to_string(or_node.operator_loc()),
                or_node.right(),
            );
        });
    });
}

fn format_pinned_expression_node(
    _ps: &mut ParserState,
    _pinned_expression_node: prism::PinnedExpressionNode,
) {
    todo!()
}

fn format_pinned_variable_node(
    _ps: &mut ParserState,
    _pinned_variable_node: prism::PinnedVariableNode,
) {
    todo!()
}

fn format_post_execution_node(ps: &mut ParserState, post_execution_node: prism::PostExecutionNode) {
    ps.emit_keyword("END");
    ps.emit_space();
    ps.emit_open_curly_bracket();

    // END { } blocks are always multi-lined
    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.emit_newline();
            if let Some(statements) = post_execution_node.statements() {
                format_node(ps, statements.as_node());
            }
        })
    });

    ps.with_start_of_line(true, |ps| {
        ps.emit_close_curly_bracket();
    });
}

fn format_pre_execution_node(ps: &mut ParserState, pre_execution_node: prism::PreExecutionNode) {
    ps.emit_keyword("BEGIN");
    ps.emit_space();
    ps.emit_open_curly_bracket();

    // BEGIN { } blocks are always multi-lined
    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.emit_newline();
            if let Some(statements) = pre_execution_node.statements() {
                format_node(ps, statements.as_node());
            }
        })
    });

    ps.with_start_of_line(true, |ps| {
        ps.emit_close_curly_bracket();
    });
}

fn format_range_node(ps: &mut ParserState, range_node: prism::RangeNode) {
    ps.with_start_of_line(false, |ps| {
        if let Some(left) = range_node.left() {
            format_node(ps, left);
        }
        ps.emit_op(Cow::Owned(loc_to_string(range_node.operator_loc())));
        if let Some(right) = range_node.right() {
            format_node(ps, right);
        }
    });
}

fn format_rational_node(ps: &mut ParserState, rational_node: prism::RationalNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(rational_node.location()),
        rational_node.location().start_offset(),
    );
}

fn format_redo_node(ps: &mut ParserState) {
    ps.emit_ident("redo".to_string());
}

fn format_regular_expression_node(
    ps: &mut ParserState,
    regular_expression_node: prism::RegularExpressionNode,
) {
    ps.emit_ident(loc_to_string(regular_expression_node.opening_loc()));
    ps.emit_string_content(loc_to_string(regular_expression_node.content_loc()));
    ps.emit_ident(loc_to_string(regular_expression_node.closing_loc()));
}

fn format_rescue_modifier_node(
    ps: &mut ParserState,
    rescue_modifier_node: prism::RescueModifierNode,
) {
    ps.with_start_of_line(false, |ps| {
        format_node(ps, rescue_modifier_node.expression());
        ps.emit_space();
        ps.emit_rescue();
        ps.emit_space();
        format_node(ps, rescue_modifier_node.rescue_expression());
    });
}

fn format_rescue_node(ps: &mut ParserState, rescue_node: prism::RescueNode) {
    // Double check that these offsets are correct, since begin/rescue/ensure/else
    // aren't always handled with `format_node`, which usually handles this
    ps.at_offset(rescue_node.location().start_offset());

    ps.emit_keyword("rescue");
    let exceptions = rescue_node.exceptions();
    if !exceptions.is_empty() {
        ps.with_start_of_line(false, |ps| {
            ps.emit_space();
            format_list_like_thing(
                ps,
                exceptions,
                rescue_node
                    .exceptions()
                    .iter()
                    .last()
                    .unwrap()
                    .location()
                    .end_offset(),
                true,
            );
        });
    }

    if let Some(reference) = rescue_node.reference() {
        ps.emit_space();
        ps.emit_op(Cow::Borrowed("=>"));
        ps.emit_space();
        ps.with_start_of_line(false, |ps| {
            format_node(ps, reference);
        });
    }

    ps.new_block(|ps| {
        ps.emit_newline();
        if let Some(statements) = rescue_node.statements() {
            format_statements(ps, statements);
        }
        ps.shift_comments();
    });

    if let Some(subsequent) = rescue_node.subsequent() {
        ps.emit_indent();
        ps.with_start_of_line(false, |ps| format_node(ps, subsequent.as_node()));
    }

    ps.at_offset(rescue_node.location().end_offset());
}

fn format_retry_node(ps: &mut ParserState) {
    ps.emit_keyword("retry");
}

fn format_return_node(ps: &mut ParserState, return_node: prism::ReturnNode) {
    ps.emit_ident("return".to_string());
    ps.with_start_of_line(false, |ps| {
        if let Some(arguments) = return_node.arguments() {
            let arguments_list = arguments.arguments();
            if arguments_list.len() == 1 {
                ps.emit_space();
                format_node(ps, arguments_list.iter().last().unwrap());
            } else {
                ps.breakable_of(BreakableDelims::for_return_kw(), |ps| {
                    ps.with_start_of_line(false, |ps| {
                        format_list_like_thing(
                            ps,
                            arguments_list,
                            arguments.location().end_offset(),
                            false,
                        );
                    });
                });
            }
        }
    });
}

fn format_shareable_constant_node(
    ps: &mut ParserState,
    shareable_constant_node: prism::ShareableConstantNode,
) {
    format_node(ps, shareable_constant_node.write());
}

fn format_singleton_class_node(
    ps: &mut ParserState,
    singleton_class_node: prism::SingletonClassNode,
) {
    ps.emit_class_keyword();
    ps.emit_space();
    ps.emit_ident("<<".to_string());
    ps.emit_space();
    ps.emit_ident("self".to_string());

    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.with_formatting_context(FormattingContext::ClassOrModule, |ps| {
                ps.emit_newline();
                if let Some(body) = singleton_class_node.body() {
                    format_node(ps, body);
                }
            });
        })
    });

    ps.with_start_of_line(true, |ps| ps.emit_end());
}

fn format_source_encoding_node(
    ps: &mut ParserState,
    source_encoding_node: prism::SourceEncodingNode,
) {
    handle_string_at_offset(
        ps,
        "__ENCODING__".to_string(),
        source_encoding_node.location().start_offset(),
    );
}

fn format_source_file_node(ps: &mut ParserState, source_file_node: prism::SourceFileNode) {
    handle_string_at_offset(
        ps,
        "__FILE__".to_string(),
        source_file_node.location().start_offset(),
    );
}

fn format_source_line_node(ps: &mut ParserState, source_line_node: prism::SourceLineNode) {
    handle_string_at_offset(
        ps,
        "__LINE__".to_string(),
        source_line_node.location().start_offset(),
    );
}

fn format_self_node(ps: &mut ParserState) {
    ps.emit_ident("self".to_string());
}

fn format_true_node(ps: &mut ParserState, true_node: prism::TrueNode) {
    handle_string_at_offset(ps, "true".to_string(), true_node.location().start_offset());
}

fn format_undef_node(ps: &mut ParserState, undef_node: prism::UndefNode) {
    let names = undef_node.names();
    let end_offset = names
        .iter()
        .last()
        .expect("`undef` must have at least one argument")
        .location()
        .end_offset();

    ps.emit_ident("undef ".to_string());
    ps.with_start_of_line(false, |ps| {
        format_list_like_thing(ps, names, end_offset, true);
    });
}

fn format_unless_node(ps: &mut ParserState, unless_node: prism::UnlessNode) {
    format_conditional_node(ps, "unless", true, &Conditional::Unless(unless_node));
}

fn format_until_node(ps: &mut ParserState, until_node: prism::UntilNode) {
    format_conditional_node(ps, "until", true, &Conditional::Until(until_node));
}

fn format_when_node(ps: &mut ParserState, when_node: prism::WhenNode) {
    ps.at_offset(when_node.location().start_offset());
    ps.emit_indent();
    ps.emit_when_keyword();

    ps.with_start_of_line(false, |ps| {
        ps.new_block(|ps| {
            ps.inline_breakable_of(BreakableDelims::for_when(), |ps| {
                ps.emit_collapsing_newline();
                format_list_like_thing(
                    ps,
                    when_node.conditions(),
                    when_node.location().end_offset(),
                    false,
                );
            });
        });
    });

    ps.new_block(|ps| {
        ps.with_start_of_line(true, |ps| {
            ps.emit_newline();
            if let Some(statements) = when_node.statements() {
                format_node(ps, statements.as_node());
            }
        });
    });
}

fn format_while_node(ps: &mut ParserState, while_node: prism::WhileNode) {
    format_conditional_node(ps, "while", true, &Conditional::While(while_node));
}

fn format_x_string_node(ps: &mut ParserState, x_string_node: prism::XStringNode) {
    ps.emit_ident("`".to_string());
    ps.emit_string_content(loc_to_string(x_string_node.content_loc()));
    ps.emit_ident("`".to_string());
}

fn format_yield_node(ps: &mut ParserState, yield_node: prism::YieldNode) {
    ps.emit_ident("yield".to_string());
    if let Some(arguments) = yield_node.arguments() {
        let use_parens = ps.current_formatting_context_requires_parens()
            || yield_node.lparen_loc().is_some()
            // For single assoc values (`yield a: b`) we keep parens
            || (yield_node
                .arguments()
                .map(|args| {
                    args.arguments().len() == 1
                        && args
                            .arguments()
                            .iter()
                            .last()
                            .unwrap()
                            .as_keyword_hash_node()
                            .is_some()
                })
                .unwrap_or(false));

        let delims = if use_parens {
            BreakableDelims::for_method_call()
        } else {
            BreakableDelims::for_kw()
        };

        ps.breakable_of(delims, |ps| {
            format_arguments_node(ps, arguments);
        });
    }
}

fn handle_string_at_offset(ps: &mut ParserState, ident: String, offset: usize) {
    ps.at_offset(offset);
    ps.emit_ident(ident);
}

fn non_null_positions(params: &prism::ParametersNode) -> Vec<bool> {
    vec![
        !params.requireds().is_empty(),
        !params.optionals().is_empty(),
        params.rest().is_some(),
        !params.posts().is_empty(),
        !params.keywords().is_empty(),
        params.keyword_rest().is_some(),
        params.block().is_some(),
    ]
}

/// Checks if a node is an empty ParenthesesNode (like `()` in `foo ()`).
/// These should be treated as "no arguments" and the parens removed entirely.
fn is_empty_parentheses_node(node: &prism::Node) -> bool {
    if let Some(paren_node) = node.as_parentheses_node() {
        match paren_node.body() {
            None => true,
            Some(body) => {
                if let Some(statements) = body.as_statements_node() {
                    statements.body().is_empty()
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}

fn format_list_like_thing(
    ps: &mut ParserState,
    node_list: prism::NodeList,
    end_offset: SourceOffset,
    single_line: bool,
) -> bool {
    let mut emitted_args = false;
    let args_count = node_list.len();

    ps.magic_handle_comments_for_multiline_arrays(
        Some(ps.get_line_number_for_offset(end_offset)),
        |ps| {
            for (idx, expr) in node_list.iter().enumerate() {
                if single_line {
                    format_node(ps, expr);
                    if idx != args_count - 1 {
                        ps.emit_comma_space();
                    }
                } else {
                    ps.with_start_of_line(false, |ps| {
                        if let Some(assoc_node) = expr.as_assoc_node() {
                            if idx > 0 {
                                ps.emit_soft_indent();
                            }
                            format_assoc_node(ps, assoc_node)
                        } else if let Some(splat_node) = expr.as_assoc_splat_node() {
                            if idx > 0 {
                                ps.emit_soft_indent();
                            }
                            format_assoc_splat_node(ps, splat_node)
                        } else {
                            ps.emit_soft_indent();
                            format_node(ps, expr);
                        }

                        if idx != args_count - 1 {
                            ps.emit_comma();
                            ps.emit_soft_newline();
                        } else {
                            ps.shift_comments();
                        }
                    });
                };
                emitted_args = true;
            }
        },
    );
    emitted_args
}

fn format_write_node(
    ps: &mut ParserState,
    name: String,
    op: Cow<'static, str>,
    value: prism::Node,
) {
    ps.emit_ident(name);
    ps.emit_space();
    ps.emit_op(op);
    ps.emit_space();
    ps.with_start_of_line(false, |ps| format_node(ps, value));
}
