if Method.call
  <<~HEREDOC
    text
  HEREDOC
end

unless Method.call
  <<~HEREDOC
    text
  HEREDOC
end

if Method.call || other_condition
  <<~HEREDOC
    text
  HEREDOC
end

if other_condition && Method.call
  <<~HEREDOC
    text
  HEREDOC
end

if Method.call
  x = <<~HEREDOC
    text
  HEREDOC
end

if Method.call
  @y = <<~HEREDOC
    text
  HEREDOC
end

<<~HEREDOC while x.foo
  text
HEREDOC

<<~HEREDOC until x.foo
  text
HEREDOC

if <<~PRED
    pred
  PRED
    .empty?
  something
end

if <<~PRED
    pred
  PRED
    .empty?
  <<~BODY
    body
  BODY
end

if <<~PRED
    is this enabled?
  PRED
    .include?("yes")
  puts(
    <<~OUT
      output
    OUT
  )
end
