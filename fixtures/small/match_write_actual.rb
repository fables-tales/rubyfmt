/(?<name>\w+)/ =~ str
puts name

/(?<first>\w+)\s+(?<last>\w+)/ =~ full_name

if /(?<year>\d{4})-(?<month>\d{2})-(?<day>\d{2})/ =~ date_str
  puts year
  puts month
  puts day
end

/(?<host>[^:]+):(?<port>\d+)/ =~ address

result = /(?<key>\w+)=(?<value>\w+)/ =~ pair

while /(?<line>.+)/ =~ input
  process(line)
end
