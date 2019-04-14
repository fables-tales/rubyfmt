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
    @line = Line.new([])
    @current_orig_line_number = 0
    @comments_hash = line_metadata.comment_blocks
    @conditional_indent = [0]
    @heredoc_strings = []
    @string_concat_position = []
  end

  def self.with_depth_stack(output, from:)
    i = new(output, LineMetadata.new({}))
    i.depth_stack = from.depth_stack.dup
    i
  end

  def breakable_entry(&blk)
    @line.breakable_entry(&blk)
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

    # buf gets an extra newline on the end, trim it
    @heredoc_strings << [symbol, indent, buf.read[0...-1]]
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

    while !comments_hash.empty? && comments_hash.keys.sort.first < line_number
      key = comments_hash.keys.sort.first
      comment = comments_hash.delete(key)
      @line.push_comment(comment) unless @surpress_comments_stack.last
    end

    @current_orig_line_number = line_number
  end

  def write
    on_line(100000000000000)
    clear_empty_trailing_lines

    lines = render_queue
    clear_double_spaces(lines)
    add_valid_blanklines(lines)

    lines.each do |line|
      line.remove_redundant_indents
    end

    ensure_file_ends_with_exactly_one_newline(lines)

    result.write("\n")
    result.flush
  end

  def emit_indent
    spaces = (@conditional_indent.last) + (2 * @depth_stack.last)
    line << Indent.new(spaces)
  end

  def emit_slash
    line << SingleSlash.new
  end

  def push_conditional_indent(type)
    if line.empty?
      @conditional_indent << 2*@depth_stack.last
    else
      if type == :conditional
        @conditional_indent << 2*@depth_stack.last
      elsif type == :string
        @conditional_indent << line.string_length
      end
    end

    @depth_stack << 0
  end

  def pop_conditional_indent
    @conditional_indent.pop
    @depth_stack.pop
  end

  def emit_comma_space
    line << CommaSpace.new
  end

  def emit_comma
    line << Comma.new
  end

  def ensure_file_ends_with_exactly_one_newline(lines)
    lines.each_with_index do |line, i|
      if i == lines.length-1
        line.strip_trailing_newlines
      end

      result.write(line)
    end
  end

  def add_valid_blanklines(lines)
    line = lines.first
    next_index = 1
    while next_index < lines.length
      if want_blankline?(line, lines[next_index])
        lines.insert(next_index, Line.new(["\n"]))
        next_index += 1
      end

      line = lines[next_index]
      next_index += 1
    end
  end

  def clear_double_spaces(lines)
    line = lines.first
    next_index = 1
    while next_index < lines.length
      while line.ends_with_newline? && lines[next_index] && lines[next_index].is_only_a_newline?
        lines.delete_at(next_index)
      end

      line = lines[next_index]
      next_index += 1
    end
  end

  def clear_empty_trailing_lines
    while render_queue.last == []
      render_queue.pop
    end

    while render_queue.last.is_only_a_newline?
      render_queue.pop
    end
  end

  def emit_def(def_name)
    line << Keyword.new(:def)
    line << " #{def_name}"
  end

  def emit_end
    emit_newline
    emit_indent if start_of_line.last
    line << Keyword.new(:end)
  end

  def emit_do
    line << Keyword.new(:do)
  end

  def emit_rescue
    line << Keyword.new(:rescue)
  end

  def emit_module_keyword
    line << Keyword.new(:module)
  end

  def emit_class_keyword
    line << Keyword.new(:class)
  end

  def emit_while
    line << Keyword.new(:while)
  end

  def emit_for
    line << Keyword.new(:for)
  end

  def emit_in
    line << Keyword.new(:in)
  end

  def emit_else
    line << Keyword.new(:else)
  end

  def emit_elsif
    line << Keyword.new(:elsif)
  end

  def emit_return
    line << Keyword.new(:return)
  end

  def emit_ensure
    line << Keyword.new(:ensure)
  end

  def emit_when
    line << Keyword.new(:when)
  end

  def emit_stabby_lambda
    line << Keyword.new(:"->")
  end

  def emit_case
    line << Keyword.new(:case)
  end

  def emit_begin
    line << Keyword.new(:begin)
  end

  def emit_params_list(params_list)
  end

  def emit_binary(symbol)
    line << Binary.new(symbol)
  end

  def emit_space
    line << Space.new
  end

  def emit_newline
    line << HardNewLine.new
    render_queue << line
    self.line = Line.new([])
    render_heredocs
  end

  def emit_dot
    line << Dot.new
  end

  def emit_lonely_operator
    line << LonelyOperator.new
  end

  def emit_ident(ident)
    line << ident
  end

  def emit_op(op)
    line << op
  end

  def emit_int(int)
    line << int
  end

  def emit_var_ref(ref)
    line << ref
  end

  def emit_open_paren
    line << OpenParen.new
  end

  def emit_close_paren
    line << CloseParen.new
  end

  def emit_open_square_bracket
    line << OpenSquareBracket.new
  end

  def emit_close_square_bracket
    line << CloseSquareBracket.ne
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
    line << "|"
  end

  def emit_close_block_arg_list
    line << "|"
  end

  def emit_double_quote
    line << "\""
  end

  def emit_const(const)
    line << const
  end

  def emit_double_colon
    line << "::"
  end

  def emit_symbol(symbol)
    line << ":#{symbol}"
  end

  def render_heredocs(skip=false)
    while !heredoc_strings.empty?
      symbol, indent, string = heredoc_strings.pop
      unless render_queue[-1] && render_queue[-1].ends_with_newline?
        line << "\n"
      end

      if string.end_with?("\n")
        string = string[0...-1]
      end

      line << string
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
