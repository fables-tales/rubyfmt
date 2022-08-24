a = <<EOD
part 1 of heredoc
part 2 of heredoc
EOD

b = <<-EOD
part 1 of heredoc
EOD

def foo
  c = <<~EOD
    part 1 of heredoc
  EOD
  d = <<-DASH
all the way over here
  DASH
  e = <<BARE
also over here, but same with the closing tag
BARE

  puts(c)
end

<<~FOO
  some stuff
  with a empty last line

FOO

<<-FOO
  some stuff
  with a empty last line

FOO

<<FOO
  some stuff
  with a empty last line

FOO

puts(a)
puts(b)
foo
