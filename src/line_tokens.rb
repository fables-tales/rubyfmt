class TokenBase
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

  def is_require?
    false
  end

  def is_private?
    false
  end

  def is_empty_string?
    false
  end
end

class HardNewLine < TokenBase
  def to_s
    "\n"
  end

  def is_a_newline?
    true
  end
end

class DirectPart < TokenBase
  def initialize(part)
    @part = part
  end

  def to_s
    @part
  end

  def is_a_newline?
    @part == "\n"
  end

  def is_require?
    @part == "require"
  end

  def is_private?
    @part == "private"
  end

  def is_empty_string?
    @part == ""
  end
end

class SingleSlash < TokenBase
  def to_s
    "\\"
  end
end

class Binary < TokenBase
  def initialize(symbol)
    @symbol = symbol
  end

  def to_s
    " #{@symbol} "
  end
end

class Space < TokenBase
  def to_s
    " "
  end
end

class Dot < TokenBase
  def to_s
    "."
  end
end

class LonelyOperator < TokenBase
  def to_s
    "&."
  end
end

class OpenParen < TokenBase
  def to_s
    "("
  end
end

class CloseParen < TokenBase
  def to_s
    ")"
  end
end

class OpenArgPipe < TokenBase
  def to_s
    "|"
  end
end

class CloseArgPipe < TokenBase
  def to_s
    "|"
  end
end

class DoubleQuote < TokenBase
  def to_s
    "\""
  end
end

class OpenSquareBracket < TokenBase
  def to_s
    "["
  end
end

class CloseSquareBracket < TokenBase
  def to_s
    "]"
  end
end

class Keyword < TokenBase
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

class Indent < TokenBase
  def initialize(spaces)
    @spaces = spaces
  end

  def to_s
    " " * @spaces
  end
end

class CommaSpace < TokenBase
  def to_s
    ", "
  end
end

class Comma < TokenBase
  def to_s
    ","
  end
end

class Op < TokenBase
  def initialize(op)
    @op = op
  end

  def to_s
    @op
  end
end
