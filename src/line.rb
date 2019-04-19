class Line < SimpleDelegator
  attr_accessor :parts
  def initialize(parts)
    @comments = []
    @breakable_entry_stack = []
    @parts = TokenCollection.new(parts)
    super(@parts)
  end

  def to_s
    build = join("")

    unless @comments.empty?
      build = "#{@comments.join("\n")}\n#{build}"
    end

    build
  end

  def breakable_entry(&blk)
    @breakable_entry_stack << BreakableEntry.new
    blk.call
    be = @breakable_entry_stack
    if be.length >= 2
      be_last = be.pop
      be_next = be.last
      be_next << be_last
    else
      @parts << @breakable_entry_stack.pop
    end
  end

  def <<(item)
    if be = @breakable_entry_stack.last
      be << item
    else
      @parts << item
    end
  end
end

def want_blankline?(line, next_line)
  return unless next_line
  return false if line.has_comment? && next_line.declares_class_or_module?
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.surpresses_blankline?
  return true if next_line.contains_if_or_unless? && !line.surpresses_blankline?
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
  return true if !line.declares_class_or_module? && next_line.has_comment?
  return true if !line.declares_class_or_module? && next_line.declares_class_or_module?
end

