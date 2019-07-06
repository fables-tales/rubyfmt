class Intermediary
  def initialize
    @content = TokenCollection.new([])
    @lines = []
    @build = []
  end

  def <<(x)
    @content << x
    if x.is_a_newline?
      @lines << TokenCollection.new(@build) unless @build.empty?
      @build = []
    else
      @build << x
    end
  end

  def insert_newline_before_last
    @content.insert(@content.rindex_by { |x| x.is_a_newline? } - 1, HardNewLine.new)
  end

  def string_length
    to_s.length
  end

  def to_s
    @content.to_s
  end

  def delete_last_newline
    @content.delete_at(@content.rindex_by { |x| x.is_a_newline? } - 1)
  end

  def prev_line
    return false if _line_length < 2
    if @build.empty?
      @lines[-2]
    else
      @lines[-1]
    end
  end

  def current_line
    return false if _line_length < 2
    line = @build
    if line.empty?
      line = @lines[-1]
    end
    line
  end

  def length
    @content.length
  end

  def last
    @content.last
  end

  def pop
    @content.pop
  end

  def pluck_chars(n)
    raise unless n == 3
    (@content[-n..-1] || []).tap { |x|
      # #raise "omg #{x.inspect}> #{@last_3.inspect}" if @last_3 != x
    }
  end

  def each(*args, &blk)
    @content.each(*args, &blk)
  end

  private

  def _line_length
    @lines.length + (@build.empty? ? 0 : 1)
  end
end

class RenderQueueDFA
  def initialize(render_queue)
    @render_queue_in = render_queue
    @render_queue_out = Intermediary.new
  end

  def render_as(q, target_collection, idx, &blk)
    q = q.dup
    orig_idx = idx
    breakable_state = target_collection[orig_idx]
    token = target_collection[orig_idx + 1]

    q << blk.call(token)
    nested_tokens = target_collection[orig_idx + 2]
    nested_index = 0
    while nested_index < nested_tokens.length
      t = nested_tokens[nested_index]

      if BreakableState === t
        q, nested_index = flatten_breakable_state(q, nested_tokens, nested_index)
      else
        q << blk.call(t)
      end

      nested_index += 1
    end

    idx = orig_idx + 3
    while target_collection[idx] != breakable_state
      q << blk.call(target_collection[idx])
      idx += 1
    end
    [q, idx]
  end

  def flatten_breakable_state(q, token_collection, idx)
    q = q.dup
    length = token_collection[idx+2].single_line_string_length
    if length > MAX_WIDTH
      q, idx = render_as(q, token_collection, idx, &:as_multi_line)
    else
      token_collection[idx+1] = NULL_DIRECT_PART
      q, idx = render_as(q, token_collection, idx, &:as_single_line)
      p q
      q.pop while [
        q.last.is_a_comma?,
        SoftNewLine === q.last,
        BreakableState === q.last,
        Space === q.last,
        NULL_DIRECT_PART == q.last,
      ].any?
    end

    [q, idx]
  end

  def call
    i = 0

    q = TokenCollection.new([])
    idx = 0
    while idx < @render_queue_in.length
      token = @render_queue_in[idx]
      if BreakableState === token
        q, idx = flatten_breakable_state(q, @render_queue_in, idx)
      else
        q << token
      end

      idx += 1
    end

    chars = q.each_flat.to_a
    while i < chars.length
      char = chars[i]
      pc = pluck_chars(3)
      case
      when is_end_and_not_end?(pc + [char])
        # make sure that ends have blanklines if they are followed by something
        # that isn't an end
        push_additional_newline
      when is_end_with_blankline?(pc + [char])
        # make sure the ends don't get extra blanklines
        # e.g.
        # if bees
        #   foo
        #
        # end
        #
        #this also somehow occurs with:
        #
        #  a do
        #    a =  begin
        #         end
        #  end
        #  which generates triple blanklines, so hence the while loop
        while is_end_with_blankline?(pluck_chars(3) + [char])
          c = @render_queue_out.delete_last_newline
          raise "omg" if !c.is_a_newline?
        end
      when is_comment_with_double_newline?(pc + [char])
        c = @render_queue_out.delete_last_newline
        raise "omg" if !c.is_a_newline?
      when is_non_requirish_and_previous_line_is_requirish(char)
        push_additional_newline
      when comment_wants_leading_newline?(char)
        push_additional_newline
      when do_block_wants_leading_newline?(char)
        push_additional_newline
      when class_wants_leading_newline?(char)
        push_additional_newline
      when if_wants_leading_newline?(char)
        push_additional_newline
      when private_wants_trailing_blankline?(pc + [char])
        push_additional_newline
      end

      @render_queue_out << char
      i += 1
    end

    while @render_queue_out.last.is_a_newline?
      @render_queue_out.pop
    end

    @render_queue_out
  end

  def push_additional_newline
    @render_queue_out.insert_newline_before_last
  end

  def pluck_chars(n)
    @render_queue_out.pluck_chars(n)
  end

  def if_wants_leading_newline?(char)
    return false unless char.declares_if_or_unless?
    return false unless prev_line

    return true unless prev_line.any? { |x| x.is_a_comment? || x.is_def? || x.declares_class_or_module? || x.is_do? || x.declares_if_or_unless? || x.is_else? } || prev_line.is_only_a_newline?
  end

  def private_wants_trailing_blankline?(chars)
    return false unless chars.length == 4

    chars[1].is_private? && chars[2].is_a_newline? && !chars[3].is_a_newline?
  end

  def class_wants_leading_newline?(char)
    return false unless char.declares_class_or_module?
    return false unless prev_line

    return true unless prev_line.any? { |x|
      x.is_a_comment? ||
      x.is_requirish? ||
      x.declares_class_or_module?
    } || prev_line.is_only_a_newline?
  end

  def have_end_with_double_blankline?(chars)
    return false unless chars.length == 5

    chars[0].is_end? && chars[1].is_a_newline? && chars[2].is_a_newline? && chars[3].is_indent? && chars[4].is_end?
  end

  def do_block_wants_leading_newline?(char)
    return false unless char.is_do?
    return false unless prev_line

    surpress = prev_line.any? { |x| x.declares_class_or_module? || x.is_def? || x.is_a_comment? || x.is_do? || x.is_end? }
    !surpress
  end

  def comment_wants_leading_newline?(char)
    return false unless char.is_a_comment?
    return false unless current_line

    surpress = current_line.any? { |x| x.declares_class_or_module? || x.is_def? || x.is_a_comment? }
    !surpress
  end

  def is_non_requirish_and_previous_line_is_requirish(char)
    return false unless char.is_a_newline?
    return false unless (prev_line && current_line)
    prev_line.any?(&:is_requirish?) && !(current_line.any?(&:is_requirish?))
  end

  def prev_line
    @render_queue_out.prev_line
  end

  def current_line
    @render_queue_out.current_line
  end

  def is_end_with_blankline?(chars)
    return false if chars.length != 4

    chars[0].is_a_newline? && chars[1].is_a_newline? && chars[2].is_indent? && chars[3].is_end?
  end

  def is_comment_with_double_newline?(chars)
    return false if chars.length != 4

    chars[1].is_a_comment? && chars[2].is_a_newline? && chars[3].is_a_newline?
  end

  def is_end_and_not_end?(chars)
    return false if chars.length < 4
    chars[0].is_end? && chars[1].is_a_newline? && chars[2].is_indent? && !chars[3].is_end?
  end
end
