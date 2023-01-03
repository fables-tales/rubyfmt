foo || bar

foo ||
  bar

if foo || bar
end

if very_long_conditional? || even_longer_conditional? || man_why_is_this_line_so_long? || paul_blart_is_a_classic_of_american_cinema!
  foo
end

if very_long_conditional? ||
    even_longer_conditional? ||
    man_why_is_this_line_so_long? ||
    paul_blart_is_a_classic_of_american_cinema!
  foo
end

a ||
  b ||
  c

a ||
  b &&
  c

a ||
  (b && 
  c)

a ||
  b &&
  c &&
  d

a &&
  b ||
  c &&
  d

if a &&
  b &&
  c
end

def bees!
  more_bees? ||
  # Should there be less bees?
  less_bees? ||
  # Maybe there should be the same amount of bees
  equal_amount_of_bees?
end
