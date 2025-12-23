require 'bees'
class Foo; end
def Foo; end

Foo 1
Foo()
Foo

foo::Bar::baz
foo::Bar()
foo.bar.Baz.quux
Foo::Bar
Foo::Bar()
Foo.Bar
Foo.Bar()
Bar(1, &fun_returning_blk)
Bar(&fun_returning_blk)
Bar do
  other_call
end
