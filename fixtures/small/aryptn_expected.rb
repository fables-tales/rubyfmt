case ["I will arise", "and go now", "and go to Innisfree"]
in [String, String]
  "and a small cabin build there, of clay and wattles made"
in SmallCabin["clay", "wattles"]
  "Nine bean-rows will I have there, a hive for the honey-bee,"
in BeeLoudGlade[String, *, String]
  "And live alone in the bee-loud glade"
end

case ["And I shall have some peace there", "for peace", "comes dropping slow"]
in [String, *]
  "Dropping from the veils of the morning to where the cricket sings;"
in Midnight[*]
  "There midnight's all a glimmer, and noon a purple glow,"
in [1, *, 2]
  "And evening full of the linnet's wings."
end

case []
in [*]
  0
in []
  1
else
  2
end
