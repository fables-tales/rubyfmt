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
  puts c
end

puts a
puts b
foo
