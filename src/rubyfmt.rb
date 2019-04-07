#!/usr/bin/env ruby
require "ripper"
require "stringio"
require "pp"
LineMetadata = Struct.new(:comment_blocks)

class Line
  attr_accessor :parts
  attr_accessor :manual_blankline
  def initialize(parts)
    @comments = []
    @parts = parts
    @manual_blankline = false
  end

  def push_comment(comment)
    @comments << comment
  end

  def has_comment?
    !@comments.empty?
  end

  def <<(item)
    @parts << item
  end

  def string_length
    @parts.join("").length
  end

  def empty?
    @parts.empty?
  end

  def to_s
    build = @parts.join("")

    unless @comments.empty?
      build = "#{@comments.join("\n")}\n#{build}"
    end

    build
  end

  def strip_trailing_newlines
    while @parts.last == "\n"
      @parts.pop
    end
  end

  def remove_redundant_indents
    @parts.shift if @parts[0] == ""
  end

  def ends_with_newline?
    @parts.last == "\n"
  end

  def is_only_a_newline?
    @parts == ["\n"]
  end

  def contains_end?
    @parts.any? { |x| x == :end }
  end

  def contains_rescue?
    @parts.any? { |x| x == :rescue }
  end

  def contains_def?
    @parts.any? { |x| x == :def }
  end

  def contains_do?
    @parts.any? { |x| x == :do }
  end

  def contains_if?
    @parts.any? { |x| x == :if }
  end

  def contains_else?
    @parts.any? { |x| x == :else }
  end

  def contains_unless?
    @parts.any? { |x| x == :unless }
  end

  def declares_private?
    @parts.any? { |x| x == "private" } && @parts.length == 3
  end

  def declares_require?
    @parts.any? { |x| x == "require" } && @parts.none? { |x| x == "}" }
  end

  def declares_class_or_module?
    @parts.any? { |x| x == :class || x == :module }
  end

  def contains_while?
    @parts.any? { |x| x == :while }
  end

  def contains_keyword?
    [
      :def,
      :do,
      :while,
      :if,
      :else,
      :elsif,
      :rescue,
      :begin,
      :finally,
      :ensure,
      :case,
      :when,
      :"->",
    ].any? { |kw| @parts.include?(kw) }
  end

  def surpresses_blankline?
    contains_keyword? || declares_class_or_module? || ends_with_parenish_construct?
  end

  def ends_with_parenish_construct?
    ["(", "{", "["].any? { |item| @parts[-2].to_s.end_with?(item) }
  end
end

def want_blankline?(line, next_line)
  if ENV["RUBYFMT_DEBUG"] == "3"
    require 'pry'; binding.pry
  end
  return unless next_line
  return false if line.surpresses_blankline?
  return true if line.manual_blankline
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do?
  return true if (next_line.contains_if? || next_line.contains_unless?)
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
  return true if !line.declares_class_or_module? && next_line.has_comment?
  return true if !line.declares_class_or_module? && next_line.declares_class_or_module?
end

class ParserState
  attr_accessor :depth_stack, :start_of_line, :line, :string_concat_position, :surpress_one_paren
  attr_reader :heredoc_strings
  attr_reader :result
  attr_reader :current_orig_line_number
  attr_accessor :render_queue
  attr_reader :comments_hash
  attr_reader :depth_stack
  attr_accessor :formatting_class_or_module_stack
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
    @formatting_class_or_module_stack = [false]
  end

  def self.block_will_render_as_multiline?(ps, &blk)
    output = StringIO.new

    next_ps = ParserState.with_depth_stack(output, from: ps)
    blk.call(next_ps)
    next_ps.write
    output.rewind
    output.read.strip.split("\n").length > 1
  end

  def self.with_depth_stack(output, from:)
    i = new(output, LineMetadata.new({}))
    i.depth_stack = from.depth_stack.dup
    i
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

  def blankline_please
    (@render_queue.last || line).manual_blankline = true
  end

  def on_line(line_number, skip=false)
    if ENV["RUBYFMT_DEBUG"] == "2"
      p [line_number, @current_orig_line_number]
    end
    if line_number > @current_orig_line_number + 1
      if ENV["RUBYFMT_DEBUG"] == 2
        require 'pry'; binding.pry
      end
      blankline_please
    end

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

  def emit_while
    line << :while
  end

  def emit_for
    line << :for
  end

  def emit_in
    line << :in
  end

  def emit_indent
    spaces = (@conditional_indent.last) + (2 * @depth_stack.last)
    line << " " * spaces
  end

  def emit_slash
    line << "\\"
  end

  def emit_else
    line << :else
  end

  def emit_elsif
    line << "elsif"
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
    line << ", "
  end

  def emit_return
    line << :return
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

    while render_queue.last == ["\n"]
      render_queue.pop
    end
  end

  def emit_def(def_name)
    line << :def
    line << " #{def_name}"
  end

  def emit_params_list(params_list)
  end

  def emit_binary(symbol)
    line << " #{symbol} "
  end

  def emit_end
    emit_newline
    emit_indent if start_of_line.last
    line << :end
  end

  def emit_space
    line << " "
  end

  def emit_do
    line << :do
  end

  def emit_newline
    line << "\n"
    render_queue << line
    self.line = Line.new([])
    render_heredocs
  end

  def emit_dot
    line << "."
  end

  def emit_lonely_operator
    line << "&."
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
    line << "("
  end

  def emit_close_paren
    line << ")"
  end

  def with_formatting_class_or_module(formatting_class_or_module, &blk)
    formatting_class_or_module_stack << formatting_class_or_module
    blk.call
    formatting_class_or_module_stack.pop
  end

  def new_block(formatting_class_or_module: false, &blk)
    depth_stack[-1] += 1
    with_formatting_class_or_module(formatting_class_or_module) do
      with_start_of_line(true, &blk)
    end
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

  def emit_module_keyword
    line << :module
  end

  def emit_class_keyword
    line << :class
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

  def emit_rescue
    line << :rescue
  end

  def emit_stabby_lambda
    line << :"->"
  end

  def emit_case
    line << :case
  end

  def emit_when
    line << :when
  end

  def emit_ensure
    line << :ensure
  end

  def emit_begin
    line << :begin
  end

  def render_heredocs(skip=false)
    while !heredoc_strings.empty?
      line.manual_blankline = false
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

def format_block_params_list(ps, params_list)
  ps.emit_open_block_arg_list
  ps.emit_params_list(params_list)
  ps.emit_close_block_arg_list
end

def format_until(ps, rest)
  conditional, expressions = rest


  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("until")
  ps.emit_space
  ps.with_start_of_line(false) do
    format_expression(ps, conditional)
  end

  ps.emit_newline

  (expressions || []).each do |expr|
    ps.new_block do
      format_expression(ps, expr)
    end
  end

  ps.emit_end
  ps.emit_newline
end

def format_def(ps, rest)
  ps.blankline_please
  def_expression, params, body = rest

  def_name = def_expression[1]

  line_number = def_expression.last.first
  ps.on_line(line_number)

  params = rest[1]
  body = rest[2]

  ps.blankline_please
  ps.emit_indent
  ps.emit_def(def_name)

  ps.with_formatting_class_or_module(false) do
    format_params(ps, params, "(", ")")
  end

  ps.emit_newline
  ps.new_block do
    format_expression(ps, body)
  end

  ps.emit_end
  ps.emit_newline
  ps.emit_newline
end

def format_required_params(ps, required_params)
  return if required_params.empty?

  ps.with_start_of_line(false) do
    required_params.each_with_index do |expr, index|
      format_expression(ps, expr)
      if index != required_params.length - 1
        ps.emit_comma_space
      end
    end
  end
end

def format_optional_params(ps, optional_params)
  ps.with_start_of_line(false) do
    optional_params.each_with_index do |param, i|
      left,right = param
      format_expression(ps, left)
      ps.emit_ident("=")
      format_expression(ps, right)
      if i != optional_params.length - 1
        ps.emit_ident(", ")
      end
    end
  end
end

def format_kwargs(ps, kwargs)
  return if kwargs.empty?

  kwargs.each_with_index do |kwarg, index|
    label, false_or_expr = kwarg
    raise "got non label in kwarg" if label[0] != :@label

    ps.emit_ident(label[1])
    ps.with_start_of_line(false) do
      if false_or_expr
        ps.emit_space
      end
      format_expression(ps, false_or_expr) if false_or_expr
    end

    ps.emit_ident(", ") if index != kwargs.length-1
  end
end

def format_rest_params(ps, rest_params)
  return if rest_params == 0 || rest_params.empty? || rest_params == [:excessed_comma]
  ps.emit_ident("*")
  return if rest_params[1].nil?

  rest_param, expr = rest_params
  raise "got bad rest_params" if rest_param != :rest_param

  ps.with_start_of_line(false) do
    format_expression(ps, expr)
  end
end

def format_kwrest_params(ps, kwrest_params)
  return if kwrest_params.empty?

  ps.emit_ident("**")
  return if kwrest_params[1].nil?

  if kwrest_params[0] == :kwrest_param
    _, expr = kwrest_params
  else
    expr = kwrest_params
  end

  ps.with_start_of_line(false) do
    format_expression(ps, expr)
  end
end

def format_blockarg(ps, blockarg)
  return if blockarg.empty?
  _, expr = blockarg
  ps.with_start_of_line(false) do
    ps.emit_ident("&")
    format_expression(ps, expr)
  end
end

def format_params(ps, params, open_delim, close_delim)
  return if params.nil?
  f_params = []
  if params[0] == :paren || params[0] == :block_var
    if params[0] == :block_var && params[-1] != nil
      f_params = params[-1]
    end

    params = params[1]
  end

  have_any_params = params[1..-1].any? { |x| !x.nil? } || !f_params.empty?

  if have_any_params
    ps.emit_ident(open_delim)
  end

  bad_params = params[7..-1].any? { |x| !x.nil? }
  bad_params = false if params[5]
  bad_params = false if params[7]

  raise "dont know how to deal with a params list" if bad_params
  required_params = params[1] || []
  optional_params = params[2] || []
  rest_params = params[3] || []
  more_required_params = params[4] || []
  kwargs = params[5] || []

  kwrest_params = params[6] || []
  # on ruby 2.3 this position contains literally the integer 183 if a `**` is
  # given in the splatted kwargs position. Why, I have no idea.
  if kwrest_params == 183
    kwrest_params = [""]
  end

  block_arg = params[7] || []

  emission_order = [
    [required_params, method(:format_required_params)],
    [optional_params, method(:format_optional_params)],
    [rest_params, method(:format_rest_params)],
    [more_required_params, method(:format_required_params)],
    [kwargs, method(:format_kwargs)],
    [kwrest_params, method(:format_kwrest_params)],
    [block_arg, method(:format_blockarg)],
  ]

  did_emit = false
  have_more = false
  emission_order.each_with_index do |(values, callable), idx|
    if values == 0
      values = []
    end
    callable.call(ps, values)
    did_emit = !values.empty?
    have_more = emission_order[idx+1..-1].map { |x| x[0] != 0 && !x[0].empty? && x[0] != [:excessed_comma] }.any?
    ps.emit_ident(", ") if did_emit && have_more && idx != emission_order.length - 1
  end

  if f_params && !f_params.empty?
    ps.emit_ident(" ;")
    format_list_like_thing_items(ps, [f_params], true)
  end

  if have_any_params
    ps.emit_ident(close_delim)
  end
end

def format_void_expression(ps, rest)
end

def format_opassign(ps, rest)
  head, op, tail = rest

  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, head)
    ps.emit_space
    ps.emit_op(op[1])
    ps.emit_space

    format_expression(ps, tail)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_assign_expression(ps, rest)
  head, tail = rest
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, head)
    ps.emit_space
    ps.emit_op("=")
    ps.emit_space

    ps.with_formatting_class_or_module(false) do
      format_expression(ps, tail)
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_method_add_block(ps, rest)
  raise "got non 2 length rest in add block" if rest.count != 2
  left, block_body = rest
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, left)
  end
  ps.emit_space

  format_expression(ps, block_body)

  ps.emit_newline if ps.start_of_line.last
end

def format_int(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.on_line(rest[1][0])
  int = rest[0]
  ps.emit_int(int)

  ps.emit_newline if ps.start_of_line.last
end

def format_rational(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.on_line(rest[1][0])
  ps.emit_ident(rest[0])

  ps.emit_newline if ps.start_of_line.last
end

def format_imaginary(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.on_line(rest[1][0])
  ps.emit_ident(rest[0])

  ps.emit_newline if ps.start_of_line.last
end

def format_var_ref(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ref = rest[0][1]
  line_number = rest[0][2][0]
  ps.on_line(line_number)
  ps.emit_var_ref(ref)
  ps.emit_newline if ps.start_of_line.last
end

def format_binary(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, rest[0])
    ps.emit_binary("#{rest[1].to_s}")
    format_expression(ps, rest[2])
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_do_block(ps, rest)
  raise "got bad block #{rest.inspect}" if rest.length != 2
  params, body = rest

  ps.emit_do

  format_params(ps, params, " |", "|")

  ps.emit_newline

  ps.new_block do
    # in ruby 2.5 blocks are bodystmts because blocks support
    # ```
    # foo do
    # rescue
    # end
    # ```
    #
    # style rescues now
    if body[0] == :bodystmt
      format_expression(ps, body)
    else
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
  end

  ps.with_start_of_line(true) do
    ps.emit_end
  end
end

def format_tstring_content(ps, rest)
  ps.emit_ident(rest[1])
  ps.on_line(rest[2][0])
end

def format_inner_string(ps, parts, type)
  parts = parts.dup

  parts.each_with_index do |part, idx|
    case part[0]
    when :@tstring_content
      ps.emit_ident(part[1])
      ps.on_line(part[2][0])
    when :string_embexpr
      ps.emit_ident("\#{")
      ps.with_start_of_line(false) do
        format_expression(ps, part[1][0])
      end
      ps.emit_ident("}")
      on_line_skip = type == :heredoc && parts[idx+1] && parts[idx+1][0] == :@tstring_content && parts[idx+1][1].start_with?("\n")
      if on_line_skip
        ps.render_heredocs(true)
      end
    when :string_dvar
      ps.emit_ident("\#{")
      ps.with_start_of_line(false) do
        format_expression(ps, part[1])
      end
      ps.emit_ident("}")
    else
      raise "dont't know how to do this #{part[0].inspect}"
    end
  end
end

def format_heredoc_string_literal(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.with_surpress_comments(true) do
    heredoc_type = rest[0][1][0]
    heredoc_symbol = rest[0][1][1]

    ps.emit_ident(heredoc_type)
    ps.emit_ident(heredoc_symbol)

    string_parts = rest[1]
    #the 1 that we drop here is the literal symbol :string_content
    inner_string_components = string_parts.drop(1)
    components = inner_string_components
    ps.push_heredoc_content(heredoc_symbol, heredoc_type.include?("~"), components)
  end
end

def format_string_literal(ps, rest)
  return format_heredoc_string_literal(ps, rest) if rest[0][0] == :heredoc_string_literal
  items = rest[0]
  parts = nil
  if items[0] == :string_content
    _, parts = items[0], items[1..-1]
  else
    parts = items
  end
  ps.emit_indent if ps.start_of_line.last
  ps.emit_double_quote

  format_inner_string(ps, parts, :quoted)

  ps.emit_double_quote
  ps.emit_newline if ps.start_of_line.last && ps.string_concat_position.empty?
end

def format_character_literal(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_double_quote

  ps.emit_ident(rest[0][1..-1])

  ps.emit_double_quote
  ps.emit_newline if ps.start_of_line.last && ps.string_concat_position.empty?
end

def format_xstring_literal(ps, rest)
  items = rest[0]
  parts = nil
  if items[0] == :string_content
    _, parts = items[0], items[1..-1]
  else
    parts = items
  end
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("`")

  format_inner_string(ps, parts, :quoted)

  ps.emit_ident("`")
  ps.emit_newline if ps.start_of_line.last && ps.string_concat_position.empty?
end

def format_module(ps, rest)
  module_name = rest[0]

  ps.emit_indent
  ps.emit_module_keyword

  ps.start_of_line << false

  ps.emit_space
  format_expression(ps, module_name)

  ps.start_of_line.pop
  ps.emit_newline


  ps.new_block(formatting_class_or_module: true) do
    exprs = rest[1][1]
    exprs.each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def format_class(ps, rest)
  class_name = rest[0]

  ps.emit_indent
  ps.emit_class_keyword
  ps.with_start_of_line(false) do
    ps.emit_space
    format_expression(ps, class_name)
  end

  if rest[1] != nil
    ps.emit_ident(" < ")
    ps.start_of_line << false
    format_expression(ps, rest[1])
    ps.start_of_line.pop
  end

  ps.emit_newline

  ps.new_block(formatting_class_or_module: true) do
    exprs = rest[2][1]
    exprs.each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def format_const_path_ref(ps, rest)
  expr, const = rest

  ps.start_of_line << false
  format_expression(ps, expr)
  ps.emit_double_colon
  raise "cont a non const" if const[0] != :"@const"
  ps.emit_const(const[1])
  ps.start_of_line.pop
  if ps.start_of_line.last
    ps.emit_newline
  end
end

def format_const_path_field(ps, rest)
  format_expression(ps, rest[0])
  rest[1..-1].each do |expr|
    ps.emit_ident("::")
    format_expression(ps, expr)
  end
end

def format_top_const_field(ps, rest)
  rest.each do |expr|
    ps.emit_ident("::")
    format_expression(ps, expr)
  end
end

def format_dot(ps, rest)
  dot = rest[0]

  case
  when is_normal_dot(dot)
    ps.emit_dot
  when dot == :"::"
    ps.emit_ident("::")
  when is_lonely_operator(dot)
    ps.emit_lonely_operator
  else
    raise "got unrecognised dot"
  end
end

def format_ident(ps, ident)
  ps.on_line(ident[1][0])
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(ident[0])
end

def format_symbol_literal(ps, literal)
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, literal[0])
  end

  ps.emit_newline if ps.start_of_line.last
end

def is_normal_dot(candidate)
  candidate == :"." || (candidate.is_a?(Array) && candidate[0] == :@period)
end

def is_lonely_operator(candidate)
  candidate == :"&." || [
    candidate.is_a?(Array),
    candidate[0] == :@op,
    candidate[1] == "&.",
  ].all?
end

def format_list_like_thing_items(ps, args_list, single_line)
  return false if args_list.nil?
  emitted_args = false
  args_list[0].each_with_index do |expr, idx|
    raise "this is bad" if expr[0] == :tstring_content
    if !single_line
      ps.emit_indent
      ps.start_of_line << false
    end
    format_expression(ps, expr)

    if single_line
      ps.emit_comma_space unless idx == args_list[0].count-1
    else
      ps.emit_ident(",")
      ps.emit_newline
      ps.start_of_line.pop
    end

    emitted_args = true
  end

  emitted_args
end

# format_list_like_thing takes any inner construct (like array items, or an
# args list, and formats them).
def format_list_like_thing(ps, args_list, single_line=true)
  emitted_args = false
  return false if args_list.nil? || args_list[0].nil?
  if args_list[0][0] != :args_add_star
    emitted_args = format_list_like_thing_items(ps, args_list, single_line)
  else
    _args_add_star, args_list, *calls = args_list[0]
    raise "this is impossible" unless _args_add_star == :args_add_star
    args_list = [args_list]
    emitted_args = format_list_like_thing(ps, args_list, single_line)

    if single_line
      # if we're single line, our predecessor didn't emit a trailing comma
      # space because rubyfmt terminates single line arg lists without the
      # trailer so emit one here
      ps.emit_comma_space if emitted_args
    else
      # similarly if we're multi line, we emit a newline but not an indent
      # at the end our formatting spree, because we might be at a terminator
      # so fix up the indent
      ps.emit_indent
    end

    emitted_args = true
    ps.with_start_of_line(false) do
      ps.emit_ident("*")
      first_call = calls.shift
      format_expression(ps, first_call)

      calls.each do |call|
        emit_intermediate_array_separator(ps, single_line)
        format_expression(ps, call)
      end

      # if we are not single line, we need to emit a comma newline, to be a
      # good citizen
      if !single_line
        ps.emit_ident(",")
        ps.emit_newline
      end
    end
  end

  emitted_args
end

def emit_intermediate_array_separator(ps, single_line)
  if single_line
    ps.emit_comma_space
  else
    ps.emit_ident(",")
    ps.emit_newline
    ps.emit_indent
  end
end

def emit_extra_separator(ps, single_line, emitted_args)
  return unless emitted_args

  if single_line
    ps.emit_comma_space
  else
    ps.emit_indent
  end
end

def format_args_add_block(ps, args_list)
  surpress_paren = ps.surpress_one_paren
  ps.surpress_one_paren = false
  ps.emit_open_paren unless surpress_paren
  ps.start_of_line << false

  emitted_args = format_list_like_thing(ps, args_list)

  if args_list[1]
    ps.emit_ident(", ") if emitted_args
    ps.emit_ident("&")
    format_expression(ps, args_list[1])
  end

  ps.emit_close_paren unless surpress_paren

  ps.start_of_line.pop
end

def format_const_ref(ps, expression)
  raise "got more tahn one thing in const ref" if expression.length != 1
  format_expression(ps, expression[0])
end

def format_const(ps, expression)
  line_number = expression.last.first
  ps.on_line(line_number)
  raise "didn't get exactly a const" if expression.length != 2
  ps.emit_indent if ps.start_of_line.last
  ps.emit_const(expression[0])
end

def format_defs(ps, rest)
  head, period, tail, params, body = rest
  ps.emit_indent if ps.start_of_line.last
  ps.blankline_please
  ps.emit_ident("def")
  ps.emit_space
  ps.start_of_line << false
  format_expression(ps, head)
  ps.emit_dot
  format_expression(ps, tail)

  format_params(ps, params, "(", ")")
  ps.emit_newline
  ps.start_of_line.pop
  ps.new_block do
    format_expression(ps, body)
  end

  ps.emit_end

  ps.emit_newline if ps.start_of_line.last
end

def format_kw(ps, rest)
  ps.emit_ident(rest[0])
  ps.on_line(rest.last.first)
end

def format_rescue(ps, rescue_part)
  return if rescue_part.nil?
  _, rescue_class, rescue_capture, rescue_expressions, next_rescue = rescue_part
  ps.dedent do
    ps.emit_indent
    ps.emit_rescue
    ps.with_start_of_line(false) do
      if !rescue_class.nil? || !rescue_capture.nil?
        ps.emit_space
      end

      if !rescue_class.nil?
        if rescue_class.count == 1
          rescue_class = rescue_class[0]
        end
        format_expression(ps, rescue_class)
      end

      if !rescue_class.nil? && !rescue_capture.nil?
        ps.emit_space
      end

      if !rescue_capture.nil?
        ps.emit_ident("=> ")
        format_expression(ps, rescue_capture)
      end
    end
  end

  if !rescue_expressions.nil?
    ps.emit_newline
    ps.with_start_of_line(true) do
      rescue_expressions.each do |expr|
        format_expression(ps, expr)
      end
    end
  end

  format_rescue(ps, next_rescue) unless next_rescue.nil?
end

def format_ensure(ps, ensure_part)
  return if ensure_part.nil?

  _, ensure_expressions = ensure_part
  ps.dedent do
    ps.emit_indent
    ps.emit_ensure
  end

  if !ensure_expressions.nil?
    ps.emit_newline
    ps.with_start_of_line(true) do
      ensure_expressions.each do |expr|
        format_expression(ps, expr)
      end
    end
  end
end

def format_else(ps, else_part)
  return if else_part.nil?

  exprs = if RUBY_VERSION.to_f > 2.5
    else_part
  else
    _, a = else_part
    a
  end

  ps.dedent do
    ps.emit_indent
    ps.emit_else
  end

  if !exprs.nil?
    ps.emit_newline
    ps.with_start_of_line(true) do
      exprs.each do |expr|
        format_expression(ps, expr)
      end
    end
  end
end

def format_bodystmt(ps, rest, inside_begin=false)
  expressions = rest[0]
  rescue_part = rest[1]
  else_part = rest[2]
  ensure_part = rest[3]
  if rest[4..-1].any? {|x| x != nil }
    raise "got something other than a nil in a format body statement"
  end

  expressions.each do |line|
    format_expression(ps, line)
  end

  format_rescue(ps, rescue_part)
  format_else(ps, else_part)
  format_ensure(ps, ensure_part)

end

def format_if_mod(ps, rest)
  format_conditional_mod(ps, rest, "if")
end

def format_unless_mod(ps, rest)
  format_conditional_mod(ps, rest, "unless")
end

def format_conditional_mod(ps, rest, conditional_type)
  conditional, guarded_expression = rest
  ps.emit_indent if ps.start_of_line.last
  ps.start_of_line << false

  format_expression(ps, guarded_expression)
  ps.emit_space
  ps.emit_ident(conditional_type)
  ps.emit_space
  format_expression(ps, conditional)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_conditional_parts(ps, further_conditionals)
  return if further_conditionals.nil?
  type = further_conditionals[0]

  case type
  when :else
    _, body = further_conditionals
    ps.emit_indent
    ps.emit_else


    ps.emit_newline
    ps.with_start_of_line(true) do
      ps.new_block do
        body.each do |expr|
          format_expression(ps, expr)
        end
      end
    end
  when :elsif
    _, cond, body, further_conditionals = further_conditionals

    ps.emit_indent
    ps.emit_elsif
    ps.emit_space

    ps.start_of_line << false
    format_expression(ps, cond)
    ps.start_of_line.pop

    ps.emit_newline
    ps.start_of_line << true

    ps.new_block do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
    ps.start_of_line.pop
    ps.emit_newline

    format_conditional_parts(ps, further_conditionals)
  when nil

  else
    raise "didn't get a known type in format conditional parts"
  end
end

def format_unless(ps, expression)
  format_conditional(ps, expression, :unless)
end

def format_if(ps, expression)
  format_conditional(ps, expression, :if)
end

def format_conditional(ps, expression, kind)
  if_conditional, body, further_conditionals = expression[0], expression[1], expression[2]

  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    ps.emit_ident(kind)
    ps.emit_space
    format_expression(ps, if_conditional)
  end

  ps.emit_newline
  ps.new_block do
    body.each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.with_start_of_line(true) do
    ps.emit_newline
    format_conditional_parts(ps, further_conditionals || [])

    ps.emit_end
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_var_field(ps, rest)
  raise "didn't get exactly one thing" if rest.length != 1
  format_expression(ps, rest[0])
end

def format_ivar(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0])
end

def format_top_const_ref(ps, rest)
  raise "got bad top const ref" if rest.length != 1
  ps.emit_indent if ps.start_of_line.last
  ps.emit_double_colon
  ps.emit_ident(rest[0][1])
end

def format_inner_args_list(ps, args_list)
  case args_list[0]
  when :args_add_star
    format_list_like_thing(ps, [args_list], single_line=true)
  when Symbol
    format_expression(ps, args_list) unless args_list.empty?
  else
    format_list_like_thing(ps, [args_list], single_line=true)
  end
end

def format_array_fast_path(ps, rest)
  single_statement = rest[0] && rest[0].length == 1
  if single_statement
    ps.emit_ident("[")
    ps.with_start_of_line(false) do
      format_list_like_thing(ps, rest, true)
    end
    ps.emit_ident("]")
  else
    line_number = extract_line_number_from_construct(rest)
    if line_number != nil
      ps.on_line(line_number-1)
      ps.on_line(line_number)
    end
    ps.emit_ident("[")
    ps.emit_newline unless rest.first.nil?

    ps.new_block do
      format_list_like_thing(ps, rest, false)
    end

    ps.emit_indent unless rest.first.nil?
    ps.on_line(ps.current_orig_line_number+1)
    ps.emit_ident("]")
  end
end

def format_array(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  if Parser.is_percent_array?(rest)
    ps.emit_ident(Parser.percent_symbol_for(rest))

    ps.emit_ident("[")
    ps.with_start_of_line(false) do
      parts = rest[0][1]

      parts.each.with_index do |expr, index|
        expr = [expr] if expr[0] == :@tstring_content
        format_inner_string(ps, expr, :array)
        ps.emit_space if index != parts.length - 1
      end
    end
    ps.emit_ident("]")
  else
    format_array_fast_path(ps, rest)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_unary(ps, rest)
  raise "got non size two unary" if rest.count != 2
  op, tail = rest
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(op.to_s.gsub("@", ""))
  if op.to_s == "not"
    ps.emit_space
  end
  ps.start_of_line << false
  format_expression(ps, tail)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_cvar(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0])
  ps.emit_newline if ps.start_of_line.last
end

def format_string_concat(ps, rest)
  ps.start_string_concat

  parts, string = rest
  ps.emit_indent if ps.start_of_line.last
  format_expression(ps, parts)
  ps.emit_space
  ps.emit_slash
  ps.emit_newline
  ps.with_start_of_line(true) do
    format_expression(ps, string)
  end

  ps.end_string_concat

  ps.emit_newline if ps.start_of_line.last && ps.string_concat_position.empty?
end

def format_paren(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("(")
  exprs = rest[0]
  case
  when Symbol === exprs[0]
    # this case arm happens when a yield with a paren is given, e.g.
    # yield(foo)
    format_expression(ps, exprs)
  when exprs.length == 1
    # paren with a single entry
    ps.with_start_of_line(false) do
      format_expression(ps, exprs[0])
    end
  else
    # paren with multiple expressions
    ps.emit_newline
    ps.new_block do
      exprs.each do |expr|
        format_expression(ps, expr)
      end
    end
    ps.emit_newline
  end
  ps.emit_ident(")")
  ps.emit_newline if ps.start_of_line.last
end

def format_begin(ps, expression)
  begin_body, rc, rb, eb = expression

  # I originally named these variables thinking they were 'rescue class'
  # 'rescue block' and 'ensure block' but they are not, those positions
  # are attached to the :bodystmt inside the begin
  raise "get better at begins" if rc != nil || rb != nil || eb != nil
  raise "begin body was not a bodystmt" if begin_body[0] != :bodystmt

  ps.emit_indent if ps.start_of_line.last
  ps.emit_begin
  ps.emit_newline
  ps.new_block do
    format_bodystmt(ps, begin_body[1..-1], inside_begin=true)
  end

  ps.with_start_of_line(true) do
    ps.emit_end
    ps.emit_newline
  end
end

def format_brace_block(ps, expression)
  raise "didn't get right array in brace block" if expression.length != 2
  params, body = expression

  multiline = ParserState.block_will_render_as_multiline?(ps) do |next_ps|
    ps.new_block do
      body.each do |expr|
        format_expression(next_ps, expr)
      end
    end
  end
  orig_params = params

  bv, params, _ = params
  raise "got something other than block var" if bv != :block_var && bv != nil

  ps.emit_ident("{")
  unless bv.nil?
    ps.emit_space
    format_params(ps, orig_params, "|", "|")
  end

  if multiline
    ps.emit_newline
  else
    ps.emit_space
  end

  ps.new_block do
    ps.with_start_of_line(multiline) do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
  end

  if multiline
    ps.emit_indent
  else
    ps.emit_space
  end
  ps.emit_ident("}")
  ps.emit_newline if ps.start_of_line.last
end

def format_float(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.on_line(expression[1][0])
  ps.emit_ident(expression[0])

  ps.emit_newline if ps.start_of_line.last
end

def format_ifop(ps, expression)
  raise "got a non 3 item ternary" if expression.length != 3
  conditional, left, right = expression
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, conditional)
    ps.emit_space
    ps.emit_ident("?")
    ps.emit_space
    format_expression(ps, left)

    if right != nil
      ps.emit_space
      ps.emit_ident(":")
      ps.emit_space

      format_expression(ps, right)
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_assocs(ps, assocs, newlines=true)
  assocs.each_with_index do |assoc, idx|
    ps.emit_indent if newlines

    ps.with_start_of_line(false) do
      if assoc[0] == :assoc_new
        if assoc[1][0] == :@label
          ps.emit_ident(assoc[1][1])
          ps.emit_space
        else
          format_expression(ps, assoc[1])
          ps.emit_space
          ps.emit_ident("=>")
          ps.emit_space
        end

        format_expression(ps, assoc[2])
      elsif assoc[0] == :assoc_splat
        ps.emit_ident("**")
        format_expression(ps, assoc[1])
      else
        raise "got non assoc_new in hash literal #{assocs}"
      end
      if newlines
        ps.emit_ident(",")
        ps.emit_newline
      elsif idx != assocs.length - 1
        ps.emit_ident(",")
        ps.emit_space
      end
    end
  end
end

def format_hash(ps, expression)
  ps.emit_indent if ps.start_of_line.last
  if expression == [nil]
    ps.emit_ident("{}")
  elsif expression[0][0] == :assoclist_from_args
    assocs = expression[0][1]
    ps.emit_ident("{")
    ps.emit_newline
    ps.new_block do
      format_assocs(ps, assocs)
    end
    ps.emit_indent
    ps.emit_ident("}")
  else
    raise "omg"
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_aref_field(ps, expression)
  raise "got bad aref field" if expression.length != 2
  expression, sqb_args = expression
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, expression)
    ps.emit_ident("[")
    ps.surpress_one_paren = true
    format_expression(ps, sqb_args)
    ps.emit_ident("]")
  end
end

def format_aref(ps, expression)
  raise "got bad aref" if expression.length != 2
  expression, sqb_args = expression
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, expression)
    ps.emit_ident("[")
    ps.surpress_one_paren = true
    format_inner_args_list(ps, sqb_args) if sqb_args
    ps.emit_ident("]")
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_bare_assoc_hash(ps, expression)
  if expression[0][0][0] == :assoc_splat
    ps.emit_ident("**")
    assoc_expr = expression[0][0][1]
    ps.with_start_of_line(false) do
      format_expression(ps, assoc_expr)
    end
  else
    ps.new_block do
      format_assocs(ps, expression[0], newlines = false)
    end
  end
end

def format_defined(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("defined?")
  ps.emit_open_paren
  format_expression(ps, rest[0])
  ps.emit_close_paren

  ps.emit_newline if ps.start_of_line.last
end

def format_return0(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("return")

  ps.emit_newline if ps.start_of_line.last
end

def format_massign(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    assigns, rhs = expression

    if assigns[0] == :mlhs_add_star
      assigns, last = [assigns[1],assigns[2]]
      item = last
      assigns << [:rest_param, item]
    end

    assigns.each_with_index do |assign, index|
      format_expression(ps, assign)
      ps.emit_ident(", ") if index != assigns.length - 1
    end

    ps.emit_ident(" = ")
    format_expression(ps, rhs)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_yield(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("yield")

  if expression.first.first != :paren
    ps.emit_space
  end

  ps.with_start_of_line(false) do
    ps.surpress_one_paren = true
    format_expression(ps, expression.first)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_regexp_literal(ps, expression)
  parts,re_end = expression
  ps.on_line(re_end[2][0])
  re_delimiters = case re_end[3][0]
            when "%"
              ["%r#{re_end[3][2]}", re_end[1]]
            when "/"
              ["/", "/"]
            else
              raise "got unknown regular expression"
            end

  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident(re_delimiters[0])

  format_inner_string(ps, parts, :regexp)

  ps.emit_ident(re_delimiters[1])

  if re_end[1].length > 1
    extra_chars = re_end[1][1..-1]
    ps.emit_ident(extra_chars)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_alias(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  first, last = expression

  ps.emit_ident("alias ")
  ps.with_start_of_line(false) do
    format_expression(ps, first)
    ps.emit_space
    format_expression(ps, last)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_field(ps, rest)
  raise "got non 3 length rest" if rest.length != 3
  front = rest[0]
  dot = rest[1]
  back = rest[2]

  ps.emit_indent if ps.start_of_line.last

  line_number = back.last.first
  ps.on_line(line_number)

  ps.start_of_line << false
  format_expression(ps, front)

  format_dot(ps, [dot])

  format_expression(ps, back)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_mrhs_new_from_args(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  parts,tail = expression

  ps.with_start_of_line(false) do
    format_list_like_thing(ps, [parts], true)
    ps.emit_comma_space if tail != nil && tail != []
    format_expression(ps, tail)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_dot2(ps, expression)
  left, right = expression
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, left)
    ps.emit_ident("..")
    format_expression(ps, right) unless right.nil?
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_dot3(ps, expression)
  left, right = expression
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, left)
    ps.emit_ident("...")
    format_expression(ps, right)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_yield0(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("yield")

  ps.emit_newline if ps.start_of_line.last
end

def format_op(ps, expression)
  ps.emit_ident(expression[0])
end

def format_case_parts(ps, case_parts)
  return if case_parts.nil?

  type = case_parts[0]
  if type == :when
    _, conditional, body, case_parts = case_parts
    ps.emit_indent
    ps.emit_when
    ps.emit_space
    ps.with_start_of_line(false) do
      format_list_like_thing(ps, [conditional], true)
    end

    ps.emit_newline
    ps.new_block do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end

    format_case_parts(ps, case_parts)
  elsif type == :else
    _, body = case_parts
    ps.emit_indent
    ps.emit_else
    ps.emit_newline

    ps.new_block do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
  else
    raise "got got bad case"
  end
end

def format_case(ps, rest)
  case_expr, case_parts = rest
  ps.emit_indent if ps.start_of_line.last

  ps.emit_case
  if !case_expr.nil?
    ps.with_start_of_line(false) do
      ps.emit_space
      format_expression(ps, case_expr)
    end
  end

  ps.emit_newline

  format_case_parts(ps, case_parts)
  ps.with_start_of_line(true) do
    ps.emit_end
  end
  ps.emit_newline
end

def format_gvar(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident(rest[0])
end

def format_sclass(ps, rest)
  arrow_expr, statements = rest
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("class << ")
  ps.with_start_of_line(false) do
    format_expression(ps, arrow_expr)
  end
  ps.emit_newline
  ps.new_block do
    format_expression(ps, statements)
  end
  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def format_empty_kwd(ps, expression, keyword)
  raise "omg #{expression}" if !expression.flatten.empty?
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(keyword)
  ps.emit_newline if ps.start_of_line.last
end

def format_while_mod(ps, rest, type)
  while_conditional, while_expr = rest

  ps.emit_indent if ps.start_of_line.last

  # Unwrap parens, so that we can consistently decide if we need them
  # or not when doing the final formatting.
  if while_expr[0] == :paren
    while_exprs = while_expr[1]
  else
    while_exprs = [while_expr]
  end

  multiline = ParserState.block_will_render_as_multiline?(ps) do |next_ps|
    while_exprs.each do |while_expr|
      format_expression(next_ps, while_expr)
    end
  end
  ps.with_start_of_line(false) do
    if multiline
      ps.emit_open_paren
      ps.emit_newline
      ps.new_block do
        while_exprs.each do |while_expr|
          format_expression(ps, while_expr)
        end
      end
      ps.emit_indent
      ps.emit_close_paren
    else
      format_expression(ps, while_expr)
    end

    ps.emit_ident(" #{type} ")
    format_expression(ps, while_conditional)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_mlhs(ps, expression)
  ps.emit_open_paren
  ps.with_start_of_line(false) do
    expression.each_with_index do |expr, index|
      format_expression(ps, expr)
      if index != expression.length - 1
        ps.emit_comma_space
      end
    end
  end
  ps.emit_close_paren
end

def format_dyna_symbol(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(":")
  ps.with_start_of_line(false) do
    format_string_literal(ps, rest)
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_rest_param(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("*")
  ps.with_start_of_line(false) do
    format_expression(ps, rest[0])
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_undef(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("undef ")
  ps.with_start_of_line(false) do
    format_expression(ps, rest[0][0])
  end
  ps.emit_newline if ps.start_of_line.last
end

# mlhs_paren occurs when a block arg has parenthesisation for array unpacking
# e.g. do |a, (b, c, (d, e))|. it is illegal to call this function with
# start_of_line.last == true
def format_mlhs_paren(ps, rest)
  raise if ps.start_of_line.last
  ps.emit_ident("(")

  ps.with_start_of_line(false) do
    rest[0].each_with_index do |item, idx|
      case item[1][0]
      when Array
        format_mlhs_paren(ps, [item[1]])
      when :@ident
        ps.emit_ident(item[1][1])
      else
        raise "got a bad mlhs paren"
      end

      ps.emit_comma_space unless idx == rest[0].count - 1
    end
  end

  ps.emit_ident(")")
end

def format_mrhs_add_star(ps, expression)
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    ps.emit_ident("*")
    format_expression(ps, expression.last)
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_while(ps, rest)
  condition, expressions = rest

  ps.emit_indent if ps.start_of_line.last

  ps.emit_while
  ps.emit_ident(" ")
  ps.with_start_of_line(false) do
    format_expression(ps, condition)
  end
  ps.emit_newline
  ps.new_block do
    expressions.each do |expression|
      ps.with_start_of_line(true) do
        format_expression(ps, expression)
      end
      ps.emit_newline
    end
  end
  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def format_for(ps, rest)
  loop_vars, iterable, expressions = rest

  unless Array === loop_vars[0]
    loop_vars = [loop_vars]
  end

  ps.emit_indent if ps.start_of_line.last

  ps.emit_for
  ps.emit_ident(" ")
  format_list_like_thing_items(ps, [loop_vars], true)
  ps.emit_ident(" ")
  ps.emit_in
  ps.emit_ident(" ")
  ps.with_start_of_line(false) do
    format_expression(ps, iterable)
  end
  ps.emit_newline
  ps.new_block do
    expressions.each do |expression|
      ps.with_start_of_line(true) do
        format_expression(ps, expression)
      end
      ps.emit_newline
    end
  end
  ps.emit_end

  ps.emit_newline if ps.start_of_line.last
end

def format_lambda(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  line_number = extract_line_number_from_construct(rest)
  if line_number
    ps.on_line(line_number)
  end
  params, type, body = rest
  ps.emit_stabby_lambda
  if params[0] == :paren
    params = params[1]
  end
  ps.emit_space if params.drop(1).any?
  format_params(ps, params, "(", ")")

  delim = if type == :do
    ["do", "end"]
  else
    ["{", "}"]
  end

  # lambdas typically are a single statement, so line breaking them would
  # be masochistic
  if delim[0] == "{" && body.length == 1
    ps.emit_ident(" { ")
    ps.with_start_of_line(false) do
      format_expression(ps, body[0])
    end

    ps.emit_ident(" }")
  else
    ps.emit_ident(" #{delim[0]}")
    ps.emit_newline
    ps.new_block do
      if body[0] != :bodystmt
        body.each do |expr|
          format_expression(ps, expr)
          ps.emit_newline
        end
      else
        format_bodystmt(ps, body.drop(1))
      end
    end
    ps.emit_ident(delim[1])

  end

  ps.emit_newline if ps.start_of_line.last
end

def format_rescue_mod(ps, expression)
  expression, rescue_clause = expression

  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    format_expression(ps, expression)
    ps.emit_ident(" rescue ")
    format_expression(ps, rescue_clause)
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_backref(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident(expression[0])
  ps.on_line(expression[1][0])

  ps.emit_newline if ps.start_of_line.last
end

def format_keyword_with_args(ps, expression, keyword)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident(keyword)
  if expression[0] && expression[0][1]
    ps.emit_ident(" ")
    ps.with_start_of_line(false) do
      format_expression(ps, expression[0][1][0])
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_symbol(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  expression = expression[0]
  ps.on_line(expression[2][0])
  ps.emit_ident(":")
  ps.emit_ident(expression[1])

  ps.emit_newline if ps.start_of_line.last
end

def format_redo(ps, expression)
  if !expression.empty?
    raise "omg redo #{expression}"
  end

  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("redo")
  ps.emit_newline if ps.start_of_line.last
end

def format_splat(ps, rest)
  ps.emit_ident("*")
  format_expression(ps, rest[0])
end

def format_to_proc(ps, rest)
  ps.emit_ident("&")
  format_expression(ps, rest[0])
end

def format_keyword(ps, rest)
  ps.emit_ident(rest[0])
end

def use_parens_for_method_call(ps, method, args, original_used_parens)
  return false if ps.formatting_class_or_module_stack.last

  # Always use parens for the shorthand `foo::()` syntax
  return true if method == :call

  # Never use parens for some methods and keywords
  return false if ["return", "raise"].include?(method[1])

  # Follow the original code style for super and yield
  # Note that `super()` has different semantics to `super`
  return original_used_parens if ["super", "yield", "require"].include?(method[1])

  # No parens if there are no arguments
  return false if args.empty?

  # If in doubt, use parens
  true
end

def format_method_call(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  chain, method, original_used_parens, args = rest

  use_parens = use_parens_for_method_call(
    ps,
    method,
    args,
    original_used_parens,
  )

  ps.with_start_of_line(false) do
    chain.each do |chain_expr|
      format_expression(ps, chain_expr)
    end

    if method != :call
      format_expression(ps, method)
    end

    if use_parens
      ps.emit_ident("(")
    elsif args.any?
      ps.emit_ident(" ")
    end

    format_list_like_thing_items(ps, [args], true)

    if use_parens
      ps.emit_ident(")")
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def normalize_method_add_arg(rest)
  [
    :method_call,
    *normalize_inner_call(rest[0]),
    true,
    normalize_args(rest[1]),
  ]
end

def normalize_command(rest)
  [
    :method_call,
    [],
    rest[0],
    false,
    normalize_args(rest[1]),
  ]
end

def normalize_command_call(rest)
  head, tail = normalize_inner_call(rest[0])
  [
    :method_call,
    head + [tail, [:dot, rest[1]]],
    rest[2],
    false,
    normalize_args(rest[3]),
  ]
end

def normalize_call(rest)
  [
    :method_call,
    *normalize_inner_call([:call, *rest]),
    false,
    [],
  ]
end

def normalize_vcall(rest)
  [
    :method_call,
    [],
    rest[0],
    false,
    [],
  ]
end

def normalize_zsuper(rest)
  [
    :method_call,
    [],
    [:keyword, "super"],
    false,
    [],
  ]
end

def normalize_super(rest)
  [
    :method_call,
    [],
    [:keyword, "super"],
    rest[0][0] == :arg_paren,
    normalize_args(rest[0]),
  ]
end

def normalize_return(rest)
  [
    :method_call,
    [],
    [:keyword, "return"],
    false,
    normalize_args(rest[0]),
  ]
end

def normalize_yield(rest)
  [
    :method_call,
    [],
    [:keyword, "yield"],
    rest[0][0] == :paren,
    normalize_args(rest[0]),
  ]
end

def normalize_arg_paren(rest)
  args = rest[0]

  if args.nil?
    []
  else
    normalize_args(args)
  end
end

def normalize_paren(rest)
  args = rest[0]

  if args.nil?
    []
  else
    normalize_args(args)
  end
end

def normalize_args_add_block(rest)
  block = rest[1]
  if block
    [*normalize_args(rest[0]), [:to_proc, block]]
  else
    normalize_args(rest[0])
  end
end

def normalize_args_add_star(rest)
  [*rest[0], [:splat, rest[1]]]
end

def normalize_inner_call(expr)
  type, rest = expr[0], expr[1..-1]

  case type
  when :fcall, :vcall
    [[], rest[0]]
  when :call
    a, b = normalize_inner_call(rest[0])
    [
      a + [b, [:dot, rest[1]]],
      rest[2],
    ]
  else
    [[], expr]
  end
end

def normalize_args(expr)
  type, rest = expr[0], expr[1..-1]

  case type
  when :arg_paren
    normalize_arg_paren(rest)
  when :paren
    normalize_paren(rest)
  when :args_add_block
    normalize_args_add_block(rest)
  when :args_add_star
    normalize_args_add_star(rest)
  else
    expr
  end
end

def normalize(expr)
  type, rest = expr[0], expr[1..-1]

  case type
  when :method_add_arg
    normalize_method_add_arg(rest)
  when :command
    normalize_command(rest)
  when :command_call
    normalize_command_call(rest)
  when :call
    normalize_call(rest)
  when :vcall
    normalize_vcall(rest)
  when :zsuper
    normalize_zsuper(rest)
  when :super
    normalize_super(rest)
  when :return
    normalize_return(rest)
  when :yield
    normalize_yield(rest)
  else
    expr
  end
end

EXPRESSION_HANDLERS = {
  :def => method(:format_def),
  :void_stmt => method(:format_void_expression),
  :assign => method(:format_assign_expression),
  :method_add_block => method(:format_method_add_block),
  :@int => method(:format_int),
  :@rational => method(:format_rational),
  :@imaginary => method(:format_imaginary),
  :var_ref => method(:format_var_ref),
  :do_block => method(:format_do_block),
  :binary => method(:format_binary),
  :string_literal => method(:format_string_literal),
  :module => method(:format_module),
  :class => method(:format_class),
  :const_path_ref => method(:format_const_path_ref),
  :const_path_field => method(:format_const_path_field),
  :top_const_field => method(:format_top_const_field),
  :@ident => method(:format_ident),
  :symbol_literal => method(:format_symbol_literal),
  :const_ref => method(:format_const_ref),
  :"@const" => method(:format_const),
  :defs => method(:format_defs),
  :@kw => method(:format_kw),
  :bodystmt => method(:format_bodystmt),
  :if_mod => method(:format_if_mod),
  :unless_mod => method(:format_unless_mod),
  :if => method(:format_if),
  :opassign => method(:format_opassign),
  :var_field => method(:format_var_field),
  :@ivar => method(:format_ivar),
  :top_const_ref => method(:format_top_const_ref),
  :array => method(:format_array),
  :unary => method(:format_unary),
  :paren => method(:format_paren),
  :string_concat => method(:format_string_concat),
  :unless => method(:format_unless),
  :begin => method(:format_begin),
  :brace_block => method(:format_brace_block),
  :@float => method(:format_float),
  :ifop => method(:format_ifop),
  :hash => method(:format_hash),
  :aref_field => method(:format_aref_field),
  :aref => method(:format_aref),
  :args_add_block => method(:format_args_add_block),
  :bare_assoc_hash => method(:format_bare_assoc_hash),
  :defined => method(:format_defined),
  :until => method(:format_until),
  :return0 => method(:format_return0),
  :massign => method(:format_massign),
  :yield => method(:format_yield),
  :regexp_literal => method(:format_regexp_literal),
  :alias => method(:format_alias),
  :field => method(:format_field),
  :mrhs_new_from_args => method(:format_mrhs_new_from_args),
  :dot2 => method(:format_dot2),
  :dot3 => method(:format_dot3),
  :yield0 => method(:format_yield0),
  :@op => method(:format_op),
  :case => method(:format_case),
  :@gvar => method(:format_gvar),
  :sclass => method(:format_sclass),
  :retry => lambda { |ps, rest| format_empty_kwd(ps, rest, "retry") },
  :break => lambda { |ps, rest| format_keyword_with_args(ps, rest, "break") },
  :next => lambda { |ps, rest| format_keyword_with_args(ps, rest, "next") },
  :while_mod => lambda { |ps, rest| format_while_mod(ps, rest, "while") },
  :until_mod => lambda { |ps, rest| format_while_mod(ps, rest, "until") },
  :mlhs => method(:format_mlhs),
  :dyna_symbol => method(:format_dyna_symbol),
  :rest_param => method(:format_rest_param),
  :undef => method(:format_undef),
  :@cvar => method(:format_cvar),
  :mlhs_paren => method(:format_mlhs_paren),
  :mrhs_add_star => method(:format_mrhs_add_star),
  :while => method(:format_while),
  :for => method(:format_for),
  :lambda => method(:format_lambda),
  :rescue_mod => method(:format_rescue_mod),
  :xstring_literal => method(:format_xstring_literal),
  :@backref => method(:format_backref),
  :@CHAR => method(:format_character_literal),
  :symbol => method(:format_symbol),
  :redo => method(:format_redo),

  # Normalized by rubyfmt, not from Ripper:
  :dot => lambda { |ps, rest| format_dot(ps, rest) },
  :method_call => lambda { |ps, rest| format_method_call(ps, rest) },
  :splat => lambda { |ps, rest| format_splat(ps, rest) },
  :to_proc => lambda { |ps, rest| format_to_proc(ps, rest) },
  :keyword => lambda { |ps, rest| format_keyword(ps, rest) },
}.freeze

def extract_line_number_from_construct(construct)
  line_re = /(\[\d+, \d+\])/
  line_number = line_re.match(construct.inspect)
  if line_number != nil
    line_number = line_number.to_s.split(",")[0].gsub("[", "").to_i
    line_number
  else
    nil
  end
end

def format_expression(ps, expression)
  expression = normalize(expression)

  type, rest = expression[0],expression[1...expression.length]

  #line_number = extract_line_number_from_construct(rest)
  #if line_number != nil
  #  ps.on_line(line_number)
  #end

  EXPRESSION_HANDLERS.fetch(type).call(ps, rest)
rescue KeyError => e
  puts ps.current_orig_line_number
  puts ps.line
  raise e
end

def format_program(line_metadata, sexp, result)
  program, expressions = sexp
  ps = ParserState.new(result, line_metadata)
  expressions.each do |expression|
    format_expression(ps, expression)
  end
  ps.write
end

def extract_line_metadata(file_data)
  comment_blocks = {}

  file_data.split("\n").each_with_index do |line, index|
    comment_blocks[index] = line if /^ *#/ === line
  end

  LineMetadata.new(comment_blocks)
end

class Parser < Ripper::SexpBuilderPP
  ARRAY_SYMBOLS = {
    qsymbols: '%i',
    qwords: '%w',
    symbols: '%I',
    words: '%W'
  }.freeze

  def self.is_percent_array?(rest)
    return false if rest.nil?
    return false if rest[0].nil?
    ARRAY_SYMBOLS.include?(rest[0][0])
  end

  def self.percent_symbol_for(rest)
    ARRAY_SYMBOLS[rest[0][0]]
  end

  def initialize(file_data)
    super(file_data)
    @file_lines = file_data.split("\n")
    # heredoc stack is the stack of identified heredocs
    @heredoc_stack = []

    # next_heredoc_stack is the type identifiers of the next heredocs, that
    # we haven't emitted yet
    @next_heredoc_stack = []
    @heredoc_regex = /(<<[-~]?)(.*$)/
    @next_comment_delete = []
    @comments_delete = []
    @regexp_stack = []
    @string_stack = []
  end

  attr_reader :comments_delete

  private

  ARRAY_SYMBOLS.each do |event, symbol|
    define_method(:"on_#{event}_new") do
      [event, [], [lineno, column]]
    end

    define_method(:"on_#{event}_add") do |parts, part|
      parts.tap do |node|
        node[1] << part
      end
    end
  end

  def on_heredoc_beg(*args, &blk)
    heredoc_parts = @heredoc_regex.match(args[0]).captures
    raise "bad heredoc" unless heredoc_parts.select { |x| x != nil }.count == 2
    @next_heredoc_stack.push(heredoc_parts)
    @next_comment_delete.push(lineno)
    super
  end

  def on_heredoc_end(*args, &blk)
    @heredoc_stack.push(@next_heredoc_stack.pop)
    start_com = @next_comment_delete.pop
    end_com = lineno
    @comments_delete.push([start_com, end_com])
    super
  end

  def on_string_literal(*args, &blk)
    if @heredoc_stack.last
      heredoc_parts = @heredoc_stack.pop
      args.insert(0, [:heredoc_string_literal, heredoc_parts])
    else
      end_delim = @string_stack.pop
      start_delim = @string_stack.pop

      if start_delim != "\""
        reject_embexpr = start_delim == "'" || start_delim.start_with?("%q")

        (args[0][1..-1] || []).each do |part|
          next if part.nil?
          case part[0]
          when :@tstring_content
            part[1] = eval("#{start_delim}#{part[1]}#{end_delim}").inspect[1..-2]
          when :string_embexpr, :string_dvar
            if reject_embexpr
              raise "got #{part[0]} in a #{start_delim}...#{end_delim} string"
            end
          else
            raise "got #{part[0]} in a #{start_delim}...#{end_delim} string"
          end
        end
      end
    end
    super
  end

  def on_lambda(*args, &blk)
    terminator = @file_lines[lineno-1]
    if terminator.include?("}")
      args.insert(1, :curly)
    else
      args.insert(1, :do)
    end

    super
  end

  def on_tstring_beg(*args, &blk)
    @string_stack << args[0]
    super
  end

  def on_tstring_end(*args, &blk)
    @string_stack << args[0]
    super
  end

  def on_regexp_beg(re_part)
    @regexp_stack << re_part
  end

  def on_regexp_literal(*args)
    args[1] << @regexp_stack.pop
    super(*args)
  end


  def on_void_stmt(*args)
    args << [lineno, column]
    super(*args)
  end
end

def main
  file_data = ARGF.read
  file_data = file_data.gsub("\r\n", "\n")

  line_metadata = extract_line_metadata(file_data)

  parser = Parser.new(file_data)
  sexp = parser.parse
  if ENV["RUBYFMT_DEBUG"] == "2"
    require 'pry'; binding.pry
  end
  if parser.error?
    if ENV["RUBYFMT_DEBUG"] == "2"
      require 'pry'; binding.pry
    end
    raise parser.error
  end

  parser.comments_delete.each do |(start, last)|
    line_metadata.comment_blocks.reject! { |k, v| k >= start && k <= last }
  end
  format_program(line_metadata, sexp, $stdout)
end

main if __FILE__ == $0
