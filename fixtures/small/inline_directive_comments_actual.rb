# frozen_string_literal: true

# Basic rubocop:disable - must stay inline
user.update_column(:role, "admin") # rubocop:disable Rails/SkipsModelValidations
user.update_column(:verified, true)

# rubocop:enable - must stay inline
x = eval(input) # rubocop:enable Security/Eval

# rubocop:todo - must stay inline
x = 1 # rubocop:todo Style/FrozenStringLiteralComment

# Multiple cops on one directive
foo(bar) # rubocop:disable Style/Foo, Style/Bar

# Directive with explanation text
SolarPark.where(id: ids) # rubocop:disable Tenant/UnscopedModelAccess -- IDs pre-filtered by org

# standard:disable - must stay inline
def has_role?(name) # standard:disable Naming/PredicateMethod
  roles.include?(name)
end

# standard:enable - must stay inline
def valid? # standard:enable Naming/PredicateMethod
  true
end

# steep:ignore - must stay inline
result = untyped_method # steep:ignore

# :reek: - must stay inline
def complex_method(a, b, c, d, e) # :reek:LongParameterList
  a + b + c + d + e
end

# Regular trailing comment - must still move above
x = 42 # just a note
y = 99 # another comment

# Standalone directive comment on own line - stays on own line
# rubocop:disable all
def dangerous_method
  eval("puts 1")
end
# rubocop:enable all

# Directive inside a block
[1, 2, 3].each do |i|
  puts i # rubocop:disable Rails/Output
end

# Directive on method call with parens added by rubyfmt
render html: content.html_safe # rubocop:disable Rails/OutputSafety

# Directive inside conditional
if condition
  do_something # rubocop:disable CustomCop/Whatever
else
  do_other # rubocop:disable CustomCop/Other
end

# Indented directive in a class
class MyClass
  def foo
    bar # rubocop:disable Style/Foo
  end
end

# Multiple lines with directives and regular comments
a = 1 # regular comment
b = 2 # rubocop:disable Style/NumericLiterals
c = 3 # another regular comment
d = 4 # rubocop:enable Style/NumericLiterals

# Heredoc with directive
content = <<~HEREDOC # rubocop:disable Style/HeredocStyle
  hello world
HEREDOC

# Multiline method call with directive on closing line
result = very_long_method(
  arg1,
  arg2,
  arg3
) # rubocop:disable Metrics/ParameterLists

# Constant assignment with directive
CONST = "value" # rubocop:disable Style/MutableConstant

# Case/when with directive
case x
when 1 # rubocop:disable Style/WhenThen
  foo
when 2
  bar
end

# Rescue with directive
begin
  risky_op
rescue StandardError # rubocop:disable Lint/RescueException
  handle
end

# rubocop:push/pop directives
# rubocop:push Metrics/ClassLength
class VeryLongClass
  x = 1 # rubocop:pop Metrics/ClassLength
end
