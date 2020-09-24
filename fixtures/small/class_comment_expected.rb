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

  # this one gets a proceeding newline because it's prefixed by a statement
  def bees
    # no proceeding newline here
    a
  end

  def empty_method
    # comment inside empty method
    # second line of comment
    # third line of comment
    # fourth line of comment
  end

  # comment after empty method
  def empty_method_end_of_file
    # comment inside empty method
    # second line of comment
    # third line of comment
    # fourth line of comment
  end

  # comment after empty method
end
