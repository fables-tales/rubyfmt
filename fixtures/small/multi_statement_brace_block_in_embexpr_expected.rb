"value: #{proc {
  a
  b
}.call}"

"value: #{-> {
  a
  b
}.call}"

"value: #{lambda {
  a
  b
}.call}"

"value: #{foo {
  a
  b
}.call}"

"value: #{proc {
  a
  b
  c
}.call}"

class Foo
  def bar
    "value: #{proc {
      a
      b
    }.call}"
  end

  def baz
    if condition
      "nested: #{lambda {
        x
        y
        z
      }.call}"
    end
  end
end

"value: #{proc {
  # first
  a
  # second
  b
}.call}"

"value: #{proc {
  # leading comment
  a
  b
}.call}"
