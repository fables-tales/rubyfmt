class RenderQueueDFA
  def initialize(render_queue)
    @render_queue_in = render_queue.flatten
    @render_queue_out = TokenCollection.new
  end

  def call
    i = 0
    while i < @render_queue_in.length
      char = @render_queue_in[i]

      case
      when is_end_and_not_end?(pluck_chars(3) + [char])
        # make sure that ends have blanklines if they are followed by something
        # that isn't an end
        @render_queue_out << HardNewLine.new
      when is_end_blankline_end?(pluck_chars(4) + [char])
        # make sure that something like:
        # module A
        #   module B
        #   end
        #
        # end
        #
        # gets the extraneous blankline deleted
        @render_queue_out.delete_at(@render_queue_out.length-2)
      end

      @render_queue_out << char

      i += 1
    end

    while @render_queue_out.last.is_a_newline?
      @render_queue_out.pop
    end

    @render_queue_out
  end

  def pluck_chars(n)
    @render_queue_out[-n..-1] || []
  end

  def is_end_blankline_end?(chars)
    return false if chars.length < 4
    chars[0].is_end? && chars[1].is_a_newline? && chars[2].is_a_newline? && chars[3].is_indent? && chars[4].is_end?
  end

  def is_end_and_not_end?(chars)
    return false if chars.length < 4
    chars[0].is_end? && chars[1].is_a_newline? && chars[2].is_indent? && !chars[3].is_end?
  end
end
