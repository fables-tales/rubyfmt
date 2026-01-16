# Test cases for parentheses that MUST be preserved to maintain correct semantics
# These contain Ruby keywords that would cause syntax errors if parens were removed

# Modifier if - `foo (a if b)` must become `foo((a if b))` not `foo(a if b)` (syntax error)
foo((a if b))
method_call((value if condition))

# Modifier unless
foo((a unless b))
method_call((value unless condition))

# Modifier while
foo((a while b))

# Modifier until
foo((a until b))

# Inline rescue - `foo (x rescue y)` must become `foo((x rescue y))`
foo((risky_call rescue fallback))
method_call((might_fail rescue nil))

# `and` keyword (low precedence) - `foo (a and b)` must become `foo((a and b))`
foo((a and b))
check((condition and action))

# `or` keyword (low precedence) - `foo (a or b)` must become `foo((a or b))`
foo((a or b))
provide((default or fallback))

# case expressions
foo(
  (case x
  when 1
    :one
  when 2
    :two
  else
    :other
  end)
)

# begin/end blocks
foo(
  (begin
    risky_operation
  rescue
    fallback
  end)
)
