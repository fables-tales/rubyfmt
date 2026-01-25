a = <<EOD
part 1 of heredoc #{ "not a heredoc" + <<EOM }
eom part
EOM
part 2 of heredoc
EOD


b = <<-EOD
oweqijfoiwjefqwoefij
EOD


def foo
  c = <<~EOD
    oqweijfoqwiejf
  EOD
  puts c
end

puts a
puts b
foo

ASSIGNED_MESSAGE = lambda do |assignee|
  <<-END.lines.map {|line| line.sub('/^[ \t]+|[ \t]+$/', '')}.join.strip
    This heredoc chains together in a weird way, talk to [~#{assignee}] about it.
    Otherwise, it's probably in your best interest not to write things like this.
  END
end

ASSIGNED_MESSAGE = lambda do |assignee|
  <<-END.lines.join.strip
    This heredoc chains together in a weird way, talk to [~#{assignee}] about it.
    Otherwise, it's probably in your best interest not to write things like this.
  END
end

<<EOD
part 1 of heredoc #{"not a heredoc" + <<EOM} after brace before newline
contents of EOM
EOM
contents of EOD
EOD

# Multiple heredocs on same line
<<OUTER
first #{<<A} middle #{<<B} last
content A
A
content B
B
OUTER

# Heredoc-only interpolation with text after
<<OUTER
#{<<INNER} after
inner content
INNER
more outer
OUTER

# Heredoc with escape sequences preserved
<<EOD
line with escape \n in middle
another line
EOD

# Nested heredoc with escape sequence in interpolation
<<OUTER
prefix #{"text\n" + <<INNER} after
inner content
INNER
more outer
OUTER

# Squiggly heredoc with nested bare heredoc
<<~OUTER
  prefix #{<<INNER} after
inner content
INNER
  more outer
OUTER

# Squiggly heredoc with nested squiggly heredoc
<<~OUTER
  prefix #{<<~INNER} after
  inner content
  INNER
  more outer
OUTER
