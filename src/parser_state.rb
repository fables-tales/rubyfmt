class BreakableState
  include TokenBase
  attr_reader :indentation_depth

  def initialize(indentation_depth)
    @indentation_depth = indentation_depth
  end

  def to_s
    ""
  end
end

class ParserState
  attr_accessor :depth_stack, :start_of_line, :line, :string_concat_position, :surpress_one_paren
  attr_reader :heredoc_strings
  attr_reader :result
  attr_reader :current_orig_line_number
  attr_accessor :render_queue
  attr_reader :comments_hash
  attr_reader :depth_stack
  def initialize(result, line_metadata)
    @surpress_comments_stack = [false]
    @surpress_one_paren = false
    @result = result
    @depth_stack = [0]
    @start_of_line = [true]
    @render_queue = []
    @current_orig_line_number = 0
    @comments_hash = line_metadata.comment_blocks
    @conditional_indent = [0]
    @heredoc_strings = []
    @string_concat_position = []
    @comments_to_insert = CommentBlock.new
    @breakable_state_stack = []
  end

  def self.with_depth_stack(output, from:)
    i = new(output, LineMetadata.new({}))
    i.depth_stack = from.depth_stack.dup
    i
  end

  def render_queue_as_lines
    render_queue.flatten.split { |x| x.is_a_newline? }.map { |x| TokenCollection.new(x) }
  end

  def tokens_from_previous_line
    render_queue_as_lines.last
  end

  def insert_comment_collection(cc)
    @comments_to_insert.merge(cc)
  end

  def breakable_of(start_delim, end_delim, &blk)
    emit_ident(start_delim)
    @breakable_state_stack << BreakableState.new(current_spaces)
    # we insert breakable state markers in to the render queue indicating
    # to the formatter where it can consider breaking constructs
    @render_queue << @breakable_state_stack.last
    emit_soft_newline
    new_block do
      blk.call
    end
    emit_soft_indent
    @render_queue << @breakable_state_stack.pop
    emit_ident(end_delim)
  end

  def breakable_entry(&blk)
    render_queue = @render_queue
    be = BreakableEntry.new
    @render_queue = be
    blk.call
    @render_queue = render_queue
    @render_queue << be
  end

  def with_surpress_comments(value, &blk)
    @surpress_comments_stack << value
    blk.call
    @surpress_comments_stack.pop
  end

  def push_heredoc_content(symbol, indent, inner_string_components)
    buf = StringIO.new
    next_ps = ParserState.new(buf, LineMetadata.new({}))
    next_ps.depth_stack = depth_stack.dup
    format_inner_string(next_ps, inner_string_components, :heredoc)

    next_ps.emit_newline
    next_ps.write
    buf.rewind

    buf_data = buf.read

    # buf gets an extra newline on the end, trim it
    @heredoc_strings << [symbol, indent, buf_data[0...-1]]
    next_ps.heredoc_strings.each do |s|
      @heredoc_strings << s
    end
  end

  def with_start_of_line(value, &blk)
    start_of_line << value
    blk.call
    start_of_line.pop
  end

  def start_string_concat
    push_conditional_indent(:string) if @string_concat_position.empty?
    @string_concat_position << Object.new
  end

  def end_string_concat
    @string_concat_position.pop
    pop_conditional_indent if @string_concat_position.empty?
  end

  def on_line(line_number, skip=false)
    if line_number != @current_orig_line_number
      @arrays_on_line = -1
    end

    build_comments = CommentBlock.new
    while !comments_hash.empty? && comments_hash.keys.sort.first <= line_number
      key = comments_hash.keys.sort.first
      comment = comments_hash.delete(key)
      build_comments.add_comment(Comment.new(comment))
    end
    insert_comment_collection(build_comments) if !@surpress_comments_stack.last && !build_comments.empty?

    @current_orig_line_number = line_number
  end

  def write
    on_line(100000000000000)

    fixup_render_queue

    @render_queue.each { |x|
      result.write(x)
    }

    result.write("\n")
    result.flush
  end

  def fixup_render_queue
    @render_queue = RenderQueueDFA.new(TokenCollection.new(@render_queue)).call
  end

  def emit_indent
    @render_queue << Indent.new(current_spaces)
  end

  def emit_soft_indent
    @render_queue << SoftIndent.new(current_spaces)
  end

  def current_spaces
    (@conditional_indent.last) + (2 * @depth_stack.last)
  end

  def emit_slash
    @render_queue << SingleSlash.new
  end

  def push_conditional_indent(type)
    if start_of_line.last
      @conditional_indent << 2*@depth_stack.last
    else
      if type == :conditional
        @conditional_indent << 2*@depth_stack.last
      elsif type == :string
        @conditional_indent << render_queue_as_lines.last.to_s.length
      end
    end

    @depth_stack << 0
  end

  def pop_conditional_indent
    @conditional_indent.pop
    @depth_stack.pop
  end

  def emit_comma_space
    @render_queue << CommaSpace.new
  end

  def emit_comma
    @render_queue << Comma.new
  end

  def ensure_file_ends_with_exactly_one_newline(lines)
    lines.each_with_index do |line, i|
      if i == lines.length-1
        line.strip_trailing_newlines
      end

      result.write(line)
    end
  end

  def clear_empty_trailing_lines
    while render_queue.last == []
      render_queue.pop
    end

    while render_queue.last.is_a_newline?
      render_queue.pop
    end
  end

  def emit_def_keyword
    @render_queue << Keyword.new(:def)
  end

  def emit_def(def_name)
    emit_def_keyword
    @render_queue << DirectPart.new(" #{def_name}")
  end

  def emit_end
    emit_newline
    emit_indent if start_of_line.last
    @render_queue << Keyword.new(:end)
  end

  def emit_keyword(keyword)
    @render_queue << Keyword.new(keyword)
  end

  def emit_do
    @render_queue << Keyword.new(:do)
  end

  def emit_rescue
    @render_queue << Keyword.new(:rescue)
  end

  def emit_module_keyword
    @render_queue << Keyword.new(:module)
  end

  def emit_class_keyword
    @render_queue << Keyword.new(:class)
  end

  def emit_while
    @render_queue << Keyword.new(:while)
  end

  def emit_for
    @render_queue << Keyword.new(:for)
  end

  def emit_in
    @render_queue << Keyword.new(:in)
  end

  def emit_else
    @render_queue << Keyword.new(:else)
  end

  def emit_elsif
    @render_queue << Keyword.new(:elsif)
  end

  def emit_return
    @render_queue << Keyword.new(:return)
  end

  def emit_ensure
    @render_queue << Keyword.new(:ensure)
  end

  def emit_when
    @render_queue << Keyword.new(:when)
  end

  def emit_stabby_lambda
    @render_queue << Keyword.new(:"->")
  end

  def emit_case
    @render_queue << Keyword.new(:case)
  end

  def emit_begin
    @render_queue << Keyword.new(:begin)
  end

  def emit_params_list(params_list)
  end

  def emit_binary(symbol)
    @render_queue << Binary.new(symbol)
  end

  def emit_space
    @render_queue << Space.new
  end

  def shift_comments
    idx_of_prev_hard_newline = @render_queue.rindex_by { |x| x.is_a_newline? }
    if !@comments_to_insert.empty?
      if idx_of_prev_hard_newline
        @render_queue.insert(idx_of_prev_hard_newline, @comments_to_insert.to_token_collection)
      else
        @render_queue.insert(0, @comments_to_insert.to_token_collection)
      end
      @comments_to_insert = CommentBlock.new
    end
  end

  def emit_newline
    shift_comments
    @render_queue << HardNewLine.new
    render_heredocs
  end

  def emit_soft_newline
    return emit_newline if have_heredocs?
    shift_comments
    @render_queue << SoftNewLine.new
  end

  def emit_dot
    @render_queue << Dot.new
  end

  def emit_lonely_operator
    @render_queue << LonelyOperator.new
  end

  def emit_ident(ident)
    @render_queue << DirectPart.new(ident)
  end

  def emit_op(op)
    @render_queue << Op.new(op)
  end

  def emit_int(int)
    @render_queue << DirectPart.new(int)
  end

  def emit_var_ref(ref)
    @render_queue << DirectPart.new(ref)
  end

  def emit_open_paren
    @render_queue << OpenParen.new
  end

  def emit_close_paren
    @render_queue << CloseParen.new
  end

  def emit_open_square_bracket
    @render_queue << OpenSquareBracket.new
  end

  def emit_close_square_bracket
    @render_queue << CloseSquareBracket.new
  end

  def new_block(&blk)
    depth_stack[-1] += 1
    with_start_of_line(true, &blk)
    depth_stack[-1] -= 1
  end

  def dedent(&blk)
    depth_stack[-1] -= 1
    with_start_of_line(true, &blk)
    depth_stack[-1] += 1
  end

  def emit_open_block_arg_list
    @render_queue << OpenArgPipe.new
  end

  def emit_close_block_arg_list
    @render_queue << CloseArgPipe.new
  end

  def emit_double_quote
    @render_queue << DoubleQuote.new
  end

  def emit_const(const)
    @render_queue << DirectPart.new(const)
  end

  def emit_double_colon
    @render_queue << Op.new("::")
  end

  def emit_symbol(symbol)
    @render_queue << DirectPart.new(":#{symbol}")
  end

  def have_heredocs?
    !heredoc_strings.empty?
  end

  def render_heredocs(skip=false)
    while have_heredocs?
      symbol, indent, string = heredoc_strings.pop
      unless render_queue[-1] && render_queue[-1].is_a_newline?
        @render_queue << HardNewLine.new
      end

      if string.end_with?("\n")
        string = string[0...-1]
      end

      if string.end_with?("\n")
        string = string[0...-1]
      end

      @render_queue << DirectPart.new(string)
      emit_newline
      if indent
        emit_indent
      end
      emit_ident(symbol.to_s.gsub("'", ""))
      if !skip
        emit_newline
      end
    end
  end
end
