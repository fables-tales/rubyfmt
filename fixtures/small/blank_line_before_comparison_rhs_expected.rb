def test
  @x.foo(
    "b"
  ) == "d"

  @y.bar(
    "c"
  ) ==
    # Some comment about why this is formatted funkily
    "e"
end
