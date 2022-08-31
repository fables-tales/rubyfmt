for_splats = {}

labeled_hash = {
  label1: :value1,
  label2: :value2,
  **for_splats,
  label3: :value3
}

single_line_labeled_hash = {label1: :value1, label2: :value2, label3: :value3}

labeled_hash_trailing_comma = {
  label1: :value1,
  **for_splats,
  label2: :value2,
  label3: :value3
}

mixed_hash = {
  :label1 => :value1,
  :label2 => :value2,
  **for_splats,
  :label3 => :value3
}

single_line_mixed_hash1 = {:label1 => :value1, :label2 => :value2, :label3 => :value3}
single_line_mixed_hash2 = {:label1 => :value1, :label2 => :value2, :label3 => :value3}

mixed_hash_trailing_comma = {
  :label1 => :value1,
  **for_splats,
  :label2 => :value2,
  :label3 => :value3
}

mixed_hash_expression_keys = {
  :"a" => :value1,
  :b => :value2
}

to_the_moon = {
  :label1 => :value1,
  :label2 => :value2,
  **for_splats,
  :label3 => :value3
}

single_line_to_the_moon = {:label1 => :value1, :label2 => :value2, :label3 => :value3}

to_the_moon_trailing_comma = {
  :label1 => :value1,
  **for_splats,
  :label2 => :value2,
  :label3 => :value3
}
