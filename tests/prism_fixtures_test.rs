use std::fs::read_to_string;

use assert_cmd::Command;

macro_rules! fixture {
    ($test_name:ident, $path:expr) => {
        #[test]
        fn $test_name() {
            test_fixture($path)
        }
    };
}

fn test_fixture(name: &str) {
    let expected = read_to_string(format!("fixtures/{}_expected.rb", name)).unwrap();

    // Test if the formatting works as expected
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(format!("fixtures/{}_actual.rb", name))
        .arg("--prism")
        .assert()
        .success()
        .stdout(expected.clone());

    // Test if the formatting is idempotent
    Command::cargo_bin("rubyfmt-main")
        .unwrap()
        .arg(format!("fixtures/{}_expected.rb", name))
        .arg("--prism")
        .assert()
        .success()
        .stdout(expected);
}

fixture!(test_small_two_five, "small/2.5/2.5");
// fixture!(test_small_two_five_lambda_do_end, "small/2.5/lambda_do_end");
fixture!(
    test_small_two_five_unnamed_kwrest_param_def,
    "small/2.5/unnamed_kwrest_param_def"
);
fixture!(
    test_small_two_six_range_with_no_upper_bound,
    "small/2.6/range_with_no_upper_bound"
);

// fixture!(test_small_alias, "small/alias");
// fixture!(test_small_alias_bare_kw, "small/alias_bare_kw");
// fixture!(test_small_alias_equal, "small/alias_equal");
// fixture!(test_small_alias_string_symbol, "small/alias_string_symbol");
fixture!(test_small_all_method_blocks, "small/all_method_blocks");
// fixture!(test_small_and_or, "small/and_or");
// fixture!(test_small_anonymous_blockarg, "small/anonymous_blockarg");
// fixture!(test_small_aref_field, "small/aref_field");
// fixture!(test_small_aref_in_call, "small/aref_in_call");
// fixture!(test_small_aref_rest_param, "small/aref_rest_param");
// fixture!(
//     test_small_arg_list_with_bare_assoc_hash,
//     "small/arg_list_with_bare_assoc_hash"
// );
fixture!(test_small_arg_trailing_comma, "small/arg_trailing_comma");
fixture!(test_small_args_forwarding, "small/args_forwarding");
fixture!(
    test_small_args_forwarding_additional_args,
    "small/args_forwarding_additional_args"
);
// fixture!(
//     test_small_args_forwarding_multiline,
//     "small/args_forwarding_multiline"
// );
// fixture!(test_small_array_with_splat, "small/array_with_splat");
// fixture!(test_small_assign_field, "small/assign_field");
fixture!(test_small_assoc_double_splat, "small/assoc_double_splat");
// fixture!(test_small_backref, "small/backref");
fixture!(test_small_backtick_symbol, "small/backtick_symbol");
// fixture!(test_small_backticks, "small/backticks");
// fixture!(test_small_bare_alias, "small/bare_alias");
fixture!(
    test_small_bare_assoc_hash_trailing_comma,
    "small/bare_assoc_hash_trailing_comma"
);
fixture!(test_small_bare_assoc_multiple, "small/bare_assoc_multiple");
// fixture!(
//     test_small_bare_rescue_comments,
//     "small/bare_rescue_comments"
// );
// fixture!(test_small_bare_rescue_newline, "small/bare_rescue_newline");
fixture!(test_small_bare_return_comment, "small/bare_return_comment");
// fixture!(test_small_begin_block, "small/begin_block");
fixture!(test_small_begin_block_comment, "small/begin_block_comment");
fixture!(test_small_begin_end_stack, "small/begin_end_stack");
fixture!(test_small_begin_ensure_rescue, "small/begin_ensure_rescue");
fixture!(test_small_begin_with_comment, "small/begin_with_comment");
// fixture!(
//     test_small_binary_line_splitting,
//     "small/binary_line_splitting"
// );
// fixture!(test_small_binary_operators, "small/binary_operators");
// fixture!(test_small_binary_raise, "small/binary_raise");
// fixture!(test_small_bitwise_not, "small/bitwise_not");
// fixture!(test_small_block_argument, "small/block_argument");
fixture!(
    test_small_block_ending_with_comment,
    "small/block_ending_with_comment"
);
// fixture!(
//     test_small_block_local_variables,
//     "small/block_local_variables"
// );
// fixture!(
//     test_small_block_param_line_length,
//     "small/block_param_line_length"
// );
// fixture!(test_small_block_spacing, "small/block_spacing");
fixture!(test_small_blockarg, "small/blockarg");
fixture!(
    test_small_blocks_with_only_comments,
    "small/blocks_with_only_comments"
);
// fixture!(
//     test_small_brace_blocks_with_no_args,
//     "small/brace_blocks_with_no_args"
// );
// fixture!(
//     test_small_bracket_method_call_on_module_with_method_call_inside,
//     "small/bracket_method_call_on_module_with_method_call_inside"
// );
// fixture!(
//     test_small_bracket_method_call_on_module_with_no_args,
//     "small/bracket_method_call_on_module_with_no_args"
// );
// fixture!(
//     test_small_bracket_method_call_on_module_with_trailing_comma,
//     "small/bracket_method_call_on_module_with_trailing_comma"
// );
// fixture!(
//     test_small_break_all_the_things,
//     "small/break_all_the_things"
// );
// fixture!(test_small_breakable_binary_op, "small/breakable_binary_op");
// fixture!(
//     test_small_breakables_over_line_length,
//     "small/breakables_over_line_length"
// );
// fixture!(test_small_breaks, "small/breaks");
// fixture!(test_small_cannibalization_1, "small/cannibalization_1");
fixture!(test_small_cannibalization_2, "small/cannibalization_2");
fixture!(test_small_cannibalization_3, "small/cannibalization_3");
fixture!(test_small_cannibalization_4, "small/cannibalization_4");
// fixture!(test_small_case, "small/case");
// fixture!(test_small_case_else, "small/case_else");
// fixture!(test_small_case_multi, "small/case_multi");
// fixture!(test_small_character_literal, "small/character_literal");
fixture!(test_small_class_comment, "small/class_comment");
// fixture!(
//     test_small_class_methods_respect_parens,
//     "small/class_methods_respect_parens"
// );
// fixture!(test_small_class_module, "small/class_module");
fixture!(test_small_coloncoloncalldot, "small/coloncoloncalldot");
// fixture!(test_small_command_call_raise, "small/command_call_raise");
// fixture!(test_small_command_paren, "small/command_paren");
fixture!(test_small_commas_trailing, "small/commas_trailing");
// fixture!(
//     test_small_comment_in_block_inside_array,
//     "small/comment_in_block_inside_array"
// );
fixture!(
    test_small_comment_in_module_are_tight,
    "small/comment_in_module_are_tight"
);
fixture!(
    test_small_comment_on_inheritence,
    "small/comment_on_inheritence"
);
// fixture!(
//     test_small_comments_at_indentation_changes,
//     "small/comments_at_indentation_changes"
// );
fixture!(
    test_small_comments_inside_array,
    "small/comments_inside_array"
);
// fixture!(
//     test_small_comments_with_breaks,
//     "small/comments_with_breaks"
// );
// fixture!(test_small_complex_numbers, "small/complex_numbers");
fixture!(test_small_complex_params, "small/complex_params");
// fixture!(test_small_conditional, "small/conditional");
// fixture!(test_small_conditional_assign, "small/conditional_assign");
// fixture!(
//     test_small_conditional_expanded_chain,
//     "small/conditional_expanded_chain"
// );
// fixture!(
//     test_small_conditionals_with_ending_comments,
//     "small/conditionals_with_ending_comments"
// );
// fixture!(test_small_const_call, "small/const_call");
fixture!(
    test_small_const_path_field_assignment,
    "small/const_path_field_assignment"
);
fixture!(
    test_small_const_path_ref_class_definition,
    "small/const_path_ref_class_definition"
);
fixture!(test_small_curly_line_breaking, "small/curly_line_breaking");
// fixture!(test_small_cursed_call_01, "small/cursed_call_01");
// fixture!(test_small_cvar, "small/cvar");
fixture!(test_small_cvar_symbol, "small/cvar_symbol");
fixture!(test_small_def_const, "small/def_const");
fixture!(test_small_def_scope, "small/def_scope");
fixture!(test_small_def_with_kw, "small/def_with_kw");
// fixture!(test_small_defined, "small/defined");
fixture!(
    test_small_definitions_with_comments,
    "small/definitions_with_comments"
);
fixture!(test_small_defs_comment, "small/defs_comment");
fixture!(test_small_defs_end_comment, "small/defs_end_comment");
fixture!(test_small_defs_kw, "small/defs_kw");
fixture!(test_small_disambiguated_range, "small/disambiguated_range");
fixture!(test_small_dot2, "small/dot2");
fixture!(test_small_dot3, "small/dot3");
// fixture!(test_small_dotcall, "small/dotcall");
fixture!(test_small_double_splat, "small/double_splat");
fixture!(test_small_double_splat_def, "small/double_splat_def");
// fixture!(
//     test_small_dyna_symbol_with_escapes,
//     "small/dyna_symbol_with_escapes"
// );
// fixture!(test_small_empty_arg_paren, "small/empty_arg_paren");
fixture!(test_small_empty_array, "small/empty_array");
fixture!(test_small_empty_block, "small/empty_block");
fixture!(test_small_empty_comments, "small/empty_comments");
// fixture!(test_small_empty_heredocs, "small/empty_heredocs");
fixture!(
    test_small_empty_module_comments,
    "small/empty_module_comments"
);
fixture!(
    test_small_empty_string_literal,
    "small/empty_string_literal"
);
// fixture!(test_small_end_block, "small/end_block");
fixture!(test_small_end_data, "small/end_data");
fixture!(
    test_small_end_of_file_comments,
    "small/end_of_file_comments"
);
// fixture!(test_small_endless_methods, "small/endless_methods");
// fixture!(test_small_fib, "small/fib");
// fixture!(test_small_first_rest_param, "small/first_rest_param");
// fixture!(test_small_for_loop, "small/for_loop");
// fixture!(test_small_gemfile, "small/gemfile");
fixture!(test_small_gvar_symbol, "small/gvar_symbol");
// fixture!(test_small_hash_breaking, "small/hash_breaking");
// fixture!(test_small_hash_comments, "small/hash_comments");
// fixture!(test_small_hash_heredoc, "small/hash_heredoc");
// fixture!(test_small_hash_rocket_usage, "small/hash_rocket_usage");
// fixture!(
//     test_small_heredoc_comment_header,
//     "small/heredoc_comment_header"
// );
// fixture!(
//     test_small_heredoc_ending_with_curly,
//     "small/heredoc_ending_with_curly"
// );
// fixture!(
//     test_small_heredoc_in_array_in_call,
//     "small/heredoc_in_array_in_call"
// );
// fixture!(
//     test_small_heredoc_indented_whitespace,
//     "small/heredoc_indented_whitespace"
// );
// fixture!(test_small_heredoc_method_call, "small/heredoc_method_call");
fixture!(test_small_heredoc_with_call, "small/heredoc_with_call");
fixture!(
    test_small_heredoc_with_leading_newline,
    "small/heredoc_with_leading_newline"
);
// fixture!(test_small_heredocs, "small/heredocs");
// fixture!(
//     test_small_heredocs_ending_blocks,
//     "small/heredocs_ending_blocks"
// );
// fixture!(test_small_heredocs_in_calls, "small/heredocs_in_calls");
// fixture!(
//     test_small_heredocs_with_comments,
//     "small/heredocs_with_comments"
// );
fixture!(
    test_small_if_without_extra_parts,
    "small/if_without_extra_parts"
);
fixture!(test_small_inline_comments, "small/inline_comments");
// fixture!(test_small_inline_rescue, "small/inline_rescue");
fixture!(test_small_ivar_symbol, "small/ivar_symbol");
fixture!(test_small_kw_symbol, "small/kw_symbol");
fixture!(test_small_kwargs, "small/kwargs");
fixture!(test_small_kwargs_multiline, "small/kwargs_multiline");
// fixture!(test_small_lambda, "small/lambda");
fixture!(
    test_small_line_broken_top_const_field,
    "small/line_broken_top_const_field"
);
// fixture!(
//     test_small_list_like_things_with_comments,
//     "small/list_like_things_with_comments"
// );
fixture!(test_small_literals_newlines, "small/literals_newlines");
fixture!(test_small_long_blockvar, "small/long_blockvar");
// fixture!(
//     test_small_long_line_with_indentation,
//     "small/long_line_with_indentation"
// );
// fixture!(test_small_long_raise, "small/long_raise");
// fixture!(test_small_many_arg_types, "small/many_arg_types");
// fixture!(test_small_many_opassigns, "small/many_opassigns");
// fixture!(test_small_many_splats, "small/many_splats");
// fixture!(test_small_many_weird_args, "small/many_weird_args");
fixture!(test_small_map_curly, "small/map_curly");
// fixture!(test_small_massign, "small/massign");
// fixture!(test_small_massign_omg, "small/massign_omg");
fixture!(test_small_memoist, "small/memoist");
fixture!(
    test_small_method_add_block_command_call,
    "small/method_add_block_command_call"
);
// fixture!(test_small_method_annotation, "small/method_annotation");
fixture!(
    test_small_method_call_with_trailing_comma,
    "small/method_call_with_trailing_comma"
);
// fixture!(test_small_method_chains, "small/method_chains");
fixture!(test_small_missing_parens, "small/missing_parens");
// fixture!(test_small_mlhs_params, "small/mlhs_params");
// fixture!(test_small_mlhs_paren, "small/mlhs_paren");
fixture!(test_small_more_breaks, "small/more_breaks");
// fixture!(test_small_more_methods, "small/more_methods");
// fixture!(test_small_mrhs_add_star_2, "small/mrhs_add_star_2");
// fixture!(test_small_mrhs_add_star, "small/mrhs_add_star");
// fixture!(test_small_mrhs_new_from_args, "small/mrhs_new_from_args");
// fixture!(test_small_multi_assign_method, "small/multi_assign_method");
fixture!(
    test_small_multi_expression_embexpr,
    "small/multi_expression_embexpr"
);
// fixture!(
//     test_small_multi_line_word_arrays,
//     "small/multi_line_word_arrays"
// );
// fixture!(test_small_multi_rescue, "small/multi_rescue");
fixture!(test_small_multi_return, "small/multi_return");
// fixture!(
//     test_small_multiline_chain_in_block,
//     "small/multiline_chain_in_block"
// );
fixture!(
    test_small_multiline_chained_call,
    "small/multiline_chained_call"
);
fixture!(test_small_multiline_defs, "small/multiline_defs");
fixture!(
    test_small_multiline_method_call_with_block,
    "small/multiline_method_call_with_block"
);
// fixture!(
//     test_small_multiline_method_chain_with_arguments,
//     "small/multiline_method_chain_with_arguments"
// );
fixture!(
    test_small_multiline_method_params,
    "small/multiline_method_params"
);
fixture!(
    test_small_multiline_single_quotes,
    "small/multiline_single_quotes"
);
fixture!(
    test_small_multiline_strings_in_parameter_list,
    "small/multiline_strings_in_parameter_list"
);
// fixture!(
//     test_small_multiline_when_with_comment,
//     "small/multiline_when_with_comment"
// );
fixture!(test_small_nbsp, "small/nbsp");
fixture!(test_small_nested_command, "small/nested_command");
fixture!(test_small_nested_conditionals, "small/nested_conditionals");
// fixture!(
//     test_small_nested_destructuring,
//     "small/nested_destructuring"
// );
// fixture!(test_small_next_with_args, "small/next_with_args");
// fixture!(test_small_next_with_comments, "small/next_with_comments");
// fixture!(test_small_next_yield, "small/next_yield");
fixture!(test_small_no_mangle_jpy, "small/no_mangle_jpy");
// fixture!(
//     test_small_no_multiline_call_in_string_embexpr,
//     "small/no_multiline_call_in_string_embexpr"
// );
// fixture!(test_small_not, "small/not");
fixture!(test_small_numbers, "small/numbers");
fixture!(test_small_op_defs, "small/op_defs");
// fixture!(test_small_opassign, "small/opassign");
fixture!(
    test_small_optional_arg_in_middle,
    "small/optional_arg_in_middle"
);
// fixture!(test_small_paren_expr_calls, "small/paren_expr_calls");
fixture!(
    test_small_paren_with_multiple_expressions,
    "small/paren_with_multiple_expressions"
);
// fixture!(
//     test_small_pathological_heredocs,
//     "small/pathological_heredocs"
// );
// fixture!(test_small_percent_q, "small/percent_q");
fixture!(
    test_small_preserve_slash_u_strings,
    "small/preserve_slash_u_strings"
);
fixture!(
    test_small_private_class_method,
    "small/private_class_method"
);
fixture!(
    test_small_private_gets_trailing_blankline,
    "small/private_gets_trailing_blankline"
);
// fixture!(test_small_procs, "small/procs");
// fixture!(test_small_quoted_heredoc, "small/quoted_heredoc");
fixture!(test_small_raise_star, "small/raise_star");
// fixture!(test_small_rationals, "small/rationals");
// fixture!(test_small_redo, "small/redo");
// fixture!(test_small_regexp_literal, "small/regexp_literal");
fixture!(test_small_req_optional_params, "small/req_optional_params");
// fixture!(test_small_require_rails, "small/require_rails");
// fixture!(test_small_require_relative, "small/require_relative");
fixture!(
    test_small_requirish_followed_by_module,
    "small/requirish_followed_by_module"
);
fixture!(test_small_rescue_else, "small/rescue_else");
fixture!(test_small_rescue_multiple, "small/rescue_multiple");
fixture!(test_small_rescue_no_class, "small/rescue_no_class");
fixture!(test_small_rest_param, "small/rest_param");
// fixture!(
//     test_small_rest_param_unpacking,
//     "small/rest_param_unpacking"
// );
// fixture!(test_small_retry, "small/retry");
// fixture!(test_small_return0, "small/return0");
// fixture!(test_small_return, "small/return");
// fixture!(test_small_rspec_its, "small/rspec_its");
// fixture!(test_small_sclass, "small/sclass");
// fixture!(test_small_scoping, "small/scoping");
fixture!(
    test_small_separated_statements_with_trailing_comment,
    "small/separated_statements_with_trailing_comment"
);
// fixture!(test_small_shorthand_hash, "small/shorthand_hash");
fixture!(
    test_small_single_line_method_call_chain,
    "small/single_line_method_call_chain"
);
// fixture!(test_small_single_massign, "small/single_massign");
fixture!(
    test_small_single_quoted_string_with_embexpr,
    "small/single_quoted_string_with_embexpr"
);
fixture!(test_small_source_keywords, "small/source_keywords");
fixture!(
    test_small_single_quoted_string_with_slash_escape,
    "small/single_quoted_string_with_slash_escape"
);
// fixture!(test_small_splat_case, "small/splat_case");
fixture!(test_small_splat_in_argument, "small/splat_in_argument");
fixture!(test_small_splat_rescue, "small/splat_rescue");
// fixture!(test_small_sqb_no_parens, "small/sqb_no_parens");
// fixture!(
//     test_small_squiggly_heredoc_interpolation,
//     "small/squiggly_heredoc_interpolation"
// );
// fixture!(test_small_stabby_lambda, "small/stabby_lambda");
fixture!(test_small_star, "small/star");
fixture!(
    test_small_start_of_file_comments,
    "small/start_of_file_comments"
);
fixture!(test_small_static_call, "small/static_call");
fixture!(
    test_small_stmt_followed_by_do_block,
    "small/stmt_followed_by_do_block"
);
fixture!(test_small_string_concat, "small/string_concat");
fixture!(
    test_small_string_concat_indent,
    "small/string_concat_indent"
);
// fixture!(test_small_string_dvar, "small/string_dvar");
fixture!(test_small_string_escapes, "small/string_escapes");
// fixture!(
//     test_small_string_first_item_is_embed,
//     "small/string_first_item_is_embed"
// );
// fixture!(
//     test_small_string_literal_with_class_interp,
//     "small/string_literal_with_class_interp"
// );
// fixture!(
//     test_small_string_literals_dont_break_comments,
//     "small/string_literals_dont_break_comments"
// );
fixture!(
    test_small_string_literals_with_escaped_quotes,
    "small/string_literals_with_escaped_quotes"
);
// fixture!(
//     test_small_string_with_embexpr_dyna_symbol,
//     "small/string_with_embexpr_dyna_symbol"
// );
fixture!(test_small_super_comment, "small/super_comment");
fixture!(test_small_super_in_call_chain, "small/super_in_call_chain");
fixture!(
    test_small_super_keyword_comment_ordering,
    "small/super_keyword_comment_ordering"
);
fixture!(
    test_small_super_keyword_comment_ordering_spaces,
    "small/super_keyword_comment_ordering_spaces"
);
fixture!(test_small_super_with_block, "small/super_with_block");
fixture!(
    test_small_super_with_empty_paren,
    "small/super_with_empty_paren"
);
fixture!(
    test_small_super_with_trailing_comma,
    "small/super_with_trailing_comma"
);
fixture!(test_small_symbol_arg, "small/symbol_arg");
fixture!(test_small_symbol_op, "small/symbol_op");
fixture!(test_small_ternary, "small/ternary");
// fixture!(test_small_to_proc_operator, "small/to_proc_operator");
fixture!(
    test_small_top_const_field_assignment,
    "small/top_const_field_assignment"
);
fixture!(test_small_top_const_ref, "small/top_const_ref");
fixture!(test_small_trailing_empty_star, "small/trailing_empty_star");
// fixture!(test_small_unary, "small/unary");
// fixture!(test_small_undef, "small/undef");
// fixture!(test_small_unless, "small/unless");
// fixture!(test_small_unless_mod, "small/unless_mod");
// fixture!(test_small_until, "small/until");
// fixture!(test_small_until_while_mod, "small/until_while_mod");
// fixture!(test_small_variable_binding, "small/variable_binding");
// fixture!(test_small_visibility_modifier, "small/visibility_modifier");
// fixture!(test_small_w_array, "small/w_array");
// fixture!(test_small_w_arrays, "small/w_arrays");
fixture!(
    test_small_weird_percent_strings,
    "small/weird_percent_strings"
);
// fixture!(test_small_when_indent, "small/when_indent");
// fixture!(test_small_while, "small/while");
// fixture!(test_small_while_block_comment, "small/while_block_comment");
// fixture!(test_small_while_mod, "small/while_mod");
// fixture!(test_small_yield0, "small/yield0");
// fixture!(test_small_yield0_in_ifop, "small/yield0_in_ifop");
// fixture!(test_small_yield, "small/yield");
// fixture!(test_small_yield_comment, "small/yield_comment");
// fixture!(test_small_yield_hash_args, "small/yield_hash_args");
// fixture!(test_small_yield_in_ifop, "small/yield_in_ifop");
// fixture!(
//     test_small_yield_keyword_comment_ordering,
//     "small/yield_keyword_comment_ordering"
// );
// fixture!(
//     test_small_yield_keyword_comment_ordering_spaces,
//     "small/yield_keyword_comment_ordering_spaces"
// );
// fixture!(test_small_yield_with_paren, "small/yield_with_paren");
fixture!(test_small_zsuper, "small/zsuper");

// /*
//    Large Fixtures
// */
// // TODO: This test fails in debug but works in release
// // due to some newline shenanigans, we should investigate why
// #[cfg(not(debug_assertions))]
// fixture!(
//     test_large_concurrent_ruby_future,
//     "large/concurrent_ruby_future"
// );

// fixture!(test_large_dqt, "large/dqt");

// fixture!(
//     test_rspec_core_notifications,
//     "large/rspec_core_notifications"
// );

// // TODO: This test fails in debug but works in release
// // due to some newline shenanigans, we should investigate why
// #[cfg(not(debug_assertions))]
// fixture!(test_rspec_mocks_proxy, "large/rspec_mocks_proxy");

// /* concurrent-ruby examples */
// // TODO: This test fails in debug but works in release
// // due to some newline shenanigans, we should investigate why
// #[cfg(not(debug_assertions))]
// fixture!(
//     test_concurrent_ruby_non_concurrent_map_backend,
//     "large/concurrent-ruby/non_concurrent_map_backend"
// );

// fixture!(test_concurrent_ruby_atom, "large/concurrent-ruby/atom");
// fixture!(
//     test_concurrent_ruby_copy_on_notify_observer_set,
//     "large/concurrent-ruby/copy_on_notify_observer_set"
// );
// fixture!(test_concurrent_ruby_ivar, "large/concurrent-ruby/ivar");
// fixture!(
//     test_concurrent_ruby_java_non_concurrent_priority_queue,
//     "large/concurrent-ruby/java_non_concurrent_priority_queue"
// );
// fixture!(
//     test_concurrent_ruby_mutex_atomic,
//     "large/concurrent-ruby/mutex_atomic"
// );
// fixture!(
//     test_concurrent_ruby_non_concurrent_priority_queue,
//     "large/concurrent-ruby/non_concurrent_priority_queue"
// );
// fixture!(
//     test_concurrent_ruby_numeric_cas_wrapper,
//     "large/concurrent-ruby/numeric_cas_wrapper"
// );
// fixture!(
//     test_concurrent_ruby_ruby_non_concurrent_priority_queue,
//     "large/concurrent-ruby/ruby_non_concurrent_priority_queue"
// );
// fixture!(
//     test_concurrent_ruby_synchronized_map_backend,
//     "large/concurrent-ruby/synchronized_map_backend"
// );
// fixture!(
//     test_concurrent_ruby_truffleruby_map_backend,
//     "large/concurrent-ruby/truffleruby_map_backend"
// );
fixture!(
    test_concurrent_ruby_version,
    "large/concurrent-ruby/version"
);
