expected_result = <<-'EXPECTED'
One plus one is #{1 + 1}
EXPECTED

<<~"RUBY"
  puts("really cool stuff")
RUBY

MIND_BLOWING_RUBY = T.let(
  <<~'RUBY',
    puts("Ruby in Ruby, whoooooaaaaaaa")
  RUBY
  String
)

# prints: "One plus one is \#{1 + 1}\n"
p(expected_result)
