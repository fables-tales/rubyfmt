sig { void }
def empty_example
end

sig {
  params(foo: SomePrettyLongClassName, bar: AnEvenLongerClassName::ThatMakesThisGoPrettyFar, baz: Hasdfasdfasdfasdas)
}
def do_stuff!(foo, bar, baz)
end

sig {
  # This method doesn't return anything weeeeee
  void
  # But you can bet it has some side effects
}
def do_stuff!
end

sig do
  params(a: T::Array[String], b: T::Hash[Symbol, String])
    .returns(T::Set[Symbol])
    .checked(:tests)
end
def do_the_thing(a, b)
  puts(a)
  puts(b)

  Set.new
end

sig(:final) do
  params(
    a: String,
    b: String
  )
    .void
end
def example(a, b)
end

arbitrary_method_annotation("for science") do
  :no_exception
end
def another_one
end

_annotation(color: "green", size: "large")
def m1
end

fake_annotation(
  {
    a: b
  }
)
def self.long_boi
end

long_annotation(
  [
    1,
    2,
    3
  ]
)
def self.multiline_boi
end

boop(
  %w[
    a
    b
  ]
)
def example
end

boop(
  %W[
    a
    b
  ]
)
def example
end

my_annotation do
end
private def my_method
end

class Bees
  sig {
    # These are the params
    params(
      first_param: MyClass,
      # This one is the second one, nice
      second_param: YourClass
    )
      .void
    # Please not the bees!
  }
  def not_the_bees!
  end
end
