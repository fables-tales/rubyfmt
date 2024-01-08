case bees!
in {}
  ()
in { bees: "bzzz" }
  ()
in { bees: "bzzz", **nil }
  ()
in bees:
  ()
in bees: this_is_a_bee
    ()
in Beehive(queen_bee:, drones:)
  ()
in Beehive(queen_bee:beyonce, drones:)
  ()
in Beehive(queen_bee:, **nil)
  ()
in Beehive(queen_bee:, **rest_of_the_hive)
  ()
in Beehive(queen_bee:, **)
  ()
end
