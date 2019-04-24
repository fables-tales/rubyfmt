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
    @render_queue = TokenCollection.new([])
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

  def render_queue_as_lines
    render_queue.flatten.split { |x| HardNewLine === x }.map { |x| TokenCollection.new(x) }
  end

  def tokens_from_previous_line
    render_queue_as_lines.last
  end

  def insert_comment_collection(cc)
    if HardNewLine === cc.first && tokens_from_previous_line.declares_class_or_module?
      cc.pop
    end
    @render_queue << cc
  end

  def push_token(token)
    @render_queue << token
  end

  def breakable_entry(&blk)
    blk.call
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

    build_comments = []
    while !comments_hash.empty? && comments_hash.keys.sort.first < line_number
      key = comments_hash.keys.sort.first
      comment = comments_hash.delete(key)
      build_comments << Comment.new(comment)
      build_comments << HardNewLine.new
    end
    if !build_comments.empty?
      build_comments.insert(0, HardNewLine.new)
    end
    insert_comment_collection(TokenCollection.new(build_comments)) if !@surpress_comments_stack.last && !build_comments.empty?

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
    @render_queue = RenderQueueDFA.new(@render_queue).call
  end

  def emit_indent
    spaces = (@conditional_indent.last) + (2 * @depth_stack.last)
    push_token(Indent.new(spaces))
  end

  def emit_slash
    push_token(SingleSlash.new)
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
    push_token(CommaSpace.new)
  end

  def emit_comma
    push_token(Comma.new)
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

    while HardNewLine === render_queue.last
      render_queue.pop
    end
  end

  def emit_def(def_name)
    push_token(Keyword.new(:def))
    push_token(DirectPart.new(" #{def_name}"))
  end

  def emit_end
    emit_newline
    emit_indent if start_of_line.last
    push_token(Keyword.new(:end))
  end

  def emit_keyword(keyword)
    push_token(Keyword.new(keyword))
  end

  def emit_do
    push_token(Keyword.new(:do))
  end

  def emit_rescue
    push_token(Keyword.new(:rescue))
  end

  def emit_module_keyword
    push_token(Keyword.new(:module))
  end

  def emit_class_keyword
    push_token(Keyword.new(:class))
  end

  def emit_while
    push_token(Keyword.new(:while))
  end

  def emit_for
    push_token(Keyword.new(:for))
  end

  def emit_in
    push_token(Keyword.new(:in))
  end

  def emit_else
    push_token(Keyword.new(:else))
  end

  def emit_elsif
    push_token(Keyword.new(:elsif))
  end

  def emit_return
    push_token(Keyword.new(:return))
  end

  def emit_ensure
    push_token(Keyword.new(:ensure))
  end

  def emit_when
    push_token(Keyword.new(:when))
  end

  def emit_stabby_lambda
    push_token(Keyword.new(:"->"))
  end

  def emit_case
    push_token(Keyword.new(:case))
  end

  def emit_begin
    push_token(Keyword.new(:begin))
  end

  def emit_params_list(params_list)
  end

  def emit_binary(symbol)
    push_token(Binary.new(symbol))
  end

  def emit_space
    push_token(Space.new)
  end

  def emit_newline
    push_token(HardNewLine.new)
    render_heredocs
  end

  def emit_dot
    push_token(Dot.new)
  end

  def emit_lonely_operator
    push_token(LonelyOperator.new)
  end

  def emit_ident(ident)
    push_token(DirectPart.new(ident))
  end

  def emit_op(op)
    push_token(Op.new(op))
  end

  def emit_int(int)
    push_token(DirectPart.new(int))
  end

  def emit_var_ref(ref)
    push_token(DirectPart.new(ref))
  end

  def emit_open_paren
    push_token(OpenParen.new)
  end

  def emit_close_paren
    push_token(CloseParen.new)
  end

  def emit_open_square_bracket
    push_token(OpenSquareBracket.new)
  end

  def emit_close_square_bracket
    push_token(CloseSquareBracket.new)
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
    push_token(OpenArgPipe.new)
  end

  def emit_close_block_arg_list
    push_token(CloseArgPipe.new)
  end

  def emit_double_quote
    push_token(DoubleQuote.new)
  end

  def emit_const(const)
    push_token(DirectPart.new(const))
  end

  def emit_double_colon
    push_token(Op.new("::"))
  end

  def emit_symbol(symbol)
    push_token(DirectPart.new(":#{symbol}"))
  end

  def render_heredocs(skip=false)
    while !heredoc_strings.empty?
      symbol, indent, string = heredoc_strings.pop
      unless render_queue[-1] && HardNewLine === render_queue[-1]
        push_token(HardNewLine.new)
      end

      if string.end_with?("\n")
        string = string[0...-1]
      end

      if string.end_with?("\n")
        string = string[0...-1]
      end

      push_token(DirectPart.new(string))
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
