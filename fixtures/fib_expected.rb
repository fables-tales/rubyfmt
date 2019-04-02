def fib(i)
  a = 1
  b = 1

  i.times do
    tmp = a + b
    a = b
    b = tmp
  end

  b
end

def fibs
  10.times do |i|
    p(fib(i))
  end
end

fibs
