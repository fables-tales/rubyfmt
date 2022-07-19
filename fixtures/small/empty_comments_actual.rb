def foo
  #a
  #b
  #c
  #d
end

if a
  # a
  # b
  # c
  # d
  # e
else
  # a
  # b
  # c
  # d
  # e
end

def bad?; end

# Is this good?
def good?; end

class GoodOrEvil
  # Are we the baddies?
  def self.bad?; end

  # Is this good?
  def self.good?; end
end
