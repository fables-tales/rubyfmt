Foo.raise(bar) { baz }
Foo.raise(bar).map { baz }
foo(raise(Bar), baz)
RSpec.describe(foo).tap { }
def a(
  b: raise("")
) = 1

foo(a: raise("keyword"))
{a: raise("hash")}
[raise("array")]
[raise("array"), other]
foo(1, raise("second"))

foo(
  it("x") do
  end
)
foo(
  1,
  it("x") do
  end
)
[
  it("x") do
  end
]
{
  a: it("x") do
  end
}
[
  describe("x") do
  end
]
RSpec.describe foo do
end

foo(*raise("splat"))
foo(&raise("block"))

case value
when raise("condition")
  raise "branch"
end
