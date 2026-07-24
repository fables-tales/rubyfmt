$g = 1
puts('"\#$g"')

@g = 2
puts('"\#@g"')

x = 3
puts('"\#{x}"')
puts(%q{text with \#{x} test})
