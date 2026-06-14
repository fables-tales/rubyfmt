puts ''
puts ""
puts %Q("")
puts %^_^
puts %^\"^
puts %^\\"^
puts %(\\"\))
puts '\\"'
puts '\"'
puts '\''
puts '\\"'
puts '\\\\"'
puts '"'
puts '\"'
puts "\""
puts "\\3\3"
puts %^\\"\^^
puts '\a^'
puts %^\\"#{'\a^'}\^^
puts %{{a#{1}}}
puts %{{a}#{1}}
puts %{\\\{#{1}}
puts %{\\{#{1}}}
puts <<EOD
"abc"\"
EOD

puts '
  some stuff \
  other_stuff \\
'
puts "
  more stuff \\
  even more stuff \
"
puts "
  group {
    person {
      attribute {
        slug
      }
    }
    created
    updated
    otherPeople {
      name
    }
  }
"

puts '
  group {
    person {
      attribute {
        slug
      }
    }
    created
    updated
    otherPeople {
      name
    }
  }
'

@foo = 3
puts '#@foo'
puts '#{3}'
puts %q("")
puts %q(\"\")
puts %Q(\"\")
puts '\"\"'
puts %q(\\"\\")
puts %q(\))
puts %Q(\))
puts %<foo\>>

puts(
  1,
  2,
  <<~TXT,
    3
    4
  TXT
)

puts(
  1,
  2,
  <<TXT,
    3
    4
TXT
)

def foo
  puts(
    1,
    2,
    <<~TXT,
      3
      4
    TXT
  )

  puts(
    1,
    2,
    <<TXT,
      3
      4
TXT
  )
end

foo

puts <<EOD.gsub("a", "b")
"cde"
EOD
