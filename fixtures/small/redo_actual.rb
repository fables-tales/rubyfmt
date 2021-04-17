class Foo
  def bees
    begin
    rescue
      redo
    end
  end
end

Foo.machine
