class ForIndents
  def func
  rescue Bees => e
    a
  end

  def func2
    begin
    rescue Bees
      a
    end
  end

  def func3
  ensure
    a
  end


  def func4
    begin
    rescue Bees
      a
    ensure
      b
    end
  end
end
