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

$global = 1
puts '\#$global'
puts '"\#$global"'

@ivar = 2
puts '\#@ivar'
puts '"\#@ivar"'

local_var = 3
puts '\#{local_var}'
puts '"\#{local_var}"'
puts %q{text with \#{local_var} test}
puts '\#foo'
puts 'a\#{local_var}c'
puts '\\#{local_var}'
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
