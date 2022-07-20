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

puts(a)
puts(b)
foo
