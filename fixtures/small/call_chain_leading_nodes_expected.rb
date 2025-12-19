def example(this)
  this
    .should_break!
end

foo {
  _1
    .should_break!
}

bar {
  it
    .should_break!
}

$globals
  .should_break_too!

class Foo
  def self.do_things
    @this
      .should_break
    @@and_this
      .should_also_break
  end
end
