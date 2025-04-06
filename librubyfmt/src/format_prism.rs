use ruby_prism as prism;

use crate::{
    delimiters::BreakableDelims,
    format::SpecialCase,
    heredoc_string::HeredocKind,
    parser_state::{ConcreteParserState, FormattingContext, HashType, RenderFunc},
    render_targets::MultilineHandling,
    types::SourceOffset,
    util::{const_to_string, loc_to_string},
};

pub fn format_node(ps: &mut dyn ConcreteParserState, node: prism::Node) {
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
        Node::CallNode { .. } => format_call_node(ps, node.as_call_node().unwrap(), false),
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
        Node::ImplicitNode { .. } => format_implicit_node(ps, node.as_implicit_node().unwrap()),
        Node::ImplicitRestNode { .. } => {
            format_implicit_rest_node(ps, node.as_implicit_rest_node().unwrap())
        }
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
        Node::ItParametersNode { .. } => {
            format_it_parameters_node(ps, node.as_it_parameters_node().unwrap())
        }
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
        Node::NilNode { .. } => format_nil_node(ps, node.as_nil_node().unwrap()),
        Node::NoKeywordsParameterNode { .. } => {
            format_no_keywords_parameter_node(ps, node.as_no_keywords_parameter_node().unwrap())
        }
        Node::NumberedParametersNode { .. } => {
            format_numbered_parameters_node(ps, node.as_numbered_parameters_node().unwrap())
        }
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
        Node::RedoNode { .. } => format_redo_node(ps, node.as_redo_node().unwrap()),
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
        Node::RetryNode { .. } => format_retry_node(ps, node.as_retry_node().unwrap()),
        Node::ReturnNode { .. } => format_return_node(ps, node.as_return_node().unwrap()),
        Node::SelfNode { .. } => format_self_node(ps, node.as_self_node().unwrap()),
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
    _ps: &mut dyn ConcreteParserState,
    _alias_global_variable_node: prism::AliasGlobalVariableNode,
) {
    todo!()
}

fn format_alias_method_node(
    _ps: &mut dyn ConcreteParserState,
    _alias_method_node: prism::AliasMethodNode,
) {
    todo!()
}

fn format_alternation_pattern_node(
    _ps: &mut dyn ConcreteParserState,
    _alternation_pattern_node: prism::AlternationPatternNode,
) {
    todo!()
}

fn format_and_node(_ps: &mut dyn ConcreteParserState, _and_node: prism::AndNode) {
    todo!()
}

fn format_back_reference_read_node(
    _ps: &mut dyn ConcreteParserState,
    _back_reference_read_node: prism::BackReferenceReadNode,
) {
    todo!()
}

fn format_begin_node(_ps: &mut dyn ConcreteParserState, _begin_node: prism::BeginNode) {
    todo!()
}

fn format_break_node(_ps: &mut dyn ConcreteParserState, _break_node: prism::BreakNode) {
    todo!()
}

fn format_capture_pattern_node(
    _ps: &mut dyn ConcreteParserState,
    _capture_pattern_node: prism::CapturePatternNode,
) {
    todo!()
}

fn format_case_match_node(
    _ps: &mut dyn ConcreteParserState,
    _case_match_node: prism::CaseMatchNode,
) {
    todo!()
}

fn format_case_node(_ps: &mut dyn ConcreteParserState, _case_node: prism::CaseNode) {
    todo!()
}

pub fn format_program(
    ps: &mut dyn ConcreteParserState,
    program_node: prism::ProgramNode,
    data_loc: Option<prism::Location>,
) {
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            format_statements(ps, program_node.statements());
        }),
    );
    ps.emit_newline();
    ps.on_line(10000000000);
    ps.shift_comments();

    if let Some(data) = data_loc {
        ps.emit_data(&loc_to_string(data));
    }
}

fn format_statements(ps: &mut dyn ConcreteParserState, statements_node: prism::StatementsNode) {
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            for node in statements_node.body().iter() {
                format_node(ps, node);
            }
        }),
    );
}

fn format_string_node(ps: &mut dyn ConcreteParserState, string_node: prism::StringNode) {
    ps.at_offset(string_node.location().start_offset());

    // `opening_loc()` is only `None` in the case of the inner parts of multiline strings
    // (e.g. the inner contents of a heredoc)
    let opener = string_node
        .opening_loc()
        .map(|s| loc_to_string(s).trim().to_string());
    let closer = string_node
        .closing_loc()
        .map(|s| loc_to_string(s).trim().to_string());
    let is_heredoc = opener.clone().map(|s| s.starts_with("<")).unwrap_or(false);

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if is_heredoc {
                let heredoc_symbol = opener.clone().unwrap();
                let heredoc_kind = HeredocKind::from_string(&heredoc_symbol);

                ps.emit_heredoc_start(heredoc_symbol, heredoc_kind);
                ps.emit_newline();
            } else {
                // Always use double quotes over single quotes/percent literals
                if opener.is_some() {
                    ps.emit_double_quote();
                }
            }

            // If opener is nil, we must be in some kind of interpolated string context, which
            // means the contents must already be appropriately escaped -- hence we default to `true` here
            let in_escaped_context =
                is_heredoc || opener.clone().map(|s| s.starts_with("\"")).unwrap_or(true);
            let string_content = if in_escaped_context {
                loc_to_string(string_node.content_loc())
            } else {
                crate::string_escape::single_to_double_quoted(
                    loc_to_string(string_node.content_loc()),
                    opener.clone().unwrap().as_str(),
                    closer.clone().unwrap().as_str(),
                )
            };

            ps.emit_string_content(string_content);
            ps.wind_dumping_comments_until_offset(string_node.content_loc().end_offset());

            if is_heredoc {
                // <<~ heredocs have their closing tag indented too
                if opener.map(|s| s.starts_with("<<~")).unwrap_or(false) {
                    ps.emit_indent();
                }

                ps.emit_heredoc_close(
                    loc_to_string(string_node.closing_loc().unwrap())
                        .trim()
                        .to_string(),
                );
            } else if opener.is_some() {
                ps.emit_double_quote();
            }
        }),
    );

    ps.wind_dumping_comments_until_offset(string_node.location().end_offset());
}

fn format_interpolated_string_node(
    ps: &mut dyn ConcreteParserState,
    interpolated_string_node: prism::InterpolatedStringNode,
) {
    let is_heredoc = interpolated_string_node
        .opening_loc()
        .map(|s| loc_to_string(s).starts_with("<"))
        .unwrap_or(false);

    // Prism actually handles string concatenation when using "\", so it treats
    // ```ruby
    // "foo" \
    //   "bar"
    // ```
    // as an interpolated node with the contents `"foobar"` (in two `parts` of "foo" and "bar").
    // To detect this, we can look for any `InterpolatedStringNode` that has multiple `parts`
    // and isn't a heredoc.
    let is_backslash_string_interpolation = !is_heredoc
        && interpolated_string_node.parts().iter().all(|node| {
            node.as_string_node()
                .map(|s| s.opening_loc().is_some())
                .unwrap_or(false)
        });

    ps.at_offset(interpolated_string_node.location().start_offset());
    if let Some(s) = interpolated_string_node.opening_loc() {
        ps.emit_string_content(loc_to_string(s).trim().to_string());
    }
    if is_heredoc {
        ps.emit_newline();
    }

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            let string_parts_count = interpolated_string_node.parts().iter().count();
            for (i, part) in interpolated_string_node.parts().iter().enumerate() {
                let end_offset = part.location().end_offset();
                ps.at_offset(part.location().start_offset());

                if i > 0 {
                    ps.start_indent();

                    if is_backslash_string_interpolation {
                        ps.emit_newline();
                        ps.emit_indent();
                    }
                }

                format_node(ps, part);

                // For non-backslash-concatenated multiline strings, `part` contains newlines and indentation,
                // so we don't need to handle that ourselves.
                if is_backslash_string_interpolation && i < string_parts_count - 1 {
                    if let Some(s) = interpolated_string_node.closing_loc() {
                        ps.emit_string_content(loc_to_string(s).trim().to_string());
                    }
                    ps.emit_space();
                    ps.emit_slash();
                }

                ps.at_offset(end_offset);
                if i > 0 {
                    ps.end_indent();
                }
            }
        }),
    );

    if let Some(closing_loc) = interpolated_string_node.closing_loc() {
        ps.emit_string_content(loc_to_string(closing_loc).trim().to_string());
    }
}

fn format_interpolated_symbol_node(
    _ps: &mut dyn ConcreteParserState,
    _interpolated_string_node: prism::InterpolatedSymbolNode,
) {
    todo!()
}

fn format_interpolated_x_string_node(
    _ps: &mut dyn ConcreteParserState,
    _interpolated_x_string_node: prism::InterpolatedXStringNode,
) {
    todo!()
}

fn format_it_local_variable_read_node(
    _ps: &mut dyn ConcreteParserState,
    _it_local_variable_read_node: prism::ItLocalVariableReadNode,
) {
    todo!()
}

fn format_it_parameters_node(
    _ps: &mut dyn ConcreteParserState,
    _it_parameters_node: prism::ItParametersNode,
) {
    todo!()
}

fn format_interpolated_last_line_node(
    _ps: &mut dyn ConcreteParserState,
    _interpolated_match_last_line_node: prism::InterpolatedMatchLastLineNode,
) {
    todo!()
}

fn format_interpolated_regular_expression_node(
    _ps: &mut dyn ConcreteParserState,
    _interpolated_regular_expression_node: prism::InterpolatedRegularExpressionNode,
) {
    todo!()
}

fn format_embedded_statements_node(
    ps: &mut dyn ConcreteParserState,
    embedded_statements_node: prism::EmbeddedStatementsNode,
) {
    ps.emit_string_content("#{".to_string());
    if let Some(statements) = embedded_statements_node.statements() {
        let has_multiple_statements = statements.body().iter().count() > 1;
        ps.with_start_of_line(
            has_multiple_statements,
            Box::new(|ps| {
                if has_multiple_statements {
                    ps.emit_newline();
                    ps.new_block(Box::new(|ps| format_node(ps, statements.as_node())));
                    ps.emit_indent();
                } else {
                    format_node(ps, statements.as_node());
                }
            }),
        );
    }
    ps.emit_string_content("}".to_string());
}

fn format_embedded_variable_node(
    _ps: &mut dyn ConcreteParserState,
    _embedded_variable_node: prism::EmbeddedVariableNode,
) {
    todo!()
}

fn format_ensure_node(_ps: &mut dyn ConcreteParserState, _ensure_node: prism::EnsureNode) {
    todo!()
}

fn format_false_node(_ps: &mut dyn ConcreteParserState, _false_node: prism::FalseNode) {
    todo!()
}

fn format_find_pattern_node(
    _ps: &mut dyn ConcreteParserState,
    _find_pattern_node: prism::FindPatternNode,
) {
    todo!()
}

fn format_flip_flop_node(_ps: &mut dyn ConcreteParserState, _flip_flop_node: prism::FlipFlopNode) {
    todo!()
}

fn format_class_node(ps: &mut dyn ConcreteParserState, class_node: prism::ClassNode) {
    ps.emit_class_keyword();
    ps.emit_space();
    ps.with_start_of_line(
        false,
        Box::new(|ps| format_node(ps, class_node.constant_path())),
    );

    if let Some(superclass) = class_node.superclass() {
        ps.emit_ident("<".to_string());
        ps.emit_space();
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                format_node(ps, superclass);
            }),
        );
    }
    ps.emit_newline();

    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                if let Some(body) = class_node.body() {
                    format_node(ps, body);
                }
            }),
        )
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );
}

fn format_class_variable_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_and_write_node: prism::ClassVariableAndWriteNode,
) {
    todo!()
}

fn format_class_variable_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_operator_write_node: prism::ClassVariableOperatorWriteNode,
) {
    todo!()
}

fn format_class_variable_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_or_write_node: prism::ClassVariableOrWriteNode,
) {
    todo!()
}

fn format_class_variable_read_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_read_node: prism::ClassVariableReadNode,
) {
    todo!()
}

fn format_class_variable_target_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_target_node: prism::ClassVariableTargetNode,
) {
    todo!()
}

fn format_class_variable_write_node(
    _ps: &mut dyn ConcreteParserState,
    _class_variable_write_node: prism::ClassVariableWriteNode,
) {
    todo!()
}

fn format_module_node(ps: &mut dyn ConcreteParserState, module_node: prism::ModuleNode) {
    ps.emit_module_keyword();
    ps.emit_space();
    ps.with_start_of_line(
        false,
        Box::new(|ps| format_node(ps, module_node.constant_path())),
    );
    ps.emit_newline();

    ps.new_block(Box::new(|ps| {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                if let Some(body) = module_node.body() {
                    format_node(ps, body);
                }
            }),
        )
    }));

    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            ps.emit_end();
        }),
    );
}

fn format_def_node(ps: &mut dyn ConcreteParserState, def_node: prism::DefNode) {
    ps.emit_keyword("def".to_string());
    ps.emit_space();

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if let Some(receiver) = def_node.receiver() {
                format_node(ps, receiver);
                ps.emit_dot();
            }

            handle_string_at_offset(
                ps,
                const_to_string(def_node.name()),
                def_node.name_loc().end_offset(),
            );
        }),
    );

    format_def_body(
        ps,
        def_node.parameters(),
        def_node.body(),
        def_node
            .end_keyword_loc()
            .map(|loc| loc.end_offset())
            .unwrap(),
        def_node.end_keyword_loc().is_some(),
    );
}

fn format_def_body(
    ps: &mut dyn ConcreteParserState,
    parameters_node: Option<prism::ParametersNode>,
    bodystmt: Option<prism::Node>,
    end_offset: SourceOffset,
    has_end_keyword: bool,
) {
    ps.new_scope(Box::new(|ps| {
        if let Some(parameters_node) = parameters_node {
            ps.breakable_of(
                BreakableDelims::for_method_call(),
                Box::new(|ps| {
                    ps.with_start_of_line(
                        false,
                        Box::new(|ps| {
                            format_parameters_node(ps, parameters_node);
                        }),
                    );
                }),
            );
        }

        ps.with_formatting_context(
            FormattingContext::Def,
            Box::new(|ps| {
                if has_end_keyword {
                    ps.new_block(Box::new(|ps| {
                        ps.emit_newline();
                        ps.with_start_of_line(
                            true,
                            Box::new(|ps| {
                                if let Some(body) = bodystmt {
                                    format_node(ps, body);
                                }
                            }),
                        );
                    }));
                } else {
                    ps.emit_space();
                    ps.emit_op("=".to_string());
                    ps.emit_space();

                    ps.with_start_of_line(
                        false,
                        Box::new(|ps| {
                            if let Some(body) = bodystmt {
                                format_node(ps, body);
                            }
                        }),
                    )
                }
            }),
        );
    }));

    if has_end_keyword {
        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                ps.wind_dumping_comments_until_offset(end_offset);
                ps.emit_end();
            }),
        );
    }
}

fn format_defined_node(_ps: &mut dyn ConcreteParserState, _defined_node: prism::DefinedNode) {
    todo!()
}

fn format_else_node(_ps: &mut dyn ConcreteParserState, _else_node: prism::ElseNode) {
    todo!()
}

type ParamFormattingFunc<'a> = Box<dyn FnOnce(&mut dyn ConcreteParserState) + 'a>;

fn format_parameters_node(ps: &mut dyn ConcreteParserState, params: prism::ParametersNode) {
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
    let requireds = params.requireds();
    let optionals = params.optionals();
    let rest = params.rest();
    let posts = params.posts();
    let keywords = params.keywords();
    let keyword_rest = params.keyword_rest();
    let block = params.block();

    let formats: Vec<ParamFormattingFunc> = vec![
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if node_list_is_empty(&requireds) {
                return;
            }
            let end_offset = requireds.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, requireds, end_offset, false);
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if node_list_is_empty(&optionals) {
                return;
            }
            let end_offset = optionals.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, optionals, end_offset, false);
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if let Some(rest) = rest {
                format_rest_param(
                    ps,
                    rest.as_rest_parameter_node().unwrap(),
                    SpecialCase::NoSpecialCase,
                )
            }
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if node_list_is_empty(&posts) {
                return;
            }
            let end_offset = posts.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, posts, end_offset, false);
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if node_list_is_empty(&keywords) {
                return;
            }
            let end_offset = keywords.iter().last().unwrap().location().end_offset();
            format_list_like_thing(ps, keywords, end_offset, false);
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if let Some(keyword_rest) = keyword_rest {
                format_node(ps, keyword_rest);
            }
        }),
        Box::new(move |ps: &mut dyn ConcreteParserState| {
            if let Some(block) = block {
                format_block_parameter_node(ps, block);
            }
        }),
    ];

    for (idx, format_fn) in formats.into_iter().enumerate() {
        format_fn(ps);
        let did_emit = non_null_positions[idx];
        let have_more = non_null_positions[idx + 1..].iter().any(|&v| v);

        if did_emit && have_more {
            ps.emit_comma();
            ps.emit_soft_newline();
        }
        ps.shift_comments();
    }
}

fn format_block_parameter_node(
    ps: &mut dyn ConcreteParserState,
    block_arg: prism::BlockParameterNode,
) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.emit_soft_indent();
            ps.emit_ident("&".to_string());
            if let Some(ident) = block_arg.name() {
                let ident_str = const_to_string(ident);
                ps.bind_variable(ident_str.clone());
                format_ident(ps, ident_str, block_arg.name_loc().unwrap().end_offset());
            }
        }),
    );
}

fn format_block_argument_node(
    _ps: &mut dyn ConcreteParserState,
    _block_argument_node: prism::BlockArgumentNode,
) {
    todo!()
}

fn format_call_node(
    ps: &mut dyn ConcreteParserState,
    call_node: prism::CallNode,
    skip_receiver: bool,
) {
    if skip_receiver || call_node.receiver().is_none() {
        handle_string_at_offset(
            ps,
            const_to_string(call_node.name()),
            call_node.message_loc().unwrap().start_offset(),
        );
        if let Some(arguments) = call_node.arguments() {
            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    ps.breakable_of(
                        BreakableDelims::for_method_call(),
                        Box::new(|ps| {
                            format_arguments_node(ps, arguments);
                        }),
                    );
                }),
            );
        }
        if let Some(block) = call_node.block() {
            ps.emit_space();
            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    format_node(ps, block);
                }),
            );
        }
    } else {
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                let mut call_chain_elements = collapse_nodes_to_call_chain(call_node.as_node());
                ps.breakable_call_chain_of(
                    MultilineHandling::Prism(call_chain_elements_are_user_multilined(
                        ps,
                        call_chain_elements.iter().clone().collect(),
                    )),
                    Box::new(|ps| {
                        // The first node can be *any* expression, whereas following receivers
                        // must be additional calls -- you cannot insert literals into call chains
                        let first_expression = call_chain_elements.remove(0);
                        format_node(ps, first_expression);

                        ps.start_indent_for_call_chain();

                        ps.with_start_of_line(
                            false,
                            Box::new(|ps| {
                                for element in call_chain_elements {
                                    let element = element.as_call_node().unwrap();
                                    let call_operator = loc_to_string(
                                        // `call_operator_loc` is the `.`/`::`/`&.` etc.
                                        element.call_operator_loc().expect("We're in the middle of the chain, there must be an operator"),
                                    );
                                    if call_operator != *"::" {
                                        ps.emit_collapsing_newline();
                                        ps.emit_soft_indent();
                                    }
                                    ps.emit_ident(call_operator);

                                    ps.at_offset(element.location().start_offset());
                                    format_call_node(ps, element, true);
                                }
                            }),
                        );
                        ps.end_indent_for_call_chain();
                    }),
                );
            }),
        );

        ps.emit_after_call_chain();
    }
}

fn call_chain_elements_are_user_multilined(
    ps: &dyn ConcreteParserState,
    call_chain_elements: Vec<&prism::Node>,
) -> bool {
    // Making a mutable copy since we may pop some items off later
    let mut call_chain_elements = call_chain_elements.as_slice();

    if call_chain_elements.len() > 1 {
        // If the first item in the chain is a multiline expression (like a hash or array),
        // ignore it when checking line length.
        let is_literal_expression = !matches!(
            call_chain_elements.first().unwrap(),
            prism::Node::CallNode { .. }
                | prism::Node::ConstantReadNode { .. }
                | prism::Node::ConstantPathNode { .. }
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
            call_chain_elements[1]
                .as_call_node()
                .unwrap()
                .message_loc()
                .unwrap()
                .start_offset(),
        );
        if is_literal_expression && !has_comment {
            call_chain_elements = &call_chain_elements[1..];
        }
    }

    let start_line = ps.get_line_number_for_offset(
        call_chain_elements
            .first()
            .unwrap()
            .location()
            .start_offset(),
    );
    !call_chain_elements[1..].iter().all(|cce| {
        start_line
            == ps.get_line_number_for_offset(
                cce.as_call_node()
                    .unwrap()
                    .call_operator_loc()
                    .unwrap()
                    .start_offset(),
            )
    })
}

fn format_call_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _call_and_write_node: prism::CallAndWriteNode,
) {
    todo!()
}

fn format_call_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _call_operator_write_node: prism::CallOperatorWriteNode,
) {
    todo!()
}

fn format_call_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _call_or_write_node: prism::CallOrWriteNode,
) {
    todo!()
}

fn format_call_target_node(
    _ps: &mut dyn ConcreteParserState,
    _call_target_node: prism::CallTargetNode,
) {
    todo!()
}

fn format_symbol_node(ps: &mut dyn ConcreteParserState, symbol_node: prism::SymbolNode) {
    if let Some(opening_loc) = symbol_node.opening_loc() {
        ps.emit_ident(loc_to_string(opening_loc));
    }
    if let Some(value_loc) = symbol_node.value_loc() {
        ps.emit_ident(loc_to_string(value_loc));
    }
}

fn format_assoc_node(ps: &mut dyn ConcreteParserState, assoc_node: prism::AssocNode) {
    let as_symbol = if let Some(hash_type) = ps.hash_type_from_formatting_context() {
        matches!(hash_type, HashType::SymbolKey)
    } else {
        // `operator_loc` is only present for hash rockets, not for symbol keys
        assoc_node.operator_loc().is_none()
    };

    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_node(ps, assoc_node.key());
            if as_symbol {
                ps.emit_ident(":".to_string());
            } else {
                ps.emit_space();
                ps.emit_ident("=>".to_string());
            }
            ps.emit_space();
            format_node(ps, assoc_node.value());
        }),
    );
}

fn format_assoc_splat_node(
    _ps: &mut dyn ConcreteParserState,
    _assoc_splat_node: prism::AssocSplatNode,
) {
    todo!()
}

fn format_block_node(ps: &mut dyn ConcreteParserState, block_node: prism::BlockNode) {
    if &loc_to_string(block_node.opening_loc()) == "do" {
        ps.new_block(Box::new(|ps| {
            ps.emit_do_keyword();
            if let Some(block_parameters) = block_node.parameters() {
                format_node(ps, block_parameters);
            }

            if let Some(body) = block_node.body() {
                ps.emit_newline();
                ps.with_start_of_line(
                    true,
                    Box::new(|ps| {
                        format_node(ps, body);
                    }),
                );
            }
        }));

        ps.with_start_of_line(
            true,
            Box::new(|ps| {
                ps.wind_dumping_comments_until_offset(block_node.location().end_offset());
                ps.emit_end();
                ps.shift_comments();
            }),
        );
    } else {
        ps.inline_breakable_of(
            BreakableDelims::for_brace_block(),
            Box::new(|ps| {
                if let Some(parameters) = block_node.parameters() {
                    format_node(ps, parameters);
                }

                if let Some(body) = block_node.body() {
                    let has_multiple_statements = body
                        .as_statements_node()
                        .map(|statements_node| statements_node.body().iter().count() > 1)
                        .unwrap_or(false);
                    if has_multiple_statements {
                        ps.emit_newline();
                        ps.with_start_of_line(
                            true,
                            Box::new(|ps| {
                                format_node(ps, body);
                            }),
                        );
                    } else {
                        ps.with_start_of_line(
                            false,
                            Box::new(|ps| {
                                if let Some(node) =
                                    body.as_statements_node().unwrap().body().iter().next()
                                {
                                    ps.emit_soft_newline();
                                    ps.emit_soft_indent();
                                    format_node(ps, node);
                                    ps.emit_soft_newline();
                                }
                            }),
                        );
                    }
                }

                // `inline_breakable_of` doesn't handle the indentation for the closing delimeter for us.
                ps.dedent(Box::new(|ps| ps.emit_soft_indent()));
                ps.wind_dumping_comments_until_offset(block_node.location().end_offset());
            }),
        );
    }
}

fn format_block_parameters_node(
    ps: &mut dyn ConcreteParserState,
    block_parameters_node: prism::BlockParametersNode,
) {
    ps.breakable_of(
        BreakableDelims::for_block_params(),
        Box::new(|ps| {
            let has_locals = !node_list_is_empty(&block_parameters_node.locals());

            if let Some(parameters) = block_parameters_node.parameters() {
                format_parameters_node(ps, parameters);
            }
            if has_locals {
                ps.emit_ident(";".to_string());
                ps.emit_space();
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        format_list_like_thing(
                            ps,
                            block_parameters_node.locals(),
                            block_parameters_node.location().end_offset(),
                            false,
                        );
                    }),
                );
            }
        }),
    );
}

fn format_block_local_variable_node(
    ps: &mut dyn ConcreteParserState,
    block_local_variable_node: prism::BlockLocalVariableNode,
) {
    handle_string_at_offset(
        ps,
        const_to_string(block_local_variable_node.name()),
        block_local_variable_node.location().start_offset(),
    );
}

fn format_array_node(ps: &mut dyn ConcreteParserState, array_node: prism::ArrayNode) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            ps.breakable_of(
                BreakableDelims::for_array(),
                Box::new(|ps| {
                    format_list_like_thing(
                        ps,
                        array_node.elements(),
                        array_node.location().end_offset(),
                        false,
                    );
                    ps.wind_dumping_comments_until_offset(array_node.location().end_offset());
                }),
            );
        }),
    );
}

fn format_array_pattern_node(
    _ps: &mut dyn ConcreteParserState,
    _array_pattern_node: prism::ArrayPatternNode,
) {
    todo!()
}

fn format_parentheses_node(
    ps: &mut dyn ConcreteParserState,
    parentheses_node: prism::ParenthesesNode,
) {
    ps.emit_open_paren();
    if let Some(body) = parentheses_node.body() {
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                format_node(ps, body);
            }),
        );
    }
    ps.emit_close_paren();
}

fn collapse_nodes_to_call_chain(node: prism::Node) -> Vec<prism::Node> {
    let mut call_chain_elements = vec![];
    let mut maybe_receiver = Some(node);
    while let Some(receiver) = maybe_receiver {
        maybe_receiver = receiver
            .as_call_node()
            .and_then(|call_node| call_node.receiver());
        call_chain_elements.insert(0, receiver);
    }

    call_chain_elements
}

fn format_rest_param(
    ps: &mut dyn ConcreteParserState,
    rest_param: prism::RestParameterNode,
    special_case: SpecialCase,
) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if special_case != SpecialCase::RestParamOutsideOfParamDef {
                ps.emit_soft_indent();
            }
            ps.emit_ident("*".to_string());
            ps.with_start_of_line(
                false,
                Box::new(|ps| {
                    if let Some(name) = rest_param.name() {
                        let name_str = const_to_string(name);
                        ps.bind_variable(name_str.clone());
                        format_ident(ps, name_str, rest_param.name_loc().unwrap().end_offset());
                    }
                }),
            );
        }),
    );
}

fn format_arguments_node(ps: &mut dyn ConcreteParserState, arguments_node: prism::ArgumentsNode) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            format_list_like_thing(
                ps,
                arguments_node.arguments(),
                arguments_node.location().end_offset(),
                false,
            );
        }),
    );
}

fn format_keyword_hash_node(
    ps: &mut dyn ConcreteParserState,
    keyword_hash_node: prism::KeywordHashNode,
) {
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

    ps.with_formatting_context(
        FormattingContext::HashType(hash_type),
        Box::new(|ps| {
            format_list_like_thing(
                ps,
                keyword_hash_node.elements(),
                keyword_hash_node.location().end_offset(),
                false,
            );
        }),
    );
}

fn format_keyword_rest_parameter_node(
    ps: &mut dyn ConcreteParserState,
    keyword_rest_parameter_node: prism::KeywordRestParameterNode,
) {
    ps.emit_ident("**".to_string());
    if let Some(constant_id) = keyword_rest_parameter_node.name() {
        let name = const_to_string(constant_id);
        ps.bind_variable(name.clone());
        ps.emit_ident(name);
    }
}

fn format_required_keyword_parameter_node(
    ps: &mut dyn ConcreteParserState,
    required_keyword_parameter_node: prism::RequiredKeywordParameterNode,
) {
    let name = const_to_string(required_keyword_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
    ps.emit_ident(":".to_string());
}

fn format_required_parameter_node(
    ps: &mut dyn ConcreteParserState,
    required_parameter_node: prism::RequiredParameterNode,
) {
    let name = const_to_string(required_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
}

fn format_local_variable_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _local_variable_and_write_node: prism::LocalVariableAndWriteNode,
) {
    todo!()
}

fn format_local_variable_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _local_variable_operator_write_node: prism::LocalVariableOperatorWriteNode,
) {
    todo!()
}

fn format_local_variable_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _local_variable_or_write_node: prism::LocalVariableOrWriteNode,
) {
    todo!()
}

fn format_local_variable_target_node(
    _ps: &mut dyn ConcreteParserState,
    _local_variable_target_node: prism::LocalVariableTargetNode,
) {
    todo!()
}

fn format_local_variable_read_node(
    ps: &mut dyn ConcreteParserState,
    local_variable_read_node: prism::LocalVariableReadNode,
) {
    let name = const_to_string(local_variable_read_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
}

fn format_local_variable_write_node(
    ps: &mut dyn ConcreteParserState,
    local_variable_write_node: prism::LocalVariableWriteNode,
) {
    let name = const_to_string(local_variable_write_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);

    ps.emit_ident("=".to_string());

    ps.with_start_of_line(
        false,
        Box::new(|ps| format_node(ps, local_variable_write_node.value())),
    );
}

fn format_splat_node(ps: &mut dyn ConcreteParserState, splat_node: prism::SplatNode) {
    ps.emit_ident("*".to_string());
    if let Some(node) = splat_node.expression() {
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                format_node(ps, node);
            }),
        );
    }
}

fn format_ident(ps: &mut dyn ConcreteParserState, ident: String, offset: usize) {
    handle_string_at_offset(ps, ident, offset);
}

fn format_instance_variable_write_node(
    ps: &mut dyn ConcreteParserState,
    instance_variable_write_node: prism::InstanceVariableWriteNode,
) {
    ps.at_offset(instance_variable_write_node.location().start_offset());

    ps.emit_ident(const_to_string(instance_variable_write_node.name()));
    ps.emit_space();
    ps.emit_op("=".to_string());
    ps.emit_space();
    ps.with_start_of_line(
        false,
        Box::new(|ps| format_node(ps, instance_variable_write_node.value())),
    );
}

fn format_integer_node(ps: &mut dyn ConcreteParserState, integer_node: prism::IntegerNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(integer_node.location()),
        integer_node.location().start_offset(),
    );
}

fn format_float_node(ps: &mut dyn ConcreteParserState, float_node: prism::FloatNode) {
    handle_string_at_offset(
        ps,
        loc_to_string(float_node.location()),
        float_node.location().start_offset(),
    );
}

fn format_for_node(_ps: &mut dyn ConcreteParserState, _for_node: prism::ForNode) {
    todo!()
}

fn format_forwarding_arguments_node(
    _ps: &mut dyn ConcreteParserState,
    _forwarding_arguments_node: prism::ForwardingArgumentsNode,
) {
    todo!()
}

fn format_forwarding_parameter_node(
    _ps: &mut dyn ConcreteParserState,
    _forwarding_parameter_node: prism::ForwardingParameterNode,
) {
    todo!()
}

fn format_forwarding_super_node(
    _ps: &mut dyn ConcreteParserState,
    _forwarding_super_node: prism::ForwardingSuperNode,
) {
    todo!()
}

fn format_global_variable_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_and_write_node: prism::GlobalVariableAndWriteNode,
) {
    todo!()
}

fn format_global_variable_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_operator_write_node: prism::GlobalVariableOperatorWriteNode,
) {
    todo!()
}

fn format_global_variable_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_or_write_node: prism::GlobalVariableOrWriteNode,
) {
    todo!()
}

fn format_global_variable_read_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_read_node: prism::GlobalVariableReadNode,
) {
    todo!()
}

fn format_global_variable_target_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_target_node: prism::GlobalVariableTargetNode,
) {
    todo!()
}

fn format_global_variable_write_node(
    _ps: &mut dyn ConcreteParserState,
    _global_variable_write_node: prism::GlobalVariableWriteNode,
) {
    todo!()
}

fn format_hash_node(_ps: &mut dyn ConcreteParserState, _hash_node: prism::HashNode) {
    todo!()
}

fn format_hash_pattern_node(
    _ps: &mut dyn ConcreteParserState,
    _hash_pattern_node: prism::HashPatternNode,
) {
    todo!()
}

fn format_if_node(_ps: &mut dyn ConcreteParserState, _if_node: prism::IfNode) {
    todo!()
}

fn format_imaginary_node(_ps: &mut dyn ConcreteParserState, _imaginary_node: prism::ImaginaryNode) {
    todo!()
}

fn format_implicit_node(_ps: &mut dyn ConcreteParserState, _implicit_node: prism::ImplicitNode) {
    todo!()
}

fn format_implicit_rest_node(
    _ps: &mut dyn ConcreteParserState,
    _implicit_rest_node: prism::ImplicitRestNode,
) {
    todo!()
}

fn format_in_node(_ps: &mut dyn ConcreteParserState, _in_node: prism::InNode) {
    todo!()
}

fn format_index_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _index_and_write_node: prism::IndexAndWriteNode,
) {
    todo!()
}

fn format_index_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _index_operator_write_node: prism::IndexOperatorWriteNode,
) {
    todo!()
}

fn format_index_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _index_or_write_node: prism::IndexOrWriteNode,
) {
    todo!()
}

fn format_index_target_node(
    _ps: &mut dyn ConcreteParserState,
    _index_target_node: prism::IndexTargetNode,
) {
    todo!()
}

fn format_instance_variable_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _instance_variable_and_write_node: prism::InstanceVariableAndWriteNode,
) {
    todo!()
}

fn format_instance_variable_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _instance_variable_operator_write_node: prism::InstanceVariableOperatorWriteNode,
) {
    todo!()
}

fn format_instance_variable_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _instance_variable_or_write_node: prism::InstanceVariableOrWriteNode,
) {
    todo!()
}

fn format_instance_variable_read_node(
    _ps: &mut dyn ConcreteParserState,
    _instance_variable_read_node: prism::InstanceVariableReadNode,
) {
    todo!()
}

fn format_instance_variable_target_node(
    _ps: &mut dyn ConcreteParserState,
    _instance_variable_target_node: prism::InstanceVariableTargetNode,
) {
    todo!()
}

fn format_constant_read_node(
    ps: &mut dyn ConcreteParserState,
    constant_read_node: prism::ConstantReadNode,
) {
    handle_string_at_offset(
        ps,
        const_to_string(constant_read_node.name()),
        constant_read_node.location().start_offset(),
    );
}

fn format_constant_path_node(
    ps: &mut dyn ConcreteParserState,
    constant_path_node: prism::ConstantPathNode,
) {
    ps.with_start_of_line(
        false,
        Box::new(|ps| {
            if let Some(parent) = constant_path_node.parent() {
                format_node(ps, parent);
                ps.emit_colon_colon();
            }

            handle_string_at_offset(
                ps,
                const_to_string(constant_path_node.name().unwrap()),
                constant_path_node.name_loc().start_offset(),
            );
        }),
    );
}

fn format_constant_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_and_write_node: prism::ConstantAndWriteNode,
) {
    todo!()
}

fn format_constant_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_operator_write_node: prism::ConstantOperatorWriteNode,
) {
    todo!()
}

fn format_constant_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_or_write_node: prism::ConstantOrWriteNode,
) {
    todo!()
}

fn format_constant_path_and_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_path_and_write_node: prism::ConstantPathAndWriteNode,
) {
    todo!()
}

fn format_constant_path_operator_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_path_operator_write_node: prism::ConstantPathOperatorWriteNode,
) {
    todo!()
}

fn format_constant_path_or_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_path_or_write_node: prism::ConstantPathOrWriteNode,
) {
    todo!()
}

fn format_constant_path_target_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_path_target_node: prism::ConstantPathTargetNode,
) {
    todo!()
}

fn format_constant_path_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_path_write_node: prism::ConstantPathWriteNode,
) {
    todo!()
}

fn format_constant_target_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_target_node: prism::ConstantTargetNode,
) {
    todo!()
}

fn format_constant_write_node(
    _ps: &mut dyn ConcreteParserState,
    _constant_write_node: prism::ConstantWriteNode,
) {
    todo!()
}

fn format_lambda_node(_ps: &mut dyn ConcreteParserState, _lambda_node: prism::LambdaNode) {
    todo!()
}

fn format_match_last_line_node(
    _ps: &mut dyn ConcreteParserState,
    _match_last_line_node: prism::MatchLastLineNode,
) {
    todo!()
}

fn format_match_predicate_node(
    _ps: &mut dyn ConcreteParserState,
    _match_predicate_node: prism::MatchPredicateNode,
) {
    todo!()
}

fn format_match_required_node(
    _ps: &mut dyn ConcreteParserState,
    _match_required_node: prism::MatchRequiredNode,
) {
    todo!()
}

fn format_match_write_node(
    _ps: &mut dyn ConcreteParserState,
    _match_write_node: prism::MatchWriteNode,
) {
    todo!()
}

fn format_multi_target_node(
    _ps: &mut dyn ConcreteParserState,
    _multi_target_node: prism::MultiTargetNode,
) {
    todo!()
}

fn format_multi_write_node(
    _ps: &mut dyn ConcreteParserState,
    _multi_write_node: prism::MultiWriteNode,
) {
    todo!()
}

fn format_next_node(_ps: &mut dyn ConcreteParserState, _next_node: prism::NextNode) {
    todo!()
}

fn format_nil_node(_ps: &mut dyn ConcreteParserState, _nil_node: prism::NilNode) {
    todo!()
}

fn format_no_keywords_parameter_node(
    _ps: &mut dyn ConcreteParserState,
    _no_keywords_parameter_node: prism::NoKeywordsParameterNode,
) {
    todo!()
}

fn format_numbered_parameters_node(
    _ps: &mut dyn ConcreteParserState,
    _numbered_parameters_node: prism::NumberedParametersNode,
) {
    todo!()
}

fn format_numbered_reference_read_node(
    _ps: &mut dyn ConcreteParserState,
    _numbered_reference_read_node: prism::NumberedReferenceReadNode,
) {
    todo!()
}

fn format_optional_keyword_parameter_node(
    _ps: &mut dyn ConcreteParserState,
    _optional_keyword_parameter_node: prism::OptionalKeywordParameterNode,
) {
    todo!()
}

fn format_optional_parameter_node(
    _ps: &mut dyn ConcreteParserState,
    _optional_parameter_node: prism::OptionalParameterNode,
) {
    todo!()
}

fn format_or_node(_ps: &mut dyn ConcreteParserState, _or_node: prism::OrNode) {
    todo!()
}

fn format_pinned_expression_node(
    _ps: &mut dyn ConcreteParserState,
    _pinned_expression_node: prism::PinnedExpressionNode,
) {
    todo!()
}

fn format_pinned_variable_node(
    _ps: &mut dyn ConcreteParserState,
    _pinned_variable_node: prism::PinnedVariableNode,
) {
    todo!()
}

fn format_post_execution_node(
    _ps: &mut dyn ConcreteParserState,
    _post_execution_node: prism::PostExecutionNode,
) {
    todo!()
}

fn format_pre_execution_node(
    _ps: &mut dyn ConcreteParserState,
    _pre_execution_node: prism::PreExecutionNode,
) {
    todo!()
}

fn format_range_node(_ps: &mut dyn ConcreteParserState, _range_node: prism::RangeNode) {
    todo!()
}

fn format_rational_node(_ps: &mut dyn ConcreteParserState, _rational_node: prism::RationalNode) {
    todo!()
}

fn format_redo_node(_ps: &mut dyn ConcreteParserState, _redo_node: prism::RedoNode) {
    todo!()
}

fn format_regular_expression_node(
    _ps: &mut dyn ConcreteParserState,
    _regular_expression_node: prism::RegularExpressionNode,
) {
    todo!()
}

fn format_rescue_modifier_node(
    _ps: &mut dyn ConcreteParserState,
    _rescue_modifier_node: prism::RescueModifierNode,
) {
    todo!()
}

fn format_rescue_node(_ps: &mut dyn ConcreteParserState, _rescue_node: prism::RescueNode) {
    todo!()
}

fn format_rest_parameter_node(
    _ps: &mut dyn ConcreteParserState,
    _rest_parameter_node: prism::RestParameterNode,
) {
    todo!()
}

fn format_retry_node(_ps: &mut dyn ConcreteParserState, _retry_node: prism::RetryNode) {
    todo!()
}

fn format_return_node(_ps: &mut dyn ConcreteParserState, _return_node: prism::ReturnNode) {
    todo!()
}

fn format_shareable_constant_node(
    _ps: &mut dyn ConcreteParserState,
    _shareable_constant_node: prism::ShareableConstantNode,
) {
    todo!()
}

fn format_singleton_class_node(
    _ps: &mut dyn ConcreteParserState,
    _singleton_class_node: prism::SingletonClassNode,
) {
    todo!()
}

fn format_source_encoding_node(
    ps: &mut dyn ConcreteParserState,
    source_encoding_node: prism::SourceEncodingNode,
) {
    handle_string_at_offset(
        ps,
        "__ENCODING__".to_string(),
        source_encoding_node.location().start_offset(),
    );
}

fn format_source_file_node(
    ps: &mut dyn ConcreteParserState,
    source_file_node: prism::SourceFileNode,
) {
    handle_string_at_offset(
        ps,
        "__FILE__".to_string(),
        source_file_node.location().start_offset(),
    );
}

fn format_source_line_node(
    ps: &mut dyn ConcreteParserState,
    source_line_node: prism::SourceLineNode,
) {
    handle_string_at_offset(
        ps,
        "__LINE__".to_string(),
        source_line_node.location().start_offset(),
    );
}

fn format_super_node(_ps: &mut dyn ConcreteParserState, _super_node: prism::SuperNode) {
    todo!()
}

fn format_self_node(ps: &mut dyn ConcreteParserState, _self_node: prism::SelfNode) {
    ps.emit_ident("self".to_string());
}

fn format_true_node(_ps: &mut dyn ConcreteParserState, _true_node: prism::TrueNode) {
    todo!()
}

fn format_undef_node(_ps: &mut dyn ConcreteParserState, _undef_node: prism::UndefNode) {
    todo!()
}

fn format_unless_node(_ps: &mut dyn ConcreteParserState, _unless_node: prism::UnlessNode) {
    todo!()
}

fn format_until_node(_ps: &mut dyn ConcreteParserState, _until_node: prism::UntilNode) {
    todo!()
}

fn format_when_node(_ps: &mut dyn ConcreteParserState, _when_node: prism::WhenNode) {
    todo!()
}

fn format_while_node(_ps: &mut dyn ConcreteParserState, _while_node: prism::WhileNode) {
    todo!()
}

fn format_x_string_node(_ps: &mut dyn ConcreteParserState, _x_string_node: prism::XStringNode) {
    todo!()
}

fn format_yield_node(_ps: &mut dyn ConcreteParserState, _yield_node: prism::YieldNode) {
    todo!()
}

fn handle_string_at_offset(ps: &mut dyn ConcreteParserState, ident: String, offset: usize) {
    ps.at_offset(offset);
    ps.emit_ident(ident);
}

fn non_null_positions(params: &prism::ParametersNode) -> Vec<bool> {
    vec![
        !node_list_is_empty(&params.requireds()),
        !node_list_is_empty(&params.optionals()),
        params.rest().is_some(),
        !node_list_is_empty(&params.posts()),
        !node_list_is_empty(&params.keywords()),
        params.keyword_rest().is_some(),
        params.block().is_some(),
    ]
}

fn node_list_is_empty(node_list: &prism::NodeList) -> bool {
    node_list.iter().next().is_none()
}

fn format_list_like_thing(
    ps: &mut dyn ConcreteParserState,
    node_list: prism::NodeList,
    end_offset: SourceOffset,
    single_line: bool,
) -> bool {
    let mut emitted_args = false;
    let args_count = node_list.iter().count();
    let cls: RenderFunc = Box::new(|ps| {
        for (idx, expr) in node_list.iter().enumerate() {
            if single_line {
                format_node(ps, expr);
                if idx != args_count - 1 {
                    ps.emit_comma_space();
                }
            } else {
                ps.with_start_of_line(
                    false,
                    Box::new(|ps| {
                        if expr.as_assoc_node().is_none() {
                            ps.emit_soft_indent();
                        }
                        format_node(ps, expr);

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

    ps.magic_handle_comments_for_multiline_arrays(
        Some(ps.get_line_number_for_offset(end_offset)),
        cls,
    );
    emitted_args
}
