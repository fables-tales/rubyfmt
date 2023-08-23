a = <<EOD
part 1 of heredoc #{"not a heredoc" + <<EOM}
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
  puts(c)
end

puts(a)
puts(b)
foo

ASSIGNED_MESSAGE = lambda do |assignee|
  <<-END
    This heredoc chains together in a weird way, talk to [~#{assignee}] about it.
    Otherwise, it's probably in your best interest not to write things like this.
  END
    .lines
    .map { |line| line.sub("/^[ \\t]+|[ \\t]+$/", "") }
    .join
    .strip
end

ASSIGNED_MESSAGE = lambda do |assignee|
  <<-END
    This heredoc chains together in a weird way, talk to [~#{assignee}] about it.
    Otherwise, it's probably in your best interest not to write things like this.
  END
    .lines
    .join
    .strip
end
