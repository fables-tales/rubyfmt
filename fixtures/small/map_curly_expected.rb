module Foo
  def slowest_examples
    groups.map { |a, b| b }
    groups.map { |a, b|
      a
    }
  end
end
