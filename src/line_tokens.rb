module TokenBase
  def as_single_line
    self
  end

  def as_multi_line
    self
  end

  def is_a_comma?
    false
  end

  def is_a_newline?
    false
  end

  def is_a_comment?
    false
  end

  def is_keyword?
    false
  end

  def is_indent?
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

  def is_requirish?
    false
  end

  def is_private?
    false
  end

  def is_empty_string?
    false
  end
end

class HardNewLine
  include TokenBase
  def to_s
    "\n"
  end

  def is_a_newline?
    true
  end
end

class SoftNewLine
  include TokenBase
  def to_s
    "\n"
  end

  def as_single_line
    Space.new
  end

  def is_a_newline?
    true
  end
end

class CollapsingNewLine
  include TokenBase
  def to_s
    "\n"
  end

  def as_single_line
    DirectPart.new("")
  end

  def is_a_newline?
    true
  end
end

class DirectPart
  include TokenBase
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

  def is_requirish?
    require_regex = /([^A-Za-z0-9]|^)require([^A-Za-z0-9])?/
    require_regex === @part
  end

  def is_private?
    @part == "private"
  end

  def is_empty_string?
    @part == ""
  end
end

class SingleSlash
  include TokenBase
  def to_s
    "\\"
  end
end

class Binary
  include TokenBase
  def initialize(symbol)
    @symbol = symbol
  end

  def to_s
    " #{@symbol} "
  end
end

class Space
  include TokenBase
  def to_s
    " "
  end
end

class Dot
  include TokenBase
  def to_s
    "."
  end
end

class LonelyOperator
  include TokenBase
  def to_s
    "&."
  end
end

class OpenParen
  include TokenBase
  def to_s
    "("
  end
end

class CloseParen
  include TokenBase
  def to_s
    ")"
  end
end

class OpenArgPipe
  include TokenBase
  def to_s
    "|"
  end
end

class CloseArgPipe
  include TokenBase
  def to_s
    "|"
  end
end

class DoubleQuote
  include TokenBase
  def to_s
    "\""
  end
end

class OpenSquareBracket
  include TokenBase
  def to_s
    "["
  end
end

class CloseSquareBracket
  include TokenBase
  def to_s
    "]"
  end
end

class Keyword
  include TokenBase
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
    @keyword == :else
  end

  def to_s
    @keyword.to_s
  end
end

class Indent
  include TokenBase
  def initialize(spaces)
    @spaces = spaces
  end

  def to_s
    " " * @spaces
  end

  def is_indent?
    true
  end
end

class SoftIndent
  include TokenBase
  def initialize(spaces)
    @spaces = spaces
  end

  def to_s
    " " * @spaces
  end

  def as_single_line
    DirectPart.new("")
  end

  def is_indent?
    true
  end
end

class CommaSpace
  include TokenBase
  def to_s
    ", "
  end
end

class Comma
  include TokenBase
  def is_a_comma?
    true
  end

  def to_s
    ","
  end
end

class Op
  include TokenBase
  def initialize(op)
    @op = op
  end

  def to_s
    @op
  end
end

class Comment
  include TokenBase
  def initialize(content)
    @content = content
  end

  def is_a_comment?
    true
  end

  def to_s
    @content.to_s
  end
end

class CommentBlock
  include TokenBase

  attr_reader :comments

  def initialize
    @comments = []
  end

  def empty?
    @comments.empty?
  end

  def add_comment(comment)
    raise ArgumentError, "got something other than a comment" unless Comment === comment
    @comments << comment
  end

  def merge(other)
    @comments += other.comments
  end

  def to_token_collection
    TokenCollection.new(@comments.flat_map { |x| [x, HardNewLine.new] })
  end

  def is_a_comment?
    true
  end
end
