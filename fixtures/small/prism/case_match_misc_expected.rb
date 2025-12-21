# Alternation pattern
case x
in 1 | 2 | 3
  puts("small")
in 10 | 20
  puts("medium")
end

# Capture pattern
case obj
in String => s
  puts(s.upcase)
in Integer => i
  puts(i * 2)
end

# Pinned variables
x = 5
case value
in ^x
  puts("equals x")
in ^(x + 1)
  puts("equals x + 1")
end

# Nested patterns
case data
in {users: [{name: n, age: a}, *rest]}
  puts("first user: #{n}")
in [1, [2, 3], {key: v}]
  puts(v)
end

# Find pattern with brackets
case arr
in [*before, 42, *after]
  puts("found 42")
end

# Find pattern without brackets (should add brackets)
case arr
in [*before, 42, *after]
  puts("found 42")
end

# Complex nested multiline
case response
in {
    status: 200,
    body: {
        users: [first, *rest]
      }
  }
  puts(first)
end
