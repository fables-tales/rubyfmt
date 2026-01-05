assert_equal(
  1,
  T.cast(T.must(pages[1]&.elements)[0], Name::Spacing::To::Extend::CodeQuality::AcrossManyLines).table[
    :line_items
  ]
    .length
)

foo.bar[some_really_long_index][another_really_long_index]
  .baz
  .qux

result.data[
  :first_key,
  :second_key
]
  .transform
  .process

outer_method(
  inner.call[0]
    .chain
    .another
)

matrix[row][col]
  .value
  .to_s
  .upcase
