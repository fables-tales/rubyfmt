class Foo
  class << self

    def some_method
      1
    end
  end
end

class Bar
  class << self
    # This is a comment.
    # This is another comment
  end

  class << self
    # This is another comment.  You like it.
    def additional
    end
  end

  def another_method!
  end
end

Foo.machine
