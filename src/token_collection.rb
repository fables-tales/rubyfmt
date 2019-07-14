class TokenCollection < SimpleDelegator
  def initialize(parts = [])
    super(parts)
    @parts = parts
  end

  def <<(v)
    @parts << v
  end

  def is_indent?
    false
  end

  def any?(&blk)
    @parts.any?(&blk)
  end

  def rindex_by(&blk)
    @parts.reverse_each.each_with_index do |item, idx|
      if blk.call(item)
        return @parts.length - idx
      end
    end

    nil
  end

  def last
    @parts.last
  end

  def [](arg)
    @parts[arg]
  end

  def each_flat(&blk)
    e = Enumerator.new do |yielder|
      @parts.each do |item|
        if TokenCollection === item

          item.each_flat do |i|
            yielder << i
          end

        else
          yielder << item
        end
      end
    end

    if blk
      e.each(&blk)
    else
      e
    end
  end

  def to_s
    join
  end

  def has_comment?
    any? { |x| Comment === x }
  end

  def string_length
    to_s.length
  end

  def multiline_string_length
    map(&:as_multi_line).map(&:to_s).join.length
  end

  def as_single_line
    TokenCollection.new(map { |x| x.as_single_line })
  end

  def as_multi_line
    TokenCollection.new(map { |x| x.as_multi_line })
  end

  def single_line_string_length
    map { |x|
      if x.is_indent?
        x.as_multi_line
      else
        x.as_single_line
      end
    }.map(&:to_s).sum(&:length)
  end

  def remove_redundant_indents
    shift if first && first.is_empty_string?
  end

  def strip_trailing_newlines
    while ends_with_newline?
      pop
    end
  end

  def ends_with_newline?
    last.is_a_newline?
  end

  def contains_end?
    each_flat.any? { |x| x.is_end? }
  end

  def contains_do?
    each_flat.any? { |x| x.is_do? }
  end

  def contains_else?
    each_flat.any? { |x| x.is_else? }
  end

  def declares_private?
    each_flat.any? { |x| x.is_private? }
  end

  def declares_require?
    each_flat.any? { |x| x.is_require? } && each_flat.none? { |x| x.to_s == "}" }
  end

  def declares_class_or_module?
    each_flat.any? { |x| x.declares_class_or_module? }
  end

  def contains_if_or_unless?
    each_flat.any? { |x| x.declares_if_or_unless? }
  end

  def contains_keyword?
    each_flat.any? { |x| x.is_keyword? }
  end

  def surpresses_blankline?
    contains_keyword?
  end

  def is_only_a_newline?
    length == 1 && first.is_a_newline?
  end
end
