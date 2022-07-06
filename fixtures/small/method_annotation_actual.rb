sig { void }
def empty_example
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
  ).void
end
def example(a, b)
end

arbitrary_method_annotation('for science') do
  :no_exception
end
def another_one; end

_annotation color: "green", size: "large"
def m1; end
