class BreakableEntry < TokenCollection
  include TokenBase

  def initialize(parts = [])
    super
    @parts = parts
  end

  def inspect
    "<BreakableEntry: #{super}>"
  end

  def <<(x)
    @parts << x
  end

  def rindex_by(&blk)
    @parts.reverse_each.each_with_index do |item, idx|
      if blk.call(item)
        return @parts.length - idx
      end
    end

    nil
  end
end
