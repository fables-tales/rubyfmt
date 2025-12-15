class Foo
  class << self

    def some_method
      1
    end
  end
end

class Bar
  class << self
  end

  def another_method!
  end
end

Foo.machine
