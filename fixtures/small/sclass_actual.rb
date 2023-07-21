class Foo
  class << self
  end
end

class Bar
  class << self
  end

  def another_method!; end
end

Foo.machine
