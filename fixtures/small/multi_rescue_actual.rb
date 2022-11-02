begin
  a
rescue A
  p "a"
rescue B
  p "b"
end

begin
  a
rescue A
  # ignore this exception
rescue B
  # actually do something with this
  b
rescue C
 c
  # something different
d
end
