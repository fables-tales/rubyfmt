class Intermediary
  def initialize
    @content = []
    @lines = []
    @build = []
  end

  def <<(x)
    @content << x
    if HardNewLine === x
      @lines << @build
      @build = []
    else
      @build << x
    end
  end

  def insert_newline_before_last
    @content.insert(@content.rindex_by { |x| HardNewLine === x } - 1, HardNewLine.new)
  end

  def delete_last_newline
    @content.delete_at(@content.rindex_by { |x| HardNewLine === x } - 1)
  end

  def prev_line
    return false if lines.length < 2
    TokenCollection.new(lines[-2])
  end

  def current_line
    return false if lines.length < 2
    _current_line
  end

  def _current_line
    line = @build
    i = -1
    while line.empty?
      line = @lines[i]
      i -= 1
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
    @content[-n..-1] || []
  end

  def each(*args, &blk)
    @content.each(*args, &blk)
  end

  private

  def lines
    @content.split { |x| HardNewLine === x }
  end
end

class RenderQueueDFA
  def initialize(render_queue)
    @render_queue_in = render_queue
    @render_queue_out = Intermediary.new
  end

  def call
    @render_queue_in.each_flat.each_with_index do |char, i|
      case
      when is_end_and_not_end?(pluck_chars(3) + [char])
        # make sure that ends have blanklines if they are followed by something
        # that isn't an end
        push_additional_newline
      when is_end_with_blankline?(pluck_chars(3) + [char])
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
          raise "omg" if !(HardNewLine === c)
        end
      when is_comment_with_double_newline?(pluck_chars(2) + [char])
        c = @render_queue_out.delete_last_newline
        raise "omg" if !(HardNewLine === c)
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
      when private_wants_trailing_blankline?(pluck_chars(2) + [char])
        push_additional_newline
      end

      @render_queue_out << char
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
    return false unless chars.length == 3

    chars[0].is_private? && chars[1].is_a_newline? && !chars[2].is_a_newline?
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
    return false unless HardNewLine === char
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
    return false if chars.length != 3

    chars[0].is_a_comment? && chars[1].is_a_newline? && chars[2].is_a_newline?
  end

  def is_end_and_not_end?(chars)
    return false if chars.length < 4
    chars[0].is_end? && chars[1].is_a_newline? && chars[2].is_indent? && !chars[3].is_end?
  end
end
