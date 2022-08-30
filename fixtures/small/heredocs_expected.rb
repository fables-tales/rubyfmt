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

class Foo
  ERROR_MESSAGE = T.let(
    <<~MESSAGE,
      Something bad happened! Aaahhh!
    MESSAGE
    String
  )
end

class Foo
  description(
    <<~DESC
      thing
    DESC
  )

  # namespace
  stable_id "really_stable_id"
end

add_offense(
  node,
  message: <<~RB
    Oh no you did something bad to rubocop.
    rubocop SMASH
  RB
) do |bad_thing|
  rubocop.smash(bad_thing)
end

puts(a)
puts(b)
foo
