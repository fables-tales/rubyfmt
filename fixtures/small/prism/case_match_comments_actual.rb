# Comments in case/in pattern matching
case   value
in   [a, b]
   # comment inside body
   puts a
in   {name: n}
  # another body comment
  puts n
else
  # comment in else body
  puts "no match"
end

# Comments in multiline patterns
case   data
in   [
       first,
       second
     ]
   # body after multiline array pattern
   puts first
in   {
       name: n,
       age: a
     }
  # body after multiline hash pattern
  puts n
end

# Comments with alternation patterns
case   x
in   1 | 2 | 3
  # matched small numbers
  puts "small"
in   10 | 20
  # matched larger numbers
  puts "medium"
end
