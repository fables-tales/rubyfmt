class PartBase
  def is_a_newline?
    false
  end
end

class HardNewLine < PartBase
  def to_s
    "\n"
  end

  def is_a_newline?
    true
  end

  def is_keyword?
    false
  end

  def declares_class_or_module?
    false
  end
end

class StringPart < PartBase
  def initialize(part)
    @part = part
  end

  def to_s
    part
  end

  def is_a_newline?
    part == "\n"
  end
end

class EndKeyword < PartBase
end

class Keyword < PartBase
  def initialize(keyword)
    @keyword = keyword
  end

  def is_keyword?
    true
  end

  def declares_class_or_module?
    @keyword == :class || @keyword == :module
  end

  def declares_if_unless?
    @keyword == :if || @keyword == :unless
  end

  def to_s
    @keyword.to_s
  end
end

class Indent < PartBase
  def initialize(spaces)
    @spaces = spaces
  end

  def to_s
    " " * @spaces
  end
end

class CommaSpace < PartBase
  def to_s
    ", "
  end
end

class Comma < PartBase
  def to_s
    ","
  end
end
