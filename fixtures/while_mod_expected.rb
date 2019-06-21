# typed: ignore
class Foo
  def bees
    true while false
  end
end

Foo.machine
