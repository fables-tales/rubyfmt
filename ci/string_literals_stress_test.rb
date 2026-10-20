# Override backtick execution for the `%x`/backtick literal cases
def `(cmd); cmd; end

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
puts %q{\'}
puts %q(\')
puts %q[\']
puts %q|\'|
puts %Q{\'}
puts %Q(\')
puts %q{it\'s}
puts %q(it\'s)

puts %Q"a\"b"
puts %"a\"b"
puts %Q"a\"b\"c"
local_q = 4
puts %Q"a\"#{local_q}b"

puts %x(echo hello)
puts %x(echo `date`)
puts %x(foo\tbar)
puts %x{foo\{bar}
puts %x{foo\}bar}
puts %x[foo\[bar]
puts %x[foo\]bar]
puts %x|foo\|bar|
puts %x(foo#{1}bar)
puts %x{foo#{1}\{bar}
puts %x(foo\\bar)
puts `echo hello`
puts `foo\`bar`

puts %r{foo}.source
puts %r{foo\{bar}.source
puts %r{foo\}bar}.source
puts %r(foo\(bar\)).source
puts %r[foo\[bar\]].source
puts %r|foo\|bar|.source
puts /foo\/bar/.source
puts %r{foo#{1}bar}.source

puts %s(foo).inspect
puts %s[foo].inspect
puts %s{foo}.inspect

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
