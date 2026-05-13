x = 1
y = :key

single_line = {:x => x, y => 2}

multi_line = {
  :x => x,
  y => 2
}

leading_rocket = {y => 2, :x => x}

with_other_pairs = {:a => 1, :x => x, y => 2}

with_splats = {:x => x, y => 2, **z}

only_shorthands = {x:, y:}
