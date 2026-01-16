# Test cases for nested redundant parentheses
# Multiple levels of unnecessary parens should be stripped

# Double nested
foo ((1))
bar ((a + b))

# Triple nested
foo (((1)))
bar (((x)))

# Mixed with method chains
foo ((bar.baz))

# Nested but inner contains keyword - should preserve inner parens
foo ((a if b))
bar ((x rescue y))
