class Foo
  memoize public def bar
    42
  end

  memoize protected def bar
  end

  memoize private def baz(a, b, c)
    a + b + c
  end

  memoize protected def with_body
    "hello"
  end

  memoize module_function def class_util
  end

  memoize public def no_body
  end

  memoize final public private protected def all_the_modifiers!
  end

  memoize def simple
    "no inner modifier"
  end
end
