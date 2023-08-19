class Foo
  private def bar
    42
  end

  private def baz(a, b, c)
    a + b + c
  end

  private def with_comments
    # beginning
    stuff!
    # end
  end
end

module WhiteTeaBowl
  sig do
    params(walking_this: Path)
  end
  module_function def i_choose_one; end

  sig do
    void
  end
  patch_of def sunlight
    "after another"
  end
end
