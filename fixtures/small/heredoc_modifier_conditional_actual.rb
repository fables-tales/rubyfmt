<<~HEREDOC if Method.call
  text
HEREDOC

<<~HEREDOC unless Method.call
  text
HEREDOC

<<~HEREDOC if Method.call || other_condition
  text
HEREDOC

<<~HEREDOC if other_condition && Method.call
  text
HEREDOC

x = <<~HEREDOC if Method.call
  text
HEREDOC

@y = <<~HEREDOC if Method.call
  text
HEREDOC

<<~HEREDOC while x.foo
  text
HEREDOC

<<~HEREDOC until x.foo
  text
HEREDOC

something if <<~PRED.empty?
  pred
PRED

<<~BODY if <<~PRED.empty?
  body
BODY
  pred
PRED

puts(<<~OUT) if <<~PRED.include?("yes")
  output
OUT
  is this enabled?
PRED
