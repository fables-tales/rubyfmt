class Line
  attr_accessor :parts
  def initialize(parts)
    @comments = []
    @breakable_entry_stack = []
    @parts = parts
  end

  def breakable_entry(&blk)
    #@breakable_entry_stack << BreakableEntry.new
    blk.call
    #@parts << @breakable_entry_stack.pop
  end

  def push_comment(comment)
    @comments << comment
  end

  def has_comment?
    !@comments.empty?
  end

  def <<(item)
    if be = @breakable_entry_stack.last
      be << item
    else
      @parts << item
    end
  end

  def string_length
    @parts.join("").length
  end

  def empty?
    @parts.empty?
  end

  def to_s
    build = @parts.join("")

    unless @comments.empty?
      build = "#{@comments.join("\n")}\n#{build}"
    end

    build
  end

  def strip_trailing_newlines
    while ends_with_newline?
      @parts.pop
    end
  end

  def remove_redundant_indents
    @parts.shift if @parts[0] == ""
  end

  def ends_with_newline?
    HardNewLine === @parts.last
  end

  def is_only_a_newline?
    @parts.length == 1 && HardNewLine === @parts[0]
  end

  def contains_end?
    @parts.any? { |x| x == :end }
  end

  def contains_def?
    @parts.any? { |x| x == :def }
  end

  def contains_do?
    @parts.any? { |x| x == :do }
  end

  def contains_if?
    @parts.any? { |x| x == :if }
  end

  def contains_else?
    @parts.any? { |x| x == :else }
  end

  def contains_unless?
    @parts.any? { |x| x == :unless }
  end

  def declares_private?
    @parts.any? { |x| x == "private" } && @parts.length == 3
  end

  def declares_require?
    @parts.any? { |x| x == "require" } && @parts.none? { |x| x == "}" }
  end

  def declares_class_or_module?
    @parts.any? { |x| x == :class || x == :module }
  end

  def contains_while?
    @parts.any? { |x| x == :while }
  end

  def surpresses_blankline?
    contains_def? || contains_do? || contains_while? || contains_if? || contains_else?
  end
end

def want_blankline?(line, next_line)
  return unless next_line
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.surpresses_blankline?
  return true if (next_line.contains_if? || next_line.contains_unless?) && !line.surpresses_blankline?
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
  return true if !line.declares_class_or_module? && next_line.has_comment?
  return true if !line.declares_class_or_module? && next_line.declares_class_or_module?
end

