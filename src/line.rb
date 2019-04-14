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

  def remove_redundant_indents
    @parts.shift if @parts[0] == ""
  end

  def strip_trailing_newlines
    while ends_with_newline?
      @parts.pop
    end
  end

  def ends_with_newline?
    @parts.last.respond_to?(:is_a_newline?) && @parts.last.is_a_newline?
  end

  def contains_end?
    @parts.any? { |x| x.respond_to?(:is_end?) && x.is_end? }
  end

  def contains_do?
    @parts.any? { |x| x.respond_to?(:is_do?) && x.is_do? }
  end

  def contains_else?
    @parts.any? { |x| x.respond_to?(:is_else) && x.is_else? }
  end

  def declares_private?
    @parts.any? { |x| x == "private" } && @parts.length == 3
  end

  def declares_require?
    @parts.any? { |x| x == "require" } && @parts.none? { |x| x == "}" }
  end

  def declares_class_or_module?
    @parts.any? { |x| x.respond_to?(:declares_class_or_module?) && x.declares_class_or_module? }
  end

  def contains_if_or_unless?
    @parts.any? { |x| x.respond_to?(:declares_if_or_unless?) && x.declares_if_or_unless? }
  end

  def contains_keyword?
    @parts.any? { |x| x.respond_to(:is_keyword?) && x.is_keyword? }
  end

  def surpresses_blankline?
    contains_keyword?
  end
end

def want_blankline?(line, next_line)
  return unless next_line
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.surpresses_blankline?
  return true if next_line.contains_if_or_unless? && !line.surpresses_blankline?
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
  return true if !line.declares_class_or_module? && next_line.has_comment?
  return true if !line.declares_class_or_module? && next_line.declares_class_or_module?
end

