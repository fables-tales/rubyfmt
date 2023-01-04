y = 12
lambda do |x, ;y|
  puts(x)
  puts(y)
  y = 19
  puts(y)
end
  .call(17)
puts(y)

a do |;x|
end
