use libtest_mimic::{Arguments, Trial};
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use assert_cmd::Command;

// TODO: These tests fail in debug but work in release builds
// due to some newline shenanigans, we should investigate why
#[cfg(debug_assertions)]
const IGNORED_IN_DEBUG: &[&'static str] = &[
    "test_prism_large_concurrent-ruby_non_concurrent_map_backend",
    "test_prism_large_concurrent_ruby_future",
    "test_prism_large_rspec_mocks_proxy",
    "test_ripper_large_concurrent-ruby_non_concurrent_map_backend",
    "test_ripper_large_concurrent_ruby_future",
    "test_ripper_large_rspec_mocks_proxy",
];

#[cfg(not(debug_assertions))]
const IGNORED_IN_DEBUG: &[&'static str] = &[];

// These tests are disabled due to ripper issues
const DISABLED_RIPPER_TESTS: &[&'static str] = &[
    // TODO: The Ripper implementation does not currently support args forwarding with additional
    // arguments passed in.
    // https://github.com/fables-tales/rubyfmt/issues/474
    "small_args_forwarding_additional_args",
];

// These tests will have their prism variants automatically ignored
const DISABLED_PRISM_TESTS: &[&'static str] = &[
    "large_concurrent-ruby_atom",
    "large_concurrent-ruby_copy_on_notify_observer_set",
    "large_concurrent-ruby_ivar",
    "large_concurrent-ruby_java_non_concurrent_priority_queue",
    "large_concurrent-ruby_mutex_atomic",
    "large_concurrent-ruby_non_concurrent_map_backend",
    "large_concurrent-ruby_non_concurrent_priority_queue",
    "large_concurrent-ruby_ruby_non_concurrent_priority_queue",
    "large_concurrent-ruby_synchronized_map_backend",
    "large_concurrent-ruby_truffleruby_map_backend",
    "large_concurrent_ruby_future",
    "large_dqt",
    "large_rspec_core_notifications",
    "large_rspec_mocks_proxy",
    "small_2.5_lambda_do_end",
    "small_aref_in_call",
    "small_aref_rest_param",
    "small_bare_rescue_comments",
    "small_binary_operators",
    "small_bitwise_not",
    "small_block_param_line_length",
    "small_brace_blocks_with_no_args",
    "small_break_all_the_things",
    "small_breakables_over_line_length",
    "small_character_literal",
    "small_class_methods_respect_parens",
    "small_class_module",
    "small_command_call_raise",
    "small_comments_at_indentation_changes",
    "small_comments_with_breaks",
    "small_const_call",
    "small_dotcall",
    "small_dyna_symbol_with_escapes",
    "small_empty_arg_paren",
    "small_for_loop",
    "small_gemfile",
    "small_hash_rocket_usage",
    "small_heredocs",
    "small_heredocs_ending_blocks",
    "small_inline_rescue",
    "small_lambda",
    "small_long_raise",
    "small_many_opassigns",
    "small_massign",
    "small_massign_omg",
    "small_method_annotation",
    "small_method_chains",
    "small_mlhs_params",
    "small_mlhs_paren",
    "small_more_methods",
    "small_mrhs_add_star",
    "small_mrhs_add_star_2",
    "small_mrhs_new_from_args",
    "small_multi_assign_method",
    "small_multi_line_word_arrays",
    "small_multi_rescue",
    "small_multiline_chain_in_block",
    "small_multiline_method_chain_with_arguments",
    "small_nested_destructuring",
    "small_not",
    "small_opassign",
    "small_paren_expr_calls",
    "small_pathological_heredocs",
    "small_percent_q",
    "small_require_rails",
    "small_require_relative",
    "small_rest_param_unpacking",
    "small_return",
    "small_rspec_its",
    "small_sclass",
    "small_scoping",
    "small_single_massign",
    "small_stabby_lambda",
    "small_string_dvar",
    "small_string_first_item_is_embed",
    "small_string_literal_with_class_interp",
    "small_string_literals_dont_break_comments",
    "small_string_with_embexpr_dyna_symbol",
    "small_unary",
    "small_variable_binding",
    "small_visibility_modifier",
    "small_w_array",
    "small_w_arrays",
    "small_yield_hash_args",
];

fn main() -> io::Result<()> {
    let args = Arguments::from_args();

    let mut tests = Vec::new();

    let fixtures = collect_fixtures("fixtures".into())?;

    tests.extend(fixtures.iter().map(|f| f.to_trial(Flavor::Ripper)));
    tests.extend(fixtures.iter().map(|f| f.to_trial(Flavor::Prism)));

    libtest_mimic::run(&args, tests).exit();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flavor {
    Ripper,
    Prism,
}

impl Flavor {
    fn to_str(&self) -> &'static str {
        match self {
            Flavor::Ripper => "ripper",
            Flavor::Prism => "prism",
        }
    }

    fn disabled_list(&self) -> &[&'static str] {
        match self {
            Flavor::Ripper => DISABLED_RIPPER_TESTS,
            Flavor::Prism => DISABLED_PRISM_TESTS,
        }
    }
}

#[derive(Debug)]
struct Fixture {
    name: String,
    actual: Option<PathBuf>,
    expected: Option<PathBuf>,
}

impl Fixture {
    fn to_trial(&self, flavor: Flavor) -> Trial {
        let name = format!("test_{}_{}", flavor.to_str(), self.name);
        let is_ignored = IGNORED_IN_DEBUG.contains(&name.as_str())
            || flavor.disabled_list().contains(&self.name.as_str());
        let actual = self.actual.clone();
        let expected = self.expected.clone();
        Trial::test(name.clone(), move || {
            let Some(actual) = actual else {
                return Result::Err(format!("Test {} is missing an _actual.rb file", name).into());
            };

            let Some(expected) = expected else {
                return Result::Err(
                    format!("Test {} is missing an _expected.rb file", name).into(),
                );
            };

            let expected_text = read_to_string(&expected)?;

            // Test if the formatting works as expected
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(actual.to_str().unwrap());

            if flavor == Flavor::Prism {
                cmd.arg("--prism");
            }

            cmd.assert().success().stdout(expected_text.clone());

            // Test if the formatting is idempotent
            let mut cmd = Command::cargo_bin("rubyfmt-main").unwrap();
            cmd.arg(expected.to_str().unwrap());

            if flavor == Flavor::Prism {
                cmd.arg("--prism");
            }

            cmd.assert().success().stdout(expected_text);
            Ok(())
        })
        .with_ignored_flag(is_ignored)
    }
}

fn collect_fixtures(path: PathBuf) -> io::Result<Vec<Fixture>> {
    #[derive(Default)]
    struct Partial {
        actual: Option<PathBuf>,
        expected: Option<PathBuf>,
    }

    fn recurse(results: &mut Vec<Fixture>, path: &Path) -> io::Result<()> {
        let mut partial: HashMap<String, Partial> = HashMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type()?.is_dir() {
                recurse(results, &path)?;
                continue;
            }

            let file = path.to_str().unwrap();
            if file.ends_with("_actual.rb") {
                let len = file.len() - "_actual.rb".len();
                let prefix = file[0..len].to_owned();
                let p = partial.entry(prefix).or_default();
                assert!(p.actual.is_none());
                p.actual = Some(path);
            } else if file.ends_with("_expected.rb") {
                let len = file.len() - "_expected.rb".len();
                let prefix = file[0..len].to_owned();
                let p = partial.entry(prefix).or_default();
                assert!(p.expected.is_none());
                p.expected = Some(path);
            }
        }

        for (mut k, v) in partial {
            k.drain(0.."fixtures/".len());
            results.push(Fixture {
                name: k.replace('/', "_"),
                actual: v.actual,
                expected: v.expected,
            });
        }

        Ok(())
    }

    let mut results = Vec::new();
    recurse(&mut results, &path)?;
    Ok(results)
}
