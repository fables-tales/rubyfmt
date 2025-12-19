if /foo/
  puts "matched"
end

while /pattern/
  break
end

unless /bar/i
  puts "not matched"
end

result = /test/ ? "yes" : "no"

puts "found" if /keyword/

x = 1 until /done/

if /multi/m
  puts "multiline mode"
end

if /extended/x
  puts "extended mode"
end
