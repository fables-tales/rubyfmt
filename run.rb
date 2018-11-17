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
end

def want_blankline?(line, next_line)
  return unless next_line
  return true if line.contains_end? && !next_line.contains_end?
  return true if next_line.contains_do? && !line.contains_def?
end

class ParserState
  attr_accessor :depth, :start_of_line, :line

  def initialize(result)
    @result = result
    @depth = 0
    @start_of_line = [true]
    @render_queue = []
    @line = []
  end

  def write
    while render_queue.last == []
      render_queue.pop
    end

    while render_queue.last == ["\n"]
      render_queue.pop
    end

    lines = render_queue.map { |item| Line.new(item) }

    line = lines.first
    next_index = 1
    while next_index < lines.length
      if line.ends_with_newline? && lines[next_index] && lines[next_index].is_only_a_newline?
        lines.delete_at(next_index)
      end

      line = lines[next_index]
      next_index += 1
    end

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

    lines.each do |line|
      line.remove_redundant_indents
    end

    lines.each_with_index do |line, i|
      if i == render_queue.length-1
        line.strip_trailing_newlines
      end

      result.write(line)
    end

    result.write("\n")
    result.flush
  end

  def emit_indent
    line << " " * (2 * depth)
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
    emit_indent if start_of_line.last
    line << ident
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
    self.depth += 1
    self.start_of_line << true

    blk.call

    self.start_of_line.pop
    self.depth -= 1
  end

  def emit_open_block_arg_list
    line << "|"
  end

  def emit_close_block_arg_list
    line << "|"
  end

  private

  attr_reader :result, :render_queue
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
  def_name = rest[0][1]
  params = rest[1]

  body_expressions = rest[2][1]

  ps.emit_indent
  ps.emit_def(def_name)

  if params[1] != nil
    params_list = params[1][1]
    format_params_list(ps, params_list)
  end

  ps.emit_newline
  ps.new_block do
    body_expressions.each do |expression|
      format_expression(ps, expression)
    end
  end

  ps.emit_end
  ps.emit_newline
  ps.emit_newline
end

def format_void_expression(ps, rest)
end

def format_assign_expression(ps, rest)
  raise "got something other than var field in assignment" unless rest[0][0] == :var_field
  variable = rest[0][1][1]
  expression = rest[1]

  ps.emit_assignment(variable)

  ps.start_of_line << false
  format_expression(ps, expression)
  ps.start_of_line.pop

  ps.emit_newline
end

def format_call(ps, rest)
  raise "didn't get a dot symbol in call" unless rest[1] == :"."
  raise "didn't get ident in call" unless rest[2][0] == :"@ident"

  ident = rest[2][1]
  format_expression(ps, rest[0])
  ps.start_of_line << false
  ps.emit_dot
  ps.emit_ident(ident)
  ps.start_of_line.pop
end

def format_method_add_block(ps, rest)
  raise "got something other than call in method_add_block" unless rest[0][0] == :call
  call_rest = rest[0][1...rest[0].length]
  format_call(ps, call_rest)
  ps.emit_space

  ps.start_of_line << false
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

  ident = rest[0][1]

  params = if rest[1][0] == :arg_paren
             rest[1][1]
           else
             rest[1]
           end
  build = [ident, params]

  format_command(ps, build)
end

def format_command(ps, rest)
  ident = rest[0]
  raise "got non ident in the first position" if rest[0][0] != :"@ident"
  ps.emit_ident(ident[1])

  ps.emit_open_paren
  ps.start_of_line << false

  args_list = rest[1]
  raise "got non args list" if args_list[0] != :args_add_block

  args_list[1].each do |expr|
    format_expression(ps, expr)
  end

  ps.start_of_line.pop
  ps.emit_close_paren
end

def format_vcall(ps, rest)
  raise "didn't get exactly one part" if rest.count != 1
  raise "didn't get an ident" if rest[0][0] != :"@ident"

  ps.emit_ident(rest[0][1])
  ps.emit_newline
end

def format_expression(ps, expression)
  type, rest = expression[0],expression[1...expression.length]
  {
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
  }.fetch(type).call(ps, rest)
end

def format_program(sexp, result)
  program, expressions = sexp
  ps = ParserState.new(result)
  expressions.each do |expression|
    format_expression(ps, expression)
  end
ensure
  ps.write
end

def main
  sexp = Ripper.sexp(File.read(FILE))
  format_program(sexp, $stdout)
end

main
