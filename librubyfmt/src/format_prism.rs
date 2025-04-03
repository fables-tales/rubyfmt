use ruby_prism as prism;

use crate::{
    delimiters::BreakableDelims,
    format::SpecialCase,
    parser_state::{ConcreteParserState, FormattingContext, HashType, RenderFunc},
    render_targets::MultilineHandling,
    types::SourceOffset,
    util::{const_to_string, loc_to_string},
};

pub fn format_node(ps: &mut dyn ConcreteParserState, node: prism::Node) {
    use prism::Node;
    match node {
        Node::AliasGlobalVariableNode { .. } => todo!(),
        Node::AliasMethodNode { .. } => todo!(),
        Node::AlternationPatternNode { .. } => todo!(),
        Node::AndNode { .. } => todo!(),
        Node::ArgumentsNode { .. } => format_arguments_node(ps, node.as_arguments_node().unwrap()),
        Node::ArrayNode { .. } => format_array_node(ps, node.as_array_node().unwrap()),
        Node::ArrayPatternNode { .. } => todo!(),
        Node::AssocNode { .. } => format_assoc_node(ps, node.as_assoc_node().unwrap()),
        Node::AssocSplatNode { .. } => todo!(),
        Node::BackReferenceReadNode { .. } => todo!(),
        Node::BeginNode { .. } => todo!(),
        Node::BlockArgumentNode { .. } => todo!(),
        Node::BlockLocalVariableNode { .. } => todo!(),
        Node::BlockNode { .. } => todo!(),
        Node::BlockParameterNode { .. } => todo!(),
        Node::BlockParametersNode { .. } => todo!(),
        Node::BreakNode { .. } => todo!(),
        Node::CallAndWriteNode { .. } => todo!(),
        Node::CallNode { .. } => format_call_node(ps, node.as_call_node().unwrap(), false),
        Node::CallOperatorWriteNode { .. } => todo!(),
        Node::CallOrWriteNode { .. } => todo!(),
        Node::CallTargetNode { .. } => todo!(),
        Node::CapturePatternNode { .. } => todo!(),
        Node::CaseMatchNode { .. } => todo!(),
        Node::CaseNode { .. } => todo!(),
        Node::ClassNode { .. } => format_class_node(ps, node.as_class_node().unwrap()),
        Node::ClassVariableAndWriteNode { .. } => todo!(),
        Node::ClassVariableOperatorWriteNode { .. } => todo!(),
        Node::ClassVariableOrWriteNode { .. } => todo!(),
        Node::ClassVariableReadNode { .. } => todo!(),
        Node::ClassVariableTargetNode { .. } => todo!(),
        Node::ClassVariableWriteNode { .. } => todo!(),
        Node::ConstantAndWriteNode { .. } => todo!(),
        Node::ConstantOperatorWriteNode { .. } => todo!(),
        Node::ConstantOrWriteNode { .. } => todo!(),
        Node::ConstantPathAndWriteNode { .. } => todo!(),
        Node::ConstantPathNode { .. } => {
            format_constant_path_node(ps, node.as_constant_path_node().unwrap())
        }
        Node::ConstantPathOperatorWriteNode { .. } => todo!(),
        Node::ConstantPathOrWriteNode { .. } => todo!(),
        Node::ConstantPathTargetNode { .. } => todo!(),
        Node::ConstantPathWriteNode { .. } => todo!(),
        Node::ConstantReadNode { .. } => {
            format_constant_read_node(ps, node.as_constant_read_node().unwrap())
        }
        Node::ConstantTargetNode { .. } => todo!(),
        Node::ConstantWriteNode { .. } => todo!(),
        Node::DefNode { .. } => format_def_node(ps, node.as_def_node().unwrap()),
        Node::DefinedNode { .. } => todo!(),
        Node::ElseNode { .. } => todo!(),
        Node::EmbeddedStatementsNode { .. } => todo!(),
        Node::EmbeddedVariableNode { .. } => todo!(),
        Node::EnsureNode { .. } => todo!(),
        Node::FalseNode { .. } => todo!(),
        Node::FindPatternNode { .. } => todo!(),
        Node::FlipFlopNode { .. } => todo!(),
        Node::FloatNode { .. } => format_float_node(ps, node.as_float_node().unwrap()),
        Node::ForNode { .. } => todo!(),
        Node::ForwardingArgumentsNode { .. } => todo!(),
        Node::ForwardingParameterNode { .. } => todo!(),
        Node::ForwardingSuperNode { .. } => todo!(),
        Node::GlobalVariableAndWriteNode { .. } => todo!(),
        Node::GlobalVariableOperatorWriteNode { .. } => todo!(),
        Node::GlobalVariableOrWriteNode { .. } => todo!(),
        Node::GlobalVariableReadNode { .. } => todo!(),
        Node::GlobalVariableTargetNode { .. } => todo!(),
        Node::GlobalVariableWriteNode { .. } => todo!(),
        Node::HashNode { .. } => todo!(),
        Node::HashPatternNode { .. } => todo!(),
        Node::IfNode { .. } => todo!(),
        Node::ImaginaryNode { .. } => todo!(),
        Node::ImplicitNode { .. } => todo!(),
        Node::ImplicitRestNode { .. } => todo!(),
        Node::InNode { .. } => todo!(),
        Node::IndexAndWriteNode { .. } => todo!(),
        Node::IndexOperatorWriteNode { .. } => todo!(),
        Node::IndexOrWriteNode { .. } => todo!(),
        Node::IndexTargetNode { .. } => todo!(),
        Node::InstanceVariableAndWriteNode { .. } => todo!(),
        Node::InstanceVariableOperatorWriteNode { .. } => todo!(),
        Node::InstanceVariableOrWriteNode { .. } => todo!(),
        Node::InstanceVariableReadNode { .. } => todo!(),
        Node::InstanceVariableTargetNode { .. } => todo!(),
        Node::InstanceVariableWriteNode { .. } => todo!(),
        Node::IntegerNode { .. } => format_integer_node(ps, node.as_integer_node().unwrap()),
        Node::InterpolatedMatchLastLineNode { .. } => todo!(),
        Node::InterpolatedRegularExpressionNode { .. } => todo!(),
        Node::InterpolatedStringNode { .. } => todo!(),
        Node::InterpolatedSymbolNode { .. } => todo!(),
        Node::InterpolatedXStringNode { .. } => todo!(),
        Node::ItLocalVariableReadNode { .. } => todo!(),
        Node::ItParametersNode { .. } => todo!(),
        Node::KeywordHashNode { .. } => {
            format_keyword_hash_node(ps, node.as_keyword_hash_node().unwrap())
        }
        Node::KeywordRestParameterNode { .. } => {
            format_keyword_rest_parameter_node(ps, node.as_keyword_rest_parameter_node().unwrap())
        }
        Node::LambdaNode { .. } => todo!(),
        Node::LocalVariableAndWriteNode { .. } => todo!(),
        Node::LocalVariableOperatorWriteNode { .. } => todo!(),
        Node::LocalVariableOrWriteNode { .. } => todo!(),
        Node::LocalVariableReadNode { .. } => {
            format_local_variable_read_node(ps, node.as_local_variable_read_node().unwrap())
        }
        Node::LocalVariableTargetNode { .. } => todo!(),
        Node::LocalVariableWriteNode { .. } => {
            format_local_variable_write_node(ps, node.as_local_variable_write_node().unwrap())
        }
        Node::MatchLastLineNode { .. } => todo!(),
        Node::MatchPredicateNode { .. } => todo!(),
        Node::MatchRequiredNode { .. } => todo!(),
        Node::MatchWriteNode { .. } => todo!(),
        Node::MissingNode { .. } => todo!(),
        Node::ModuleNode { .. } => todo!(),
        Node::MultiTargetNode { .. } => todo!(),
        Node::MultiWriteNode { .. } => todo!(),
        Node::NextNode { .. } => todo!(),
        Node::NilNode { .. } => todo!(),
        Node::NoKeywordsParameterNode { .. } => todo!(),
        Node::NumberedParametersNode { .. } => todo!(),
        Node::NumberedReferenceReadNode { .. } => todo!(),
        Node::OptionalKeywordParameterNode { .. } => todo!(),
        Node::OptionalParameterNode { .. } => todo!(),
        Node::OrNode { .. } => todo!(),
        Node::ParametersNode { .. } => todo!(),
        Node::ParenthesesNode { .. } => {
            format_parentheses_node(ps, node.as_parentheses_node().unwrap())
        }
        Node::PinnedExpressionNode { .. } => todo!(),
        Node::PinnedVariableNode { .. } => todo!(),
        Node::PostExecutionNode { .. } => todo!(),
        Node::PreExecutionNode { .. } => todo!(),
        Node::ProgramNode { .. } => format_program(ps, node.as_program_node().unwrap()),
        Node::RangeNode { .. } => todo!(),
        Node::RationalNode { .. } => todo!(),
        Node::RedoNode { .. } => todo!(),
        Node::RegularExpressionNode { .. } => todo!(),
        Node::RequiredKeywordParameterNode { .. } => format_required_keyword_parameter_node(
            ps,
            node.as_required_keyword_parameter_node().unwrap(),
        ),
        Node::RequiredParameterNode { .. } => {
            format_required_parameter_node(ps, node.as_required_parameter_node().unwrap())
        }
        Node::RescueModifierNode { .. } => todo!(),
        Node::RescueNode { .. } => todo!(),
        Node::RestParameterNode { .. } => todo!(),
        Node::RetryNode { .. } => todo!(),
        Node::ReturnNode { .. } => todo!(),
        Node::SelfNode { .. } => format_self_node(ps, node.as_self_node().unwrap()),
        Node::ShareableConstantNode { .. } => todo!(),
        Node::SingletonClassNode { .. } => todo!(),
        Node::SourceEncodingNode { .. } => todo!(),
        Node::SourceFileNode { .. } => todo!(),
        Node::SourceLineNode { .. } => todo!(),
        Node::SplatNode { .. } => format_splat_node(ps, node.as_splat_node().unwrap()),
        Node::StatementsNode { .. } => format_statements(ps, node.as_statements_node().unwrap()),
        Node::StringNode { .. } => todo!(),
        Node::SuperNode { .. } => todo!(),
        Node::SymbolNode { .. } => format_symbol_node(ps, node.as_symbol_node().unwrap()),
        Node::TrueNode { .. } => todo!(),
        Node::UndefNode { .. } => todo!(),
        Node::UnlessNode { .. } => todo!(),
        Node::UntilNode { .. } => todo!(),
        Node::WhenNode { .. } => todo!(),
        Node::WhileNode { .. } => todo!(),
        Node::XStringNode { .. } => todo!(),
        Node::YieldNode { .. } => todo!(),
    }
}

fn format_program(ps: &mut dyn ConcreteParserState, program_node: prism::ProgramNode) {
    ps.with_start_of_line(
        true,
        Box::new(|ps| {
            format_statements(ps, program_node.statements());
        }),
    );
    ps.emit_newline();
    ps.on_line(10000000000);
    ps.shift_comments();
}

fn format_statements(ps: &mut dyn ConcreteParserState, statements_node: prism::StatementsNode) {
    for node in statements_node.body().iter() {
        format_node(ps, node);
    }
}

fn format_class_node(ps: &mut dyn ConcreteParserState, class_node: prism::ClassNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(class_node.location().start_offset());

    ps.emit_class_keyword();
    ps.emit_space();
    ps.emit_ident(const_to_string(class_node.name()));

    if let Some(superclass) = class_node.superclass() {
        ps.at_offset(superclass.location().start_offset());

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
    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_def_node(ps: &mut dyn ConcreteParserState, def_node: prism::DefNode) {
    ps.at_offset(def_node.def_keyword_loc().start_offset());
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

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

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
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

fn format_call_node(
    ps: &mut dyn ConcreteParserState,
    call_node: prism::CallNode,
    skip_receiver: bool,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(call_node.location().start_offset());

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

    if ps.at_start_of_line() {
        ps.emit_newline();
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

fn format_symbol_node(ps: &mut dyn ConcreteParserState, symbol_node: prism::SymbolNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    if let Some(opening_loc) = symbol_node.opening_loc() {
        ps.emit_ident(loc_to_string(opening_loc));
    }
    if let Some(value_loc) = symbol_node.value_loc() {
        ps.emit_ident(loc_to_string(value_loc));
    }

    if ps.at_start_of_line() {
        ps.emit_newline();
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

fn format_array_node(ps: &mut dyn ConcreteParserState, array_node: prism::ArrayNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(array_node.location().start_offset());

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

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_parentheses_node(
    ps: &mut dyn ConcreteParserState,
    parentheses_node: prism::ParenthesesNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(parentheses_node.location().start_offset());

    ps.emit_open_paren();
    if let Some(body) = parentheses_node.body() {
        ps.with_start_of_line(
            false,
            Box::new(|ps| {
                format_node(ps, body);
            }),
        );
        ps.at_offset(parentheses_node.location().end_offset());
    }
    ps.emit_close_paren();

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
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
    format_list_like_thing(
        ps,
        arguments_node.arguments(),
        arguments_node.location().end_offset(),
        false,
    );
}

fn format_keyword_hash_node(
    ps: &mut dyn ConcreteParserState,
    keyword_hash_node: prism::KeywordHashNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

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

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_keyword_rest_parameter_node(
    ps: &mut dyn ConcreteParserState,
    keyword_rest_parameter_node: prism::KeywordRestParameterNode,
) {
    ps.at_offset(keyword_rest_parameter_node.location().start_offset());

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
    ps.at_offset(required_keyword_parameter_node.location().start_offset());

    let name = const_to_string(required_keyword_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
    ps.emit_ident(":".to_string());
}

fn format_required_parameter_node(
    ps: &mut dyn ConcreteParserState,
    required_parameter_node: prism::RequiredParameterNode,
) {
    ps.at_offset(required_parameter_node.location().start_offset());

    let name = const_to_string(required_parameter_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);
}

fn format_local_variable_read_node(
    ps: &mut dyn ConcreteParserState,
    local_variable_read_node: prism::LocalVariableReadNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(local_variable_read_node.location().start_offset());

    let name = const_to_string(local_variable_read_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_local_variable_write_node(
    ps: &mut dyn ConcreteParserState,
    local_variable_write_node: prism::LocalVariableWriteNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(local_variable_write_node.location().start_offset());

    let name = const_to_string(local_variable_write_node.name());
    ps.bind_variable(name.clone());
    ps.emit_ident(name);

    ps.emit_ident("=".to_string());

    ps.with_start_of_line(
        false,
        Box::new(|ps| format_node(ps, local_variable_write_node.value())),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_splat_node(ps: &mut dyn ConcreteParserState, splat_node: prism::SplatNode) {
    ps.at_offset(splat_node.location().start_offset());

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
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(ps, ident, offset);

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_integer_node(ps: &mut dyn ConcreteParserState, integer_node: prism::IntegerNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(
        ps,
        loc_to_string(integer_node.location()),
        integer_node.location().start_offset(),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_float_node(ps: &mut dyn ConcreteParserState, float_node: prism::FloatNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(
        ps,
        loc_to_string(float_node.location()),
        float_node.location().start_offset(),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_constant_read_node(
    ps: &mut dyn ConcreteParserState,
    constant_read_node: prism::ConstantReadNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    handle_string_at_offset(
        ps,
        const_to_string(constant_read_node.name()),
        constant_read_node.location().start_offset(),
    );

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_constant_path_node(
    ps: &mut dyn ConcreteParserState,
    constant_path_node: prism::ConstantPathNode,
) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

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

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
}

fn format_self_node(ps: &mut dyn ConcreteParserState, self_node: prism::SelfNode) {
    if ps.at_start_of_line() {
        ps.emit_indent();
    }

    ps.at_offset(self_node.location().start_offset());
    ps.emit_ident("self".to_string());

    if ps.at_start_of_line() {
        ps.emit_newline();
    }
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
