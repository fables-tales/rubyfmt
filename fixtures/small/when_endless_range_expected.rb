case x
when 0.9.. then
  "very strong"
when 0.7.. then
  "strong"
when 0.5.. then
  "moderate"
end

case x
when 1.. then
  "positive"
end

case x
when ..0
  "non-positive"
end

case x
when 0..10
  "small"
end

case x
when 0.9.., :special
  "a"
end

case x
when 0.9.. then
  "very strong"
end

case x
when 1... then
  "exclusive endless"
end

case x
when :special, 0.9.. then
  "a"
end

case x
when (0.9..)
  "parenthesized endless"
end
