def foo
  things.each do |thing|
    if thing?
      do_stuff
    elsif other_thing?
      # skip
      next
    end
  end
end

def foo
  things.each do |thing|
    if thing?
      do_stuff
    elsif other_thing?
      # skip
      next 1
    end
  end
end
