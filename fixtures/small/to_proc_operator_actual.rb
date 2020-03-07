module Foo
  def slowest_examples
    example.bees(a, &:foo)

    example.map(&:foo)
  end
end
