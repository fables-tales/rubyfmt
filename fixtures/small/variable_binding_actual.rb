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
  self.head()
  self.head("oops arg")
  self.next.head
  self.next.head()
  self.next.head("oops arg")
end

bees

class Foo < T::Struct
  prop :name, String

  def has_same_name?(name:)
    name == name()
  end

  def in_a_parameter_list(
    with_parens = self.with_parens(),
    nope_parens = self.nope_parens,
    kw_with_parens: self.kw_with_parens(),
    kw_nope_parens: self.kw_nope_parens
  )
  end
end
