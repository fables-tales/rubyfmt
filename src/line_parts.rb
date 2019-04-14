class PartBase
  def is_a_blankline?
    false
  end
end

class HardNewLine < PartBase
  def to_s
    "\n"
  end

  def is_a_blankline?
    true
  end
end

class StringPart < PartBase
  def initialize(part)
    @part = part
  end

  def to_s
    part
  end

  def is_a_blankline?
    part == "\n"
  end
end

class BreakableEntry < PartBase
  def initialize
    @parts = []
  end

  def <<(item)
    @parts << item
  end

  def to_s
    @parts.join("")
  end
end

