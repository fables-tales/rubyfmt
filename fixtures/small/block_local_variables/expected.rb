y = 12
lambda { |x;y|
  puts(x)
  puts(y)
  y = 19
  puts(y)
}.call(17)
puts(y)

a do |;x|
end
