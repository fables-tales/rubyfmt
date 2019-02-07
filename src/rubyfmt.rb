require 'ripper'
require 'stringio'
require 'pp'

FILE = ARGV[0]
MODE = :inline

class Line
  def initialize(parts)
    @comments = []
    @parts = parts
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
    @parts.any? { |x| x == "end" }
  end

  def contains_def?
    @parts.any? { |x| x == :def }
  end

  def contains_do?
    @parts.any? { |x| x == :do }
  end

  def declares_private?
    @parts.any? { |x| x == "private" } && @parts.length == 3
  end

  def declares_require?
    @parts.any? { |x| x == "require" }
  end

  def declares_class_or_module?
    @parts.any? { |x| /(class|module)/ === x }
  end
end

def want_blankline?(line, next_line)
  return unless next_line
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.contains_def?
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
  return true if !line.declares_class_or_module? && next_line.has_comment?
  return true if !line.declares_class_or_module? && next_line.declares_class_or_module?
end

class ParserState
  attr_accessor :depth, :start_of_line, :line, :string_concat_position, :surpress_one_paren
  def initialize(result, line_metadata)
    @surpress_one_paren = false
    @result = result
    @depth_stack = [0]
    @start_of_line = [true]
    @render_queue = []
    @line = Line.new([])
    @current_orig_line_number = 0
    @comments_hash = line_metadata.comment_blocks
    @conditional_indent = [0]
    @string_concat_position = []
  end

  def with_start_of_line(value, &blk)
    start_of_line << value
    blk.call
    start_of_line.pop
  end

  def start_string_concat
    push_conditional_indent if @string_concat_position.empty?
    @string_concat_position << Object.new
  end

  def end_string_concat
    @string_concat_position.pop
    pop_conditional_indent if @string_concat_position.empty?
  end

  def on_line(line_number)
    if line_number != @current_orig_line_number
      @arrays_on_line = -1
    end

    while !comments_hash.empty? && comments_hash.keys.sort.first < line_number
      key = comments_hash.keys.sort.first
      @line.push_comment(comments_hash.delete(key))
    end

    @current_orig_line_number = line_number
  end

  def write
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
    line << " " * spaces
  end

  def emit_slash
    line << "\\"
  end

  def emit_else
    line << "else"
  end

  def emit_elsif
    line << "elsif"
  end

  def push_conditional_indent
    if line.empty?
      @conditional_indent << 2*@depth_stack.last
    else
      @conditional_indent << line.string_length
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
    line << "end"
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
    emit_indent if start_of_line.last
    line << int
  end

  def emit_var_ref(ref)
    emit_indent if start_of_line.last
    line << ref
  end

  def emit_open_paren
    line << "("
  end

  def emit_close_paren
    line << ")"
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

  def emit_module_keyword
    line << "module"
  end

  def emit_class_keyword
    line << "class"
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

  private

  attr_reader :result
  attr_reader :render_queue
  attr_reader :comments_hash
  attr_reader :depth_stack
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
  def_expression, params, body = rest

  def_name = def_expression[1]

  line_number = def_expression.last.first
  ps.on_line(line_number)

  params = rest[1]
  body = rest[2]

  ps.emit_indent
  ps.emit_def(def_name)

  format_params(ps, params, "(", ")")

  ps.emit_newline
  ps.new_block do
    format_expression(ps, body)
  end

  ps.emit_end
  ps.emit_newline
  ps.emit_newline
end

def format_required_params(ps, required_params)
  build = required_params.map { |x|
    raise "got a non ident param #{x}" if x[0] != :"@ident"
    x[1]
  }.join(", ")
  ps.emit_ident(build)
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
  return if rest_params.empty?
  ps.emit_ident("*")
  return if rest_params[1].nil?

  rest_param, expr = rest_params
  raise "got bad rest_params" if rest_param != :rest_param

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
  if params[0] == :paren || params[0] == :block_var
    params = params[1]
  end

  have_any_params = params[1..-1].any? { |x| !x.nil? }

  if have_any_params
    ps.emit_ident(open_delim)
  end

  bad_params = params[4..-1].any? { |x| !x.nil? }
  bad_params = false if params[5]
  bad_params = false if params[7]

  raise "dont know how to deal with a params list" if bad_params
  required_params = params[1] || []
  optional_params = params[2] || []
  rest_params = params[3] || []
  kwargs = params[5] || []
  block_arg = params[7] || []

  format_required_params(ps, required_params)

  did_emit = !required_params.empty?
  have_more = !optional_params.empty? || !rest_params.empty? || !kwargs.empty? || !block_arg.empty?
  ps.emit_ident(", ") if did_emit && have_more

  format_rest_params(ps, rest_params)

  did_emit = !rest_params.empty?
  have_more = !optional_params.empty? || !kwargs.empty? || !block_arg.empty?
  ps.emit_ident(", ") if did_emit && have_more

  format_optional_params(ps, optional_params)

  did_emit = !optional_params.empty?
  have_more = !kwargs.empty? || !block_arg.empty?
  ps.emit_ident(", ") if did_emit && have_more

  format_kwargs(ps, kwargs)

  did_emit = !kwargs.empty?
  have_more = !block_arg.empty?
  ps.emit_ident(", ") if did_emit && have_more

  format_blockarg(ps, block_arg)

  if have_any_params
    ps.emit_ident(close_delim)
  end
end

def format_void_expression(ps, rest)
end

def format_opassign(ps, rest)
  head, op, tail = rest

  format_expression(ps, head)
  ps.emit_space
  ps.emit_op(op[1])
  ps.emit_space

  ps.with_start_of_line(false) do
    format_expression(ps, tail)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_assign_expression(ps, rest)
  head, tail = rest
  format_expression(ps, head)
  ps.emit_space
  ps.emit_op("=")
  ps.emit_space

  ps.with_start_of_line(false) do
    format_expression(ps, tail)
  end

  ps.emit_newline
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
  int = rest[0]
  ps.emit_int(int)
end

def format_var_ref(ps, rest)
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

def format_method_add_arg(ps, rest)
  type, call_rest = rest.first, rest.drop(1)

  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, type)
  end

  raise "got call rest longer than one" if call_rest.length > 1
  args_list = call_rest[0]
  if args_list[0] == :arg_paren
    args_list = args_list[1]
    if args_list.count == 1
      args_list = args_list.first
    end
  elsif args_list[0] == :args_add_block
  elsif args_list.empty?
  else
    raise "got non call paren args list"
  end

  format_expression(ps, args_list) unless args_list.empty?
  ps.emit_newline if ps.start_of_line.last
end

def format_command(ps, rest)
  # this is definitely wrong
  ident = rest[0]
  {
    :"@ident" => lambda {
      ps.emit_indent if ps.start_of_line.last
      ps.emit_ident(ident[1])
    },
  }.fetch(rest[0][0]).call

  args_list = rest[1]
  if args_list.count == 1
    args_list = args_list[0]
  end

  ps.with_start_of_line(false) do
    if !args_list.nil? && args_list[0] == :command_call
      ps.emit_space
    end
    format_expression(ps, args_list)
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_vcall(ps, rest)
  raise "didn't get exactly one part" if rest.count != 1
  raise "didn't get an ident" if rest[0][0] != :"@ident"

  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0][1])
  ps.emit_newline if ps.start_of_line.last
end

def format_tstring_content(ps, rest)
  ps.emit_ident(rest[1])
  ps.on_line(rest[2][0])
end

def format_inner_string(ps, parts)
  parts.each do |part|
    case part[0]
    when :@tstring_content
      ps.emit_ident(part[1])
      ps.on_line(part[2][0])
    when :string_embexpr
      ps.emit_ident("\#{")
      ps.start_of_line << false
      format_expression(ps, part[1][0])
      ps.emit_ident("}")
      ps.start_of_line.pop
    else
      raise "dont't know how to do this"
    end
  end
end

def format_string_literal(ps, rest)
  items = rest[0]
  string_content, parts = items[0], items[1..-1]
  ps.emit_indent if ps.start_of_line.last
  ps.emit_double_quote

  format_inner_string(ps, parts)

  ps.emit_double_quote
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


  ps.new_block do
    exprs = rest[1][1]
    exprs.each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def format_fcall(ps, rest)
  # this is definitely wrong
  raise "omg" if rest.length != 1
  {
    :@ident => lambda {
      ps.emit_indent if ps.start_of_line.last
      ps.emit_ident(rest[0][1])
    },
    :@const => lambda {
      ps.emit_indent if ps.start_of_line.last
      ps.emit_const(rest[0][1])
    },
  }.fetch(rest[0][0]).call
end

def format_class(ps, rest)
  class_name = rest[0]
  raise "didn't get a const ref" if class_name[0] != :const_ref
  raise "didn't get a const" if class_name[1][0] != :"@const"

  ps.emit_indent
  ps.emit_class_keyword
  ps.start_of_line << false
  ps.emit_space
  ps.emit_const(class_name[1][1])
  ps.on_line(class_name[1][2].first)
  ps.start_of_line.pop

  if rest[1] != nil
    ps.emit_ident(" < ")
    ps.start_of_line << false
    format_expression(ps, rest[1])
    ps.start_of_line.pop
  end

  ps.emit_newline

  ps.new_block do
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

def format_call(ps, rest)
  raise "got non 3 length rest" if rest.length != 3
  front = rest[0]
  dot = rest[1]
  back = rest[2]

  ps.emit_indent if ps.start_of_line.last

  line_number = back.last.first
  ps.on_line(line_number)

  ps.start_of_line << false
  format_expression(ps, front)

  case
  when dot == :"." || dot == :"::" || (dot.is_a?(Array) && dot[0] == :@period)
    ps.emit_dot
  when dot == :"&."
    ps.emit_lonely_operator
  else
    raise "got unrecognised dot"
  end

  format_expression(ps, back)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_ident(ps, ident)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(ident[0])
end

def format_symbol_literal(ps, literal)
  ps.emit_indent if ps.start_of_line.last
  ps.with_start_of_line(false) do
    if literal[0][0] == :@ident
      format_expression(ps, literal[0])
    elsif literal[0][0] == :symbol
      ps.emit_ident(":")
      format_expression(ps, literal[0][1])
    else
      raise "didn't get ident in right position" if literal[0][1][0] != :"@ident"
      ps.emit_symbol(literal[0][1][1])
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_command_call(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  left, dot, right, args = expression

  ps.with_start_of_line(false) do
    format_expression(ps, left)
    raise "got something other than a dot" if dot != :"."
    ps.emit_dot
    format_expression(ps, right)
    format_expression(ps, args)
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_args_add_block(ps, args_list)
  surpress_paren = ps.surpress_one_paren
  ps.surpress_one_paren = false
  ps.emit_open_paren unless surpress_paren
  ps.start_of_line << false

  emitted_args = false

  if args_list[0][0] != :args_add_star
    args_list[0].each_with_index do |expr, idx|
      format_expression(ps, expr)
      ps.emit_comma_space unless idx == args_list[0].count-1
      emitted_args = true
    end
  else
    _, other_args, call = args_list[0]
    other_args.each_with_index do |expr, idx|
      format_expression(ps, expr)
      ps.emit_comma_space unless idx == other_args.count-1
      emitted_args = true
    end
    ps.emit_comma_space if emitted_args
    ps.emit_ident("*")
    emitted_args = true
    format_expression(ps, call)
  end

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
  _, rescue_class, rescue_capture, rescue_expressions = rescue_part
  ps.dedent do
    ps.emit_indent
    ps.emit_ident("rescue")
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
end

def format_ensure(ps, ensure_part)
  return if ensure_part.nil?

  _, ensure_expressions = ensure_part
  ps.dedent do
    ps.emit_indent
    ps.emit_ident("ensure")
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

  _, exprs = else_part
  ps.dedent do
    ps.emit_indent
    ps.emit_ident("else")
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

def format_return(ps, rest)
  raise "got wrong size return args" if rest.length != 1
  raise "didn't get args add block to return" if rest.first.first != :args_add_block
  ps.emit_indent if ps.start_of_line.last
  ps.start_of_line << false
  ps.emit_return
  ps.emit_space
  format_expression(ps, rest.first[1].first)
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
  format_conditional(ps, expression, "unless")
end

def format_if(ps, expression)
  format_conditional(ps, expression, "if")
end

def format_conditional(ps, expression, kind)
  ps.push_conditional_indent
  if_conditional, body, further_conditionals = expression[0], expression[1], expression[2]

  ps.emit_indent if ps.start_of_line.last
  ps.start_of_line << false
  ps.emit_ident(kind)
  ps.emit_space
  format_expression(ps, if_conditional)
  ps.start_of_line.pop

  ps.emit_newline
  ps.new_block do
    body.each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.start_of_line << true
  ps.emit_newline
  format_conditional_parts(ps, further_conditionals || [])

  ps.emit_end
  ps.start_of_line.pop
  ps.pop_conditional_indent
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

def format_super(ps, rest)
  return if rest.nil?
  raise "got bad super" if rest.length != 1
  args = rest[0]
  if rest[0][0] == :arg_paren
    args = rest[0][1]
  end

  raise "nope args" if args[0] != :args_add_block

  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("super")

  ps.start_of_line << false
  format_expression(ps, args)
  ps.start_of_line.pop
end

def format_zsuper(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident("super")

  ps.emit_newline if ps.start_of_line.last
end

def format_array_fast_path(ps, rest)
  ps.emit_ident("[")
  ps.emit_newline unless rest.first.nil?

  ps.new_block do
    (rest.first || []).each do |expr|
      ps.emit_indent
      ps.with_start_of_line(false) do
        raise "this is bad" if expr[0] == :tstring_content
        format_expression(ps, expr)
      end
      ps.emit_ident(",")
      ps.emit_newline
    end
  end

  ps.emit_indent unless rest.first.nil?
  ps.emit_ident("]")
end

def format_array(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  if Parser::ARRAY_SYMBOLS.key?(rest[0][0])
    ps.emit_ident("#{Parser::ARRAY_SYMBOLS[rest[0][0]]}[")
    ps.with_start_of_line(false) do
      parts = rest[0][1]

      parts.each.with_index do |expr, index|
        expr = [expr] if expr[0] == :@tstring_content
        format_inner_string(ps, expr)
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
  ps.start_of_line << false
  format_expression(ps, tail)
  ps.start_of_line.pop
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
  ps.start_of_line << true
  format_expression(ps, string)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last

  ps.end_string_concat
end

def format_paren(ps, rest)
  raise "didn't get len 1 paren" if rest.length != 1 && rest[0].length != 1
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("(")
  format_expression(ps, rest[0][0])
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
  ps.emit_ident("begin")
  ps.emit_newline
  ps.new_block do
    format_bodystmt(ps, begin_body[1..-1], inside_begin=true)
  end
  ps.start_of_line << true
  ps.emit_end
  ps.start_of_line.pop
end

def format_brace_block(ps, expression)
  raise "didn't get right array in brace block" if expression.length != 2
  params, body = expression

  bv, params, f = params
  raise "got something other than block var" if bv != :block_var && bv != nil
  raise "got something other than false" if f != false && f != nil
  ps.emit_ident("{")
  unless bv.nil?
    ps.emit_space
    format_params(ps, params, "|", "|")
  end
  ps.emit_newline
  ps.new_block do
    body.each do |expr|
      format_expression(ps, expr)
    end
  end
  ps.emit_indent
  ps.emit_ident("}")
  ps.emit_newline if ps.start_of_line.last
end

def format_float(ps, expression)
  ps.emit_ident(expression[0])
end

def format_ifop(ps, expression)
  raise "got a non 3 item ternary" if expression.length != 3
  conditional, left, right = expression
  format_expression(ps, conditional)
  ps.with_start_of_line(false) do
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
        if newlines
          ps.emit_ident(",")
          ps.emit_newline
        elsif idx != assocs.length - 1
          ps.emit_ident(",")
          ps.emit_space
        end
      else
        raise "got non assoc_new in hash literal"
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
  ps.with_start_of_line(false) do
    ps.emit_indent
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
    format_expression(ps, sqb_args)
    ps.emit_ident("]")
  end
  ps.emit_newline if ps.start_of_line.last
end

def format_bare_assoc_hash(ps, expression)
  ps.new_block do
    format_assocs(ps, expression[0], newlines = false)
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

  ps.emit_ident("yield ")

  ps.with_start_of_line(false) do
    ps.surpress_one_paren = true
    format_expression(ps, expression.first)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_regexp_literal(ps, expression)
  parts,re_end = expression
  re_type = case re_end[1][0]
            when "}"
              :curly
            when "/"
              :slashy
            else
              raise "got unknown regular expression"
            end

  re_delimiters = {
    :curly => ["%r{", "}"],
    :slashy => ["/", "/"],
  }.fetch(re_type)

  ps.emit_indent if ps.start_of_line.last

  ps.emit_ident(re_delimiters[0])

  format_inner_string(ps, parts)

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
  format_call(ps, rest)
end

def format_mrhs_new_from_args(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  parts,tail = expression

  ps.with_start_of_line(false) do
    parts.each do |expr|
      format_expression(ps, expr)
      ps.emit_ident(", ")
    end

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
    ps.emit_ident("when ")
    ps.with_start_of_line(false) do
      format_expression(ps, conditional.first)
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
    ps.emit_ident("else")
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

  ps.emit_ident("case")
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

def format_retry(ps, expression)
  raise "omg" if !expression.empty?
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident("retry")
  ps.emit_newline if ps.start_of_line.last
end

def format_expression(ps, expression)
  type, rest = expression[0],expression[1...expression.length]

  line_re = /(\[\d+, \d+\])/
  line_number = line_re.match(rest.inspect)
  if line_number != nil
    line_number = line_number.to_s.split(",")[0].gsub("[", "").to_i
    ps.on_line(line_number)
  end

  {
    :return => lambda { |ps, rest| format_return(ps, rest) },
    :def => lambda { |ps, rest| format_def(ps, rest) },
    :void_stmt => lambda { |ps, rest| format_void_expression(ps, rest) },
    :assign => lambda { |ps, rest| format_assign_expression(ps, rest) },
    :method_add_block => lambda { |ps, rest| format_method_add_block(ps, rest) },
    :@int => lambda { |ps, rest| format_int(ps, rest) },
    :var_ref => lambda { |ps, rest| format_var_ref(ps, rest) },
    :do_block => lambda { |ps, rest| format_do_block(ps, rest) },
    :binary => lambda { |ps, rest| format_binary(ps, rest) },
    :command => lambda { |ps, rest| format_command(ps, rest) },
    :method_add_arg => lambda { |ps, rest| format_method_add_arg(ps, rest) },
    :vcall => lambda { |ps, rest| format_vcall(ps, rest) },
    :fcall => lambda { |ps, rest| format_fcall(ps, rest) },
    :string_literal => lambda { |ps, rest| format_string_literal(ps, rest) },
    :module => lambda { |ps, rest| format_module(ps, rest) },
    :class => lambda { |ps, rest| format_class(ps, rest) },
    :call => lambda { |ps, rest| format_call(ps, rest) },
    :const_path_ref => lambda { |ps, rest| format_const_path_ref(ps, rest) },
    :@ident => lambda { |ps, rest| format_ident(ps, rest) },
    :symbol_literal => lambda { |ps, rest| format_symbol_literal(ps, rest) },
    :command_call => lambda { |ps, rest| format_command_call(ps, rest) },
    :const_ref => lambda { |ps, rest| format_const_ref(ps, rest) },
    :"@const" => lambda { |ps, rest| format_const(ps, rest) },
    :defs => lambda { |ps, rest| format_defs(ps, rest) },
    :@kw => lambda { |ps, rest| format_kw(ps, rest) },
    :bodystmt => lambda { |ps, rest| format_bodystmt(ps, rest) },
    :if_mod => lambda { |ps, rest| format_if_mod(ps, rest) },
    :unless_mod => lambda { |ps, rest| format_unless_mod(ps, rest) },
    :if => lambda { |ps, rest| format_if(ps, rest) },
    :opassign => lambda { |ps, rest| format_opassign(ps, rest) },
    :var_field => lambda { |ps, rest| format_var_field(ps, rest) },
    :@ivar => lambda { |ps, rest| format_ivar(ps, rest) },
    :top_const_ref => lambda { |ps, rest| format_top_const_ref(ps, rest) },
    :super => lambda { |ps, rest| format_super(ps, rest) },
    :zsuper => lambda { |ps, rest| format_zsuper(ps, rest) },
    :array => lambda { |ps, rest| format_array(ps, rest) },
    :unary => lambda { |ps, rest| format_unary(ps, rest) },
    :paren => lambda { |ps, rest| format_paren(ps, rest) },
    :string_concat => lambda { |ps, rest| format_string_concat(ps, rest) },
    :unless => lambda { |ps, rest| format_unless(ps, rest) },
    :begin => lambda { |ps, rest| format_begin(ps, rest) },
    :brace_block => lambda { |ps, rest| format_brace_block(ps, rest) },
    :@float => lambda { |ps, rest| format_float(ps, rest) },
    :ifop => lambda { |ps, rest| format_ifop(ps, rest) },
    :hash => lambda { |ps, rest| format_hash(ps, rest) },
    :aref_field => lambda { |ps, rest| format_aref_field(ps, rest) },
    :aref => lambda { |ps, rest| format_aref(ps, rest) },
    :args_add_block => lambda { |ps, rest| format_args_add_block(ps, rest) },
    :bare_assoc_hash => lambda { |ps, rest| format_bare_assoc_hash(ps, rest) },
    :defined => lambda { |ps, rest| format_defined(ps, rest) },
    :until => lambda { |ps, rest| format_until(ps, rest) },
    :return0 => lambda { |ps, rest| format_return0(ps, rest) },
    :massign => lambda { |ps, rest| format_massign(ps, rest) },
    :yield => lambda { |ps, rest| format_yield(ps, rest) },
    :regexp_literal => lambda { |ps, rest| format_regexp_literal(ps, rest) },
    :alias => lambda { |ps, rest| format_alias(ps, rest) },
    :field => lambda { |ps, rest| format_field(ps, rest) },
    :mrhs_new_from_args => lambda { |ps, rest| format_mrhs_new_from_args(ps, rest) },
    :dot2 => lambda { |ps, rest| format_dot2(ps, rest) },
    :yield0 => lambda { |ps, rest| format_yield0(ps, rest) },
    :@op => lambda { |ps, rest| format_op(ps, rest) },
    :case => lambda { |ps, rest| format_case(ps, rest) },
    :@gvar => lambda { |ps, rest| format_gvar(ps, rest) },
    :sclass => lambda { |ps, rest| format_sclass(ps, rest) },
    :retry => lambda { |ps, rest| format_retry(ps, rest) }
  }.fetch(type).call(ps, rest)
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
  metadata = Struct.new(:comment_blocks)
  comment_blocks = {}

  file_data.split("\n").each_with_index do |line, index|
    comment_blocks[index] = line if /^ *#/ === line
  end

  metadata.new(comment_blocks)
end

class Parser < Ripper::SexpBuilderPP
  ARRAY_SYMBOLS = {
    qsymbols: '%i',
    qwords: '%w',
    symbols: '%I',
    words: '%W'
  }.freeze

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
end

def main
  file_data = File.read(FILE)
  file_data = file_data.gsub("\r\n", "\n")

  line_metadata = extract_line_metadata(file_data)
  sexp = Parser.new(file_data).parse
  format_program(line_metadata, sexp, $stdout)
end

main
