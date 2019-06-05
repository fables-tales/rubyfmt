#adsf
class Foo
  #but this one doesn't get a proceeding newline
  a
end

# this one gets a proceeding newline because it is prefixed by an end
class Bees
  foo

  # this one gets a proceeding newline because it's prefixed by a statement
  class Bar
  end

  def bees
    # no proceeding newline here
    a
  end
end
