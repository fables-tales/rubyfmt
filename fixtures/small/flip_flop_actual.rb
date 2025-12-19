while line = gets
  print line if line =~ /start/ .. line =~ /end/
end

while line = gets
  print line if line =~ /begin/ ... line =~ /finish/

  if line =~ /begin/ ... line =~ /finish/
    print line
  end
end

(1..20).each { |x| puts x if x == 5 .. x == 10 }

items.each do |item|
  process(item) if item.active? .. item.final?
end

data.each { |d| d if (d > 10 && d < 20)  ..  (d > 50) }

if condition
  result if start_cond..end_cond
end
