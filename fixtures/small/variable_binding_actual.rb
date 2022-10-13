def head()
  p "hi"
  :bees
end
def foo(head=head())
  p head
end

foo

def bees
  head = 1
  head()
  p head
end

def cheese
  head = 1
  self.head
end

bees

class Foo < T::Struct
  prop :name, String

  def has_same_name?(name:)
    name == name()
  end
end
