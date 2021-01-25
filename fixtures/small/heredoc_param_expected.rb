puts(
  <<~STRING,
yellow
fruit
peel
  STRING
  <<-STRING2,
    cheese
    noodle
    sauce
STRING2
  :burrito,
  <<~STRING3,
jasmine
earl grey
oolong
  STRING3
  :rice
)

puts([:one, <<~TWO, :three], :four, [:five, <<~SIX, :seven], :eight)
two
    TWO
six
    SIX
