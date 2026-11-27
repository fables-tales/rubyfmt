use ruby_prism as prism;

/// Whether two sibling statements should be separated by a blank line.
pub(crate) fn blank_line_between(
    prev_ends_with_end: bool,
    prev_is_annotation: bool,
    has_comments_between: impl FnOnce() -> bool,
    next_is_definition: impl FnOnce() -> bool,
) -> bool {
    prev_ends_with_end && !has_comments_between() && !(prev_is_annotation && next_is_definition())
}

pub(crate) fn is_annotation(node: &prism::Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.block()
            .and_then(|block| block.as_block_node())
            .is_some_and(|block| block.opening_loc().as_slice() == b"do")
    })
}

/// Whether a statement begins with a method definition, possibly behind
/// visibility/modifier calls such as `private def` or `memoize public def`.
pub(crate) fn starts_with_def(node: &prism::Node<'_>) -> bool {
    if node.as_def_node().is_some() {
        return true;
    }

    if let Some(call) = node.as_call_node()
        && call.receiver().is_none()
        && let Some(arguments) = call.arguments()
        && let Some(first_argument) = arguments.arguments().first()
    {
        return starts_with_def(&first_argument);
    }

    if let Some(parentheses) = node.as_parentheses_node()
        && let Some(body) = parentheses.body()
        && let Some(statements) = body.as_statements_node()
        && let Some(first_statement) = statements.body().first()
    {
        return starts_with_def(&first_statement);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{blank_line_between, is_annotation, starts_with_def};

    fn with_first_two_statements<R>(
        source: &str,
        f: impl FnOnce(&ruby_prism::Node<'_>, &ruby_prism::Node<'_>) -> R,
    ) -> R {
        let result = ruby_prism::parse(source.as_bytes());
        assert!(
            result.errors().next().is_none(),
            "test source must parse: {source}"
        );
        let program = result.node().as_program_node().unwrap();
        let body = program.statements().body();
        let mut statements = body.iter();
        let first = statements.next().expect("first statement");
        let second = statements.next().expect("second statement");
        f(&first, &second)
    }

    #[test]
    fn separates_block_statements() {
        assert!(blank_line_between(true, false, || false, || false));
        assert!(blank_line_between(true, false, || false, || true));
    }

    #[test]
    fn does_not_separate_other_statements() {
        assert!(!blank_line_between(false, false, || false, || false));
    }

    #[test]
    fn keeps_annotations_attached_to_definitions() {
        assert!(!blank_line_between(true, true, || false, || true));
        assert!(blank_line_between(true, true, || false, || false));
    }

    #[test]
    fn comments_control_their_own_spacing() {
        assert!(!blank_line_between(true, false, || true, || false));
        assert!(!blank_line_between(true, false, || true, || true));
    }

    #[test]
    fn skips_expensive_checks_when_not_following_end() {
        let mut checked = false;
        assert!(!blank_line_between(
            false,
            true,
            || {
                checked = true;
                true
            },
            || {
                checked = true;
                true
            },
        ));
        assert!(!checked);
    }

    #[test]
    fn recognizes_annotations() {
        with_first_two_statements("sig do\nend\nb", |sig, _| {
            assert!(is_annotation(sig));
        });
        with_first_two_statements("sig { }\nb", |brace, _| {
            assert!(!is_annotation(brace));
        });
        with_first_two_statements("-> do\nend\nb", |lambda, _| {
            assert!(!is_annotation(lambda));
        });
    }

    #[test]
    fn recognizes_definitions_behind_modifiers() {
        with_first_two_statements("a\ndef c\nend", |_, def| {
            assert!(starts_with_def(def));
        });
        with_first_two_statements("a\nprivate def c\nend", |_, private_def| {
            assert!(starts_with_def(private_def));
        });
        with_first_two_statements("a\nmemoize public def c\nend", |_, memoized_def| {
            assert!(starts_with_def(memoized_def));
        });
        with_first_two_statements("a\nc", |_, call| {
            assert!(!starts_with_def(call));
        });
    }
}
