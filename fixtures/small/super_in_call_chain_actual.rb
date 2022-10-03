class Foo < Bar
  def initialize(a)
    @a = a
    super().do_stuff!
    super.other_stuff!
  end
end
