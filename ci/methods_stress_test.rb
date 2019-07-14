Kernel.srand(2345)
def a(*args, **kwargs)
  puts rand
end

def b(keyword:)
  a(keyword: keyword)
end

def c(arg)
  a(arg)
end

def d(arg)
  a arg
end

def f arg
  a arg
end


class Foo
  def call
    a
  end

  def call2(first, second)
    a(first, second)
  end

  def call2_noparens(first, second)
    a first, second
  end
end

a
a(*[1,2,3])
c(1)
d(2)
f(3)

a *[1,2,3]
c 1
d 2
f 3

f = Foo.new
f.call
f.call()
a(c(1))
a(c 1)
a(a)
a a
f.call2_noparens 1, 2

(Object::Foo).new.call
