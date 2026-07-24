def a = 1
def b = 1

def c(&d)
  b(&d)
  b = 1
  b(&d)

  a(&d)
  a.(&d)
end
