# Array pattern matching
case    arr
in     [a, b, c]
   puts a + b + c
in  [first, *rest]
   puts first
in   []
  puts "empty"
end

# Array pattern without brackets (should add brackets)
case   tuple
in   a, b, c
   puts a
in  first, *rest
  puts first
end

# Multiline array pattern
case   big_array
in   [
       first,
       second,
       *rest
     ]
   puts first
in  [a,
     b,
     c]
  puts a + b + c
end
