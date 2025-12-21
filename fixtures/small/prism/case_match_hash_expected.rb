# Hash pattern matching
case hash
in {name: n, age: a}
  puts("#{n} is #{a}")
in {name:}
  puts(name)
end

# Multiline hash pattern
case big_hash
in {
    name: n,
    age: a,
    email: e
  }
  puts(n)
in {
    foo:,
    bar:,
    baz:
  }
  puts(foo)
end

# Hash pattern without braces (should add braces)
case data
in {name: n, age: a}
  puts(n)
in {foo:, bar:}
  puts(foo)
end

# Hash pattern with parens (constant check)
case obj
in Point(x:, y:)
  puts(x + y)
in Rectangle(width: w, height: h)
  puts(w * h)
end

# Multiline hash pattern with parens
case obj
in Point(
    x:,
    y:,
    z:
  )
  puts(x + y + z)
in Rectangle(
    width: w,
    height: h
  )
  puts(w * h)
end
