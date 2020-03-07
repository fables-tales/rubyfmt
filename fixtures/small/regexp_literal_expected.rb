class Foo
  def bees
    %r{}
    %r{bees}
    /_color\z/i
    /_c.*olor\z/i
    /_c.*(ol)(or)\z/i
    /_c.*([ol])(or)\z/i
    /_c.*(\[ol])(or)\z/i
    %r({})
  end
end
