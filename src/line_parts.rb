class PartBase
  def is_a_newline?
    false
  end

  def is_keyword?
    false
  end

  def declares_class_or_module?
    false
  end

  def declares_if_or_unless?
    false
  end

  def is_end?
    false
  end

  def is_do?
    false
  end

  def is_def?
    false
  end

  def is_else?
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
end

class StringPart < PartBase
  def initialize(part)
    @part = part
  end

  def to_s
    part
  end

  def is_a_newline?
    part == "\n".tap { |res|
      raise "is a newline returned true on a string, which should be impossible" if res
    }
  end
end

class SingleSlash < PartBase
  def to_s
    "\\"
  end
end

class Binary < PartBase
  def initialize(symbol)
    @symbol = symbol
  end

  def to_s
    " #{symbol} "
  end
end

class Space < PartBase
  def to_s
    " "
  end
end

class Dot < PartBase
  def to_s
    "."
  end
end

class LonelyOperator < PartBase
  def to_s
    "&."
  end
end

class OpenParen < PartBase
  def to_s
    "("
  end
end

class CloseParen < PartBase
  def to_s
    ")"
  end
end

class OpenSquareBracket < PartBase

end

class CloseSquareBracket < PartBase

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

  def declares_if_or_unless?
    @keyword == :if || @keyword == :unless
  end

  def is_end?
    @keyword == :end
  end

  def is_do?
    @keyword == :do
  end

  def is_def?
    @keyword == :def
  end

  def is_else?
    @keyword == :do
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
