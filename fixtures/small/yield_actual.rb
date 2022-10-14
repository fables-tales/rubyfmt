class Foo
  def bees
    yield call(foo)
  end
end

def self.rubyfmt(&blk)
  yield "hello"
  # We've now yielded are are about to set foo to 1
  foo = 1
end
