[:heredoc_string_literal, ["<<", "EOD"]]
[:heredoc_string_literal, ["<<", "EOM"]]
[:heredoc_string_literal, ["<<-", "EOD"]]
[:heredoc_string_literal, ["<<~", "EOD"]]
a = <<EOD
part 1 of heredoc #{"not a heredoc" + <<EOM
eom part
EOM
}
part 2 of heredoc
EOD
b = <<-EOD
oweqijfoiwjefqwoefij
EOD
def foo
  c = <<~EOD
oqweijfoqwiejf
EOD
  puts(c)
end

puts(a)
puts(b)
foo
