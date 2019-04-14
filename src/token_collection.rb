class TokenCollection < SimpleDelegator
  def initialize(parts=[])
    super(parts)
  end

  def string_length
    join("").length
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
    any? { |x| x.is_end? }
  end

  def contains_do?
    any? { |x| x.is_do? }
  end

  def contains_else?
    any? { |x| x.is_else? }
  end

  def declares_private?
    any? { |x| x.is_private? }
  end

  def declares_require?
    any? { |x| x.is_require? } && none? { |x| x.to_s == "}" }
  end

  def declares_class_or_module?
    any? { |x| x.declares_class_or_module? }
  end

  def contains_if_or_unless?
    any? { |x| x.declares_if_or_unless? }
  end

  def contains_keyword?
    any? { |x| x.is_keyword? }
  end

  def surpresses_blankline?
    contains_keyword?
  end

  def is_only_a_newline?
    length == 1 && first.is_a_newline?
  end
end
