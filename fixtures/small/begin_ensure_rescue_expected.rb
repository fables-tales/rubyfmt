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

  def func_with_comments
    begin
      foo!
    rescue
      # We could not foo!
      # Our sincerest apologies
    end
  end

  def func_with_mulilines
    _lambda_to_break_stuff = lambda do |this, that, the_other|
      wreak_havoc!
    end

    begin
      foo = ThisCalls.multiline_method(
        thing: "wow",
        other_thing: "even more wow"
      )
    rescue
      ThisCalls.another_multiline_method(
        this: "*Owen Wilson Voice* wow",
        that: "I don't know more ways to say wow",
        the_other: "wowie zowie"
      )
    else
      ThisCalls.a_third_multiline_method(
        first: "WOWOWOWOWOWOW",
        second: "wOwOwOwOwOwOw"
      )
    ensure
      ThisCalls.the_last_multiline_method(
        bada_bing: "zowie wowie",
        bada_boom: "WoWoWoWoWoWoW"
      )
    end
  end
end
