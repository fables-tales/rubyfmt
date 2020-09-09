class Foo
  def bees
    x ? yield(x) : nil
  end
end
