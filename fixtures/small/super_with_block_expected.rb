super do
end

super(1) do
end

class X
  def m(&blk)
    # super-duper
    super do |x|
      yield(maybe_track("*", x))
    end
  end
end
