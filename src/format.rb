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

  ps.new_block do
    (expressions || []).each do |expr|
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

  ps.with_formatting_context(:def) do
    ps.new_block do
      format_expression(ps, body)
    end
  end

  ps.emit_end
  ps.emit_newline
end

def emit_params_separator(ps, index, length)
  if index != length - 1
    ps.emit_comma
    ps.emit_soft_newline
  end
end

def format_required_params(ps, required_params)
  return if required_params.empty?

  ps.with_start_of_line(false) do
    required_params.each_with_index do |expr, index|
      ps.emit_soft_indent
      format_expression(ps, expr)
      emit_params_separator(ps, index, required_params.length)
    end
  end
end

def format_optional_params(ps, optional_params)
  ps.with_start_of_line(false) do
    optional_params.each_with_index do |param, index|
      ps.emit_soft_indent
      left, right = param
      format_expression(ps, left)
      ps.emit_ident(" = ")
      format_expression(ps, right)
      emit_params_separator(ps, index, optional_params.length)
    end
  end
end

def format_kwargs(ps, kwargs)
  return if kwargs.empty?

  kwargs.each_with_index do |kwarg, index|
    label, false_or_expr = kwarg
    raise "got non label in kwarg" if label[0] != :@label
    ps.emit_soft_indent
    ps.emit_ident(label[1])

    ps.with_start_of_line(false) do
      if false_or_expr
        ps.emit_space
      end

      format_expression(ps, false_or_expr) if false_or_expr
    end

    emit_params_separator(ps, index, kwargs.length)
  end
end

def format_rest_params(ps, rest_params)
  return if rest_params == 0 || rest_params.empty? || rest_params == [:excessed_comma]
  ps.emit_soft_indent
  ps.emit_ident("*")

  if !rest_params[1].nil?
    rest_param, expr = rest_params
    raise "got bad rest_params" if rest_param != :rest_param

    ps.with_start_of_line(false) do
      format_expression(ps, expr)
    end
  end
end

def format_kwrest_params(ps, kwrest_params)
  return if kwrest_params.empty?
  ps.emit_soft_indent
  ps.emit_ident("**")

  if !kwrest_params[1].nil?
    if kwrest_params[0] == :kwrest_param
      _, expr = kwrest_params
    else
      expr = kwrest_params
    end

    ps.with_start_of_line(false) do
      format_expression(ps, expr)
    end
  end
end

def format_blockarg(ps, blockarg)
  return if blockarg.empty?
  _, expr = blockarg
  ps.emit_soft_indent

  ps.with_start_of_line(false) do
    ps.emit_ident("&")
    format_expression(ps, expr)
  end
end

def format_params(ps, params, open_delim, close_delim)
  return if params.nil?
  f_params = []

  # this deals with params lists like:
  #
  # def foo(a,b,c)
  # do |c,e,d|
  # in that the first is wrapped with :paren and the second is wrapped with
  # block_var
  if params[0] == :paren || params[0] == :block_var

    # this implements support for block local variables. That's where you
    # have syntax in a block like:
    # do |a;x,y,z|
    # x, y, and z are local variables in the called block that are hermetic
    # and cannot write variables in the closure outside
    if params[0] == :block_var && params[-1] != nil
      f_params = params[-1]
    end

    # in both these cases, the params list is more complicated than just being
    # a params list, but the thing we actually want is in the 1th position
    params = params[1]
  end

  have_any_params = params[1..-1].any? { |x| !x.nil? } || !f_params.empty?
  return unless have_any_params

      # this is the "bad params" detector, we've not yet experienced non nil
      # positions in 5 and 7 despite having thrown a lot of stuff at rubyfmt
      # so I'm not really sure what these do
  ps.breakable_of(open_delim, close_delim) do
    ps.breakable_entry do
      bad_params = params[7..-1].any? { |x| !x.nil? }
      bad_params = false if params[5]
      bad_params = false if params[7]
      raise "dont know how to deal with a params list" if bad_params

      # def foo(a, b=nil, *args, d, e:, **kwargs, &blk)
      #         ^  ^___^  ^___^  ^  ^    ^_____^   ^
      #         |    |      |    |  |      |       |
      #         |    |      |    |  |      |    block_arg
      #         |    |      |    |  |      |
      #         |    |      |    |  |  kwrest_params
      #         |    |      |    |  |
      #         |    |      |    | kwargs
      #         |    |      |    |
      #         |    |      | more_required_params
      #         |    |      |
      #         |    |  rest_params
      #         |    |
      #         | optional params
      #         |
      #     required params
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

          # we don't actually have a test case for [:excessed_comma] lmao, but
          # it's definitely in parse.y
        have_more = emission_order[idx + 1..-1].map { |x| x[0] != 0 && !x[0].empty? && x[0] != [:excessed_comma] }.any?

        if did_emit && have_more
          ps.emit_comma
          ps.emit_soft_newline
        end
      end

      if f_params && !f_params.empty?
        ps.emit_ident(";")

        ps.with_start_of_line(false) do
          format_list_like_thing_items(ps, [f_params], true)
        end
      end

      ps.emit_collapsing_newline
    end
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

    ps.with_formatting_context(:assign) do
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
  int = rest[0]
  ps.emit_int(int)
  ps.emit_newline if ps.start_of_line.last
end

def format_rational(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  ps.emit_ident(rest[0])
  ps.emit_newline if ps.start_of_line.last
end

def format_imaginary(ps, rest)
  ps.emit_indent if ps.start_of_line.last
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

  ps.with_formatting_context(:binary) do
    ps.with_start_of_line(false) do
      format_expression(ps, rest[0])
      ps.emit_binary("#{rest[1].to_s}")
      format_expression(ps, rest[2])
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_do_block(ps, rest)
  raise "got bad block #{rest.inspect}" if rest.length != 2
  params, body = rest
  ps.emit_do
  format_params(ps, params, " |", "|")
  ps.emit_newline

    # in ruby 2.5 blocks are bodystmts because blocks support
    # ```
    # foo do
    # rescue
    # end
    # ```
    #
    # style rescues now
  ps.new_block do
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
      on_line_skip = type == :heredoc && parts[idx + 1] && parts[idx + 1][0] == :@tstring_content && parts[idx + 1][1].start_with?("\n")

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

  ps.emit_newline if ps.start_of_line.last
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

  ps.with_start_of_line(false) do
    ps.emit_space
    format_expression(ps, module_name)
  end

  ps.emit_newline

  ps.new_block do
    exprs = rest[1][1]

    ps.with_formatting_context(:class_or_module) do
      exprs.each do |expr|
        format_expression(ps, expr)
      end
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

    ps.with_start_of_line(false) do
      format_expression(ps, rest[1])
    end
  end

  ps.emit_newline
  ps.on_line(ps.current_orig_line_number + 1)

  ps.new_block do
    exprs = rest[2][1]

    ps.with_formatting_context(:class_or_module) do
      exprs.each do |expr|
        format_expression(ps, expr)
      end
    end
  end

  ps.emit_end
  ps.emit_newline if ps.start_of_line.last
end

def have_empty_exprs?(exprs)
  (exprs.empty? || exprs.first.nil? || (exprs[0] == [:void_stmt] && exprs.length == 1))
end

def format_const_path_ref(ps, rest)
  expr, const = rest

  ps.with_start_of_line(false) do
    format_expression(ps, expr)
    ps.emit_double_colon
    raise "cont a non const" if const[0] != :"@const"
    ps.emit_const(const[1])
  end

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
  candidate == :"&." || [candidate.is_a?(Array), candidate[0] == :@op, candidate[1] == "&."].all?
end

def format_list_like_thing_items(ps, args_list, single_line)
  return false if args_list.nil?
  emitted_args = false

  args_list[0].each_with_index do |expr, idx|
    raise "this is bad" if expr[0] == :tstring_content

    if single_line
      format_expression(ps, expr)
      ps.emit_comma_space unless idx == args_list[0].count - 1
    else
      ps.emit_soft_indent

      ps.with_start_of_line(false) do
        format_expression(ps, expr)
        ps.emit_comma
        ps.emit_soft_newline
      end
    end

    emitted_args = true
  end

  emitted_args
end

# format_list_like_thing takes any inner construct (like array items, or an
# args list, and formats them).
def format_list_like_thing(ps, args_list, single_line = true)
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
      ps.emit_soft_indent
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

      ps.emit_comma
      ps.emit_soft_newline
    end
  end

  emitted_args
end

def emit_intermediate_array_separator(ps, single_line)
  ps.emit_comma
  ps.emit_soft_newline
  ps.emit_soft_indent
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

  ps.with_start_of_line(false) do
    emitted_args = format_list_like_thing(ps, args_list)

    if args_list[1]
      ps.emit_comma_space if emitted_args
      ps.emit_ident("&")
      format_expression(ps, args_list[1])
    end

    ps.emit_close_paren unless surpress_paren
  end
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
  ps.emit_def_keyword
  ps.emit_space

  ps.with_start_of_line(false) do
    format_expression(ps, head)
    ps.emit_dot
    format_expression(ps, tail)
    format_params(ps, params, "(", ")")
    ps.emit_newline
  end

  ps.with_formatting_context(:def) do
    ps.new_block do
      format_expression(ps, body)
    end
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

        # if this is a multiple rescue like
        # rescue *a, b
        # this will be a mrhs_new_from_args
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

def format_bodystmt(ps, rest, inside_begin = false)
  expressions = rest[0]
  rescue_part = rest[1]
  else_part = rest[2]
  ensure_part = rest[3]

  if rest[4..-1].any? { |x| x != nil }
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

  ps.with_start_of_line(false) do
    format_expression(ps, guarded_expression)
    ps.emit_space
    ps.emit_ident(conditional_type)
    ps.emit_space
    format_expression(ps, conditional)
  end

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

    ps.with_start_of_line(false) do
      format_expression(ps, cond)
    end

    ps.emit_newline

    ps.new_block do
      body.each do |expr|
        format_expression(ps, expr)
      end
    end

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
    ps.emit_keyword(kind)
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
    format_list_like_thing(ps, [args_list], single_line = true)
  when Symbol
    format_expression(ps, args_list) unless args_list.empty?
  else
    format_list_like_thing(ps, [args_list], single_line = true)
  end
end

def format_array_fast_path(ps, rest)
  if rest == [nil]
    ps.emit_open_square_bracket
    ps.emit_close_square_bracket
  else

    ps.breakable_of("[", "]") do
      ps.breakable_entry do
        format_list_like_thing(ps, rest, false)
      end
    end
  end
end

def format_array(ps, rest)
  ps.emit_indent if ps.start_of_line.last

  if Parser.is_percent_array?(rest)
    ps.emit_ident(Parser.percent_symbol_for(rest))
    ps.emit_open_square_bracket

    ps.with_start_of_line(false) do
      parts = rest[0][1]

      parts.each.with_index do |expr, index|
        expr = [expr] if expr[0] == :@tstring_content
        format_inner_string(ps, expr, :array)
        ps.emit_space if index != parts.length - 1
      end
    end

    ps.emit_close_square_bracket
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

  ps.with_start_of_line(false) do
    format_expression(ps, tail)
  end

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
    format_bodystmt(ps, begin_body[1..-1], inside_begin = true)
  end

  ps.with_start_of_line(true) do
    ps.emit_end
    ps.emit_newline
  end
end

def format_brace_block(ps, expression)
  raise "didn't get right array in brace block" if expression.length != 2
  params, body = expression
  output = StringIO.new
  next_ps = ParserState.with_depth_stack(output,   from: ps)

  ps.new_block do
    ps.with_formatting_context(:curly_block) do
      body.each do |expr|
        format_expression(next_ps, expr)
      end
    end
  end

  next_ps.write
  output.rewind
  multiline = output.read.strip.include?("\n")
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
end

def format_float(ps, expression)
  ps.emit_indent if ps.start_of_line.last
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

def format_assocs(ps, assocs, newlines = false)
  assocs.each_with_index do |assoc, idx|
    ps.emit_soft_indent

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
        ps.emit_comma
        ps.emit_soft_newline
      elsif idx != assocs.length - 1
        ps.emit_comma
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

    ps.breakable_of("{", "}") do
      ps.breakable_entry do
        format_assocs(ps, assocs, true)
      end
    end

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
    ps.emit_open_square_bracket
    ps.surpress_one_paren = true
    format_expression(ps, sqb_args)
    ps.emit_close_square_bracket
  end
end

def format_aref(ps, expression)
  raise "got bad aref" if expression.length != 2
  expression, sqb_args = expression
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    format_expression(ps, expression)
    ps.emit_open_square_bracket
    ps.surpress_one_paren = true
    format_inner_args_list(ps, sqb_args) if sqb_args
    ps.emit_close_square_bracket
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
  ps.emit_return
  ps.emit_newline if ps.start_of_line.last
end

def format_massign(ps, expression)
  ps.emit_indent if ps.start_of_line.last

  ps.with_start_of_line(false) do
    assigns, rhs = expression

    if assigns[0] == :mlhs_add_star
      assigns, last = [assigns[1], assigns[2]]
      item = last
      assigns << [:rest_param, item]
    end

    assigns.each_with_index do |assign, index|
      format_expression(ps, assign)
      ps.emit_comma_space if index != assigns.length - 1
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
  parts, re_end = expression
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

  ps.with_start_of_line(false) do
    format_expression(ps, front)
    format_dot(ps, [dot])
    format_expression(ps, back)
  end

  ps.emit_newline if ps.start_of_line.last
end

def format_mrhs_new_from_args(ps, expression)
  ps.emit_indent if ps.start_of_line.last
  parts, tail = expression

  ps.with_start_of_line(false) do
    ps.breakable_of("", "") do
      ps.breakable_entry do
        format_list_like_thing(ps, [parts], false)

        if tail != nil && tail != []

          ps.with_start_of_line(false) do
            format_expression(ps, tail)
          end

          ps.emit_comma
          ps.emit_soft_newline
        end
      end
    end
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
      ps.breakable_of("", "") do
        ps.breakable_entry do
          format_list_like_thing(ps, [conditional], false)
        end
      end
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

  buf = StringIO.new
  render = ParserState.with_depth_stack(buf,   from: ps)

  while_exprs.each do |while_expr|
    format_expression(render, while_expr)
  end

  render.write
  buf.rewind
  data = buf.read

  ps.with_start_of_line(false) do
    if data.count("\n") > 1
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
    expressions.each_with_index do |expression, idx|
      ps.with_start_of_line(true) do
        format_expression(ps, expression)
      end

      ps.emit_newline if idx != expressions.length - 1
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
    ps.emit_space
    ps.emit_rescue
    ps.emit_space
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

def use_parens_for_method_call(method, chain, args, original_used_parens, context)
  return false if method[1].start_with?("attr_") && context == :class_or_module

  # Always use parens for the shorthand `foo::()` syntax
  return true if method == :call

  # Never use parens for some methods and keywords
  return false if ["return", "raise"].include?(method[1])

  # Follow the original code style for super and yield
  # Note that `super()` has different semantics to `super`
  return original_used_parens if ["super", "yield", "require"].include?(method[1])

  # No parens if there are no arguments
  return false if args.empty?
  ci = chain.inspect
  return true if ci.include?(":@const") && method[1] == "new"
  return false if context == :class_or_module && !original_used_parens

  # If in doubt, use parens
  true
end

def format_method_call(ps, rest)
  ps.emit_indent if ps.start_of_line.last
  chain, method, original_used_parens, args = rest
  use_parens = use_parens_for_method_call(method, chain, args, original_used_parens, ps.formatting_context.last)

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

    ps.with_formatting_context(:args_list) do
      format_list_like_thing_items(ps, [args], true)
    end

    if use_parens
      ps.emit_ident(")")
    end
  end

  ps.emit_newline if ps.start_of_line.last
end

def normalize_method_add_arg(rest)
  [:method_call, *normalize_inner_call(rest[0]), true, normalize_args(rest[1])]
end

def normalize_command(rest)
  [:method_call, [], rest[0], false, normalize_args(rest[1])]
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
  [:method_call, *normalize_inner_call([:call, *rest]), false, []]
end

def normalize_vcall(rest)
  [:method_call, [], rest[0], false, []]
end

def normalize_zsuper(rest)
  [:method_call, [], [:keyword, "super"], false, []]
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
  :dot => method(:format_dot),
  :method_call => method(:format_method_call),
  :splat => method(:format_splat),
  :to_proc => method(:format_to_proc),
  :keyword => method(:format_keyword),
}.freeze

def format_expression(ps, expression)
  expression = normalize(expression)
  type, rest = expression[0], expression[1...expression.length]
  line_re = /(\[\d+, \d+\])/
  line_number = line_re.match(rest.inspect)

  if line_number != nil
    line_number = line_number.to_s.split(",")[0].gsub("[", "").to_i
    ps.on_line(line_number)
  end

  EXPRESSION_HANDLERS.fetch(type).call(ps, rest)
rescue KeyError => e
  puts(ps.current_orig_line_number)
  puts(ps.line)
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
