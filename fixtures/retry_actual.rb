class Foo
  def bees
    begin
    rescue
      retry
    end
  end
end

Foo.machine
