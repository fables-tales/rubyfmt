# Test cases for parentheses that CAN be safely unwrapped when used as method arguments
# These are simple expressions where `method (expr)` becomes `method(expr)`

# Simple literals
foo (1)
foo ("hello")
foo (:symbol)
foo (nil)
foo (true)

# Simple variables
foo (bar)
foo (BAR)
foo (@bar)

# Simple method calls
foo (bar.baz)

# Array and hash literals
foo ([1, 2, 3])
foo ({a: 1})

# Arithmetic expressions (these bind tighter than method args)
foo (a + b)
foo (a * b)

# Comparison expressions
foo (a == b)
foo (a < b)

# Logical operators (high precedence &&, ||)
foo (a && b)
foo (a || b)

# Range expressions
foo (1..10)

# Ternary without keywords
foo (condition ? a : b)

# Assignments we leave alone, for consistency with code like:
#
# if (var = thing)
foo ((var = thing))
foo(1, (var2 = thing))
