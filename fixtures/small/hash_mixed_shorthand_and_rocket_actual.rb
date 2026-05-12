x = 1
y = :key

single_line = { x:, y => 2 }

multi_line = {
  x:,
  y => 2
}

leading_rocket = { y => 2, x: }

with_other_pairs = { a: 1, x:, y => 2 }

with_splats = { x:, y => 2, **z}
