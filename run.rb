require 'ripper'
require 'stringio'
require 'pp'

FILE = ARGV[0]
MODE = :inline

class Line
  attr_reader :parts

  def initialize(parts)
    @parts = parts
  end

  def to_s
    parts.map { |x| x.to_s }.join("")
  end

  def strip_trailing_newlines
    while parts.last == "\n"
      parts.pop
    end
  end

  def remove_redundant_indents
    parts.shift if parts[0] == ""
  end

  def ends_with_newline?
    parts.last == "\n"
  end

  def is_only_a_newline?
    parts == ["\n"]
  end

  def contains_end?
    parts.any? { |x| x == "end" }
  end

  def contains_def?
    parts.any? { |x| x == :def }
  end

  def contains_do?
    parts.any? { |x| x == :do }
  end

  def declares_private?
    parts.any? { |x| x == "private" } && parts.length == 3
  end

  def declares_require?
    parts.any? { |x| x == "require" }
  end
end

def want_blankline?(line, next_line)
  return unless next_line
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.contains_def?
  return true if line.declares_private?
  return true if line.declares_require? && !next_line.declares_require?
end

class ParserState
  attr_accessor :depth, :start_of_line, :line

  def initialize(result, comments_hash)
    @result = result
    @depth_stack = [0]
    @start_of_line = [true]
    @render_queue = []
    @line = []
    @current_orig_line_number = 0
    @comments_hash = comments_hash
    @conditional_indent = [0]
  end

  def on_line(line_number)
    while !comments_hash.empty? && comments_hash.keys.sort.first < line_number
      key = comments_hash.keys.sort.first
      @render_queue << [comments_hash.delete(key), "\n"]
    end

    @current_orig_line_number = line_number
  end

  def write
    clear_empty_trailing_lines

    lines = render_queue.map { |item| Line.new(item) }
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

  def emit_else
    line << "else"
  end

  def emit_elsif
    line << "elsif"
  end

  def push_conditional_indent
    if self.line.empty?
      @conditional_indent << 2*@depth_stack.last
    else
      @conditional_indent << line.join("").length
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
    build = params_list.map { |x|
      raise "got a non ident param" if x[0] != :"@ident"
      x[1]
    }.join(", ")
    line << build
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

  def emit_assignment(variable)
    emit_indent if start_of_line.last
    line << "#{variable} = "
  end

  def emit_newline
    line << "\n"
    render_queue << line
    self.line = []
  end

  def emit_dot
    line << "."
  end

  def emit_ident(ident)
    line << ident
  end

  def emit_equals
    line << "="
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
    start_of_line << true

    blk.call

    start_of_line.pop
    depth_stack[-1] -= 1
  end

  def emit_open_block_arg_list
    line << "|"
  end

  def emit_close_block_arg_list
    line << "|"
  end

  def emit_string_literal(literal)
    line << literal.inspect
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

def format_params_list(ps, params_list)
  ps.emit_open_paren
  ps.emit_params_list(params_list)
  ps.emit_close_paren
end

def format_block_params_list(ps, params_list)
  ps.emit_open_block_arg_list
  ps.emit_params_list(params_list)
  ps.emit_close_block_arg_list
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

  if params[1] != nil
    params_list = params[1][1]
    format_params_list(ps, params_list)
  end

  ps.emit_newline
  ps.new_block do
    format_expression(ps, body)
  end

  ps.emit_end
  ps.emit_newline
  ps.emit_newline
end

def format_void_expression(ps, rest)
end

def format_opassign(ps, rest)
  head, op, tail = rest
  format_expression(ps, head)
  ps.emit_space

  pp op

  raise "omg"
end

def format_assign_expression(ps, rest)
  raise "got something other than var field in assignment" unless rest[0][0] == :var_field
  head, tail = rest
  format_expression(ps, head)
  ps.emit_space
  ps.emit_equals
  ps.emit_space

  ps.start_of_line << false
  format_expression(ps, tail)
  ps.start_of_line.pop

  ps.emit_newline
end

def format_method_add_block(ps, rest)
  raise "got something other than call in method_add_block" unless rest[0][0] == :call
  ps.emit_indent
  call_rest = rest[0][1...rest[0].length]
  ps.start_of_line << false
  format_call(ps, call_rest)
  ps.emit_space

  # rest[1] is a do_block or a curly block, which are both expressions
  format_expression(ps, rest[1])
  ps.start_of_line.pop

  ps.emit_newline
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
end

def format_binary(ps, rest)
  ps.indent if ps.start_of_line.last

  ps.start_of_line << false
  format_expression(ps, rest[0])
  ps.emit_binary("#{rest[1].to_s}")
  format_expression(ps, rest[2])
  ps.start_of_line.pop
end

def format_do_block(ps, rest)
  ps.start_of_line

  ps.emit_do

  if rest[0] != nil
    ps.emit_space
    format_block_params_list(ps, rest[0][1][1])
  end

  ps.emit_newline

  ps.new_block do
    rest[1].each do |expr|
      format_expression(ps, expr)
    end
  end

  ps.start_of_line << true
  ps.emit_end
  ps.start_of_line.pop
end

def format_method_add_arg(ps, rest)
  type, call_rest = rest[0], rest[1...rest.length]

  ps.emit_indent if ps.start_of_line.last

  ps.start_of_line << false
  format_expression(ps, type)
  ps.start_of_line.pop

  raise "got call rest longer than one" if call_rest.length > 1
  args_list = call_rest[0]
  if args_list[0] == :arg_paren
    args_list = args_list[1]
  elsif args_list[0] == :args_add_block
  else
    raise "got non call paren args list"
  end
  raise "got non args list" if args_list[0] != :args_add_block

  format_args_add_block(ps, args_list)
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
  format_args_add_block(ps, args_list)
  ps.emit_newline if ps.start_of_line.last
end

def format_vcall(ps, rest)
  raise "didn't get exactly one part" if rest.count != 1
  raise "didn't get an ident" if rest[0][0] != :"@ident"

  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0][1])
  ps.emit_newline if ps.start_of_line.last
end

def format_string_literal(ps, rest)
  raise "didn't get exactly one part" if rest.count != 1
  raise "didn't get string content" if rest[0][0] != :string_content
  raise "didn't get tstring content" if rest[0][1][0] != :"@tstring_content"

  ps.emit_indent if ps.start_of_line.last

  ps.emit_string_literal(rest[0][1][1])
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
  ps.emit_newline

  if rest[1] != nil
      raise "got inheritance?"
  end

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

  raise "got non dot middle" if dot != :"."

  ps.start_of_line << false
  format_expression(ps, front)
  ps.emit_dot
  format_expression(ps, back)
  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_ident(ps, ident)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(ident[0])
end

def format_symbol_literal(ps, literal)
  raise "didn't get ident in right position" if literal[0][1][0] != :"@ident"
  ps.emit_symbol(literal[0][1][1])
end

def format_command_call(ps, expression)
  left, dot, right, args = expression
  ps.start_of_line << false

  format_expression(ps, left)
  raise "got something other than a dot" if dot != :"."
  ps.emit_dot
  format_expression(ps, right)
  format_args_add_block(ps, args)

  ps.start_of_line.pop
  ps.emit_newline if ps.start_of_line.last
end

def format_args_add_block(ps, args_list)
  raise "got non args list" if args_list[0] != :args_add_block
  ps.emit_open_paren
  ps.start_of_line << false

  args_list[1].each do |expr|
    format_expression(ps, expr)
    ps.emit_comma_space unless expr == args_list[1].last
  end

  ps.emit_close_paren

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
  ps.emit_const(expression[0])
end

def format_defs(ps, rest)
  head, period, tail, params, body = rest
  ps.emit_ident("def")
  ps.emit_space
  ps.start_of_line << false
  format_expression(ps, head)
  ps.emit_dot
  format_expression(ps, tail)

  if params[1] != nil
    params_list = params[1][1]
    format_params_list(ps, params_list)
  end
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

def format_bodystmt(ps, rest)
  expressions = rest[0]
  if rest[1...-1].any? {|x| x != nil }
    raise "got something other than a nil in a format body statement"
  end
  expressions.each do |line|
    format_expression(ps, line)
  end
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
  type = further_conditionals[0]
  case type
  when :else
    _, body = further_conditionals
    ps.emit_indent
    ps.emit_else
    ps.emit_newline
    ps.start_of_line << true

    ps.new_block do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
    ps.start_of_line.pop
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
  else
    raise "didn't get a known type in format conditional parts"
  end
end

def format_if(ps, expression)
  ps.push_conditional_indent
  if_conditional, body, further_conditionals = expression[0], expression[1], expression[2]

  ps.emit_indent if ps.start_of_line.last
  ps.start_of_line << false
  ps.emit_ident("if")
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
  format_conditional_parts(ps, further_conditionals)

  ps.emit_end
  ps.start_of_line.pop

  ps.pop_conditional_indent
  ps.emit_newline
end

def format_var_field(ps, rest)
  raise "didn't get exactly one thing" if rest.length != 1
  format_expression(ps, rest[0])
end

def format_ivar(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0])
end

def format_expression(ps, expression)
  type, rest = expression[0],expression[1...expression.length]
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
  }.fetch(type).call(ps, rest)
end

def format_program(comments_hash, sexp, result)
  program, expressions = sexp
  ps = ParserState.new(result, comments_hash)
  expressions.each do |expression|
    format_expression(ps, expression)
  end
ensure
  ps.write
end

def build_comments_hash(file_data)
  comment_blocks = {}
  file_data.split("\n").each_with_index do |line, index|
    if /^ *#/ === line
      comment_blocks[index] = line
    end
  end

  comment_blocks
end

def main
  file_data = File.read(FILE)
  file_data = file_data.gsub("\r\n", "\n")

  comments_hash = build_comments_hash(file_data)
  sexp = Ripper.sexp(file_data)
  format_program(comments_hash, sexp, $stdout)
end

main
