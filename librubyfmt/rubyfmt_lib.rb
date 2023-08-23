module TrackAllScannerEvents
  Ripper::SCANNER_EVENTS.reject { |x| x == :sp || x == :nl || x == :ignored_nl }.each do |se|
    define_method(:"on_#{se}") do |*args|
      @lines_with_any_ruby[lineno] = true
      super(*args)
    end
  end
end

class Parser < Ripper::SexpBuilderPP
  ARRAY_SYMBOLS = {qsymbols: "%i", qwords: "%w", symbols: "%I", words: "%W"}.freeze

  def self.is_percent_array?(rest)
    return false if rest.nil?
    return false if rest[0].nil?
    ARRAY_SYMBOLS.include?(rest[0][0])
  end

  def self.percent_symbol_for(rest)
    ARRAY_SYMBOLS[rest[0][0]]
  end

  include TrackAllScannerEvents

  def initialize(file_data)
    super(file_data)
    @file_lines = file_data.lines

    @lines_with_any_ruby = {}

    # heredoc stack is the stack of identified heredocs
    @heredoc_stack = []

    # next_heredoc_stack is the type identifiers of the next heredocs, that
    # we haven't emitted yet
    @next_heredoc_stack = []
    @heredoc_regex = /(<<[-~]?)(.*$)/
    @regexp_stack = []
    @embexpr_stack = []
    @string_stack = []
    @kw_stacks = {
      "do" => [],
      "ensure" => [],
      "next" => [],
      "return" => [],
      "when" => [],
      "case" => [],
      "yield" => [],
      "break" => [],
      "super" => [],
      "retry" => [],
      "redo" => [],
      "rescue" => [],
      "begin" => [],
      "else" => [],
      "if" => [],
      "unless" => [],
      "elsif" => [],
      "while" => [],
      "until" => [],
    }
    @op_locations = []
    @tlambda_stack = []
    @array_location_stacks = []
    @rbracket_stack = []
    @lbrace_stack = []
    @comments = {}
    # binary contents comming after a `__END__` node
    @data_contents_start_line = nil
  end

  # This method has incorrect behavior inside Ripper,
  # so we patch it for now
  # original: https://github.com/ruby/ruby/blob/118368c1dd9304c0c21a4437016af235bd9b8438/ext/ripper/lib/ripper/sexp.rb#L144-L155
  def on_heredoc_dedent(val, width)
    dedented_lines = []
    val.map! do |e|
      next e if e.is_a?(Symbol) && /_content\z/ =~ e

      if e.is_a?(Array) && e[0] == :@tstring_content && !dedented_lines.include?(e[2][0])
        e = dedent_element(e, width)
        dedented_lines << e[2][0]
      elsif e.is_a?(Array) && e[0] == :string_embexpr
        # String embexprs can also span multiple lines, but they don't need
        # any dedenting since they're not strings, so we should mark
        # any line they touch as dedented
        line_start, line_end = e.last
        (line_start..line_end).each { |line_number| dedented_lines << line_number }
      elsif String === e
        dedent_string(e, width)
      end
      e
    end
    val
  end

  def parse
    res = super

    if res == nil || error?
      nil
    else
      data_contents = if @data_contents_start_line
        @file_lines[@data_contents_start_line..].join
      else
        nil
      end

      [res, @comments, @lines_with_any_ruby, @file_lines.count, data_contents]
    end
  end

  DELIM_CLOSE_PAREN={ '{' => '}', '[' => ']', '(' => ')', '<' => '>' }

  def escape_percent_array_paren_content(part, pattern)
    return unless part[0] == :@tstring_content
    part[1].gsub!(pattern) do |str|
      str_last = str[-1]
      if str_last == ']' || str_last == '['
        # insert needed escape
        "#{str[0..-2]}\\#{str_last}"
      else
        # drop unnecessary escape
        "#{str[0..-3]}#{str_last}"
      end
    end
  end

  ARRAY_SYMBOLS.each do |event, symbol|
    define_method(:"on_#{event}_new") do
      # there doesn't seem to be an _end so _rubyfmt_delim is a hacky way
      # to get the value from _beg into parts for _add.
      # it's removed again in on_array.
      [event, [[:_rubyfmt_delim, @percent_array_stack]], [lineno, column]]
    end

    define_method(:"on_#{event}_beg") do |delim|
      @array_location_stacks << lineno
      @percent_array_stack = delim
    end

    define_method(:"on_#{event}_add") do |parts, part|
      delim = parts[1][0][1]

      unless delim.end_with?('[')
        delim_start = delim[-1]
        delim_close = DELIM_CLOSE_PAREN[delim_start]
        pattern = if delim_close
          /(?<!\\)(?:\\\\)*(?:\\[#{Regexp.escape(delim_start)}#{Regexp.escape(delim_close)}]|[\[\]])/
        else
          /(?<!\\)(?:\\\\)*(?:\\#{Regexp.escape(delim_start)}|[\[\]])/
        end
        if part[0].is_a?(Array)
          part.each do |sub_part|
            escape_percent_array_paren_content(sub_part, pattern)
          end
        else
          escape_percent_array_paren_content(part, pattern)
        end
      end
      parts[1] << part

      super(parts, part)
    end
  end

  def on_op(*_args)
    @op_locations << lineno
    super + [[lineno, lineno]]
  end

  def on_binary(left, operator, right)
    res = super
    op_location = @op_locations.pop
    res[2] = [res[2], [op_location, op_location]]
    res
  end

  def on_next(*_args)
    super + [start_end_for_keyword('next')]
  end

  def on_if(*_args)
    super + [start_end_for_keyword('if')]
  end

  def on_unless(*_args)
    super + [start_end_for_keyword('unless')]
  end

  def on_else(*_args)
    super + [start_end_for_keyword('else')]
  end

  def on_elsif(*_args)
    super + [start_end_for_keyword('elsif')]
  end

  def on_lbrace(*args)
    @lbrace_stack << lineno
  end

  def on_rbracket(*_args)
    @rbracket_stack << lineno
    super
  end

  def on_brace_block(params, body)
    start_line = @lbrace_stack.pop
    end_line = lineno
    super + [[start_line, end_line]]
  end

  def on_do_block(*args)
    super + [start_end_for_keyword('do')]
  end

  def on_block_var(*_args)
    with_lineno { super }
  end

  # In the case of mod statements, we've previously
  # pushed their lines onto the stack but now
  # don't need them, so we pop them off and ignore them

  def on_if_mod(*_args)
    # In this case, we don't use the line here, so just remove it
    @kw_stacks['if'].pop
    super
  end

  def on_unless_mod(*_args)
    # In this case, we don't use the line here, so just remove it
    @kw_stacks['unless'].pop
    super
  end

  def on_while_mod(*_args)
    @kw_stacks['while'].pop
    super
  end

  def on_until_mod(*_args)
    @kw_stacks['until'].pop
    super
  end

  def on_while(*args)
    super + [start_end_for_keyword('while')]
  end

  def on_until(*args)
    super + [start_end_for_keyword('until')]
  end

  def on_hash(assocs)
    [:hash, assocs, [@lbrace_stack.pop, lineno]]
  end

  def on_zsuper
    # ripper doesn't handle on_zsuper correctly.
    # however `on_kw` catches zsuper, so use that!
    [:zsuper, start_end_for_keyword('super')]
  end

  def on_yield0
    # ripper doesn't handle on_yield0 correctly.
    # however `on_kw` catches yield0, so use that!
    [:yield0, start_end_for_keyword('yield')]
  end

  def on_redo
    # ripper doesn't handle on_redo correctly.
    # however `on_kw` catches redo, so use that!
    [:redo, start_end_for_keyword('redo')]
  end

  def on_begin(*args)
    beg, statements = super
    [beg, start_end_for_keyword('begin'), statements]
  end

  def on_rescue(*args)
    super + [start_end_for_keyword('rescue')]
  end

  def on_ensure(*args)
    super + [start_end_for_keyword('ensure')]
  end

  def on_retry
    # ripper doesn't handle on_retry correctly.
    # however `on_kw` catches retry, so use that!
    [:retry, start_end_for_keyword('retry')]
  end

  def on_arg_paren(args_node)
    with_lineno { super }
  end

  def on_call(*_args)
    with_lineno { super }
  end

  def on_method_add_arg(*_args)
    with_lineno { super }
  end

  def on_paren(*_args)
    with_lineno { super }
  end

  def on_args_add_block(*_args)
    with_lineno { super }
  end

  def on_params(*_args)
    with_lineno { super }
  end

  def on_vcall(*_args)
    with_lineno { super }
  end

  def on_lbracket(*args)
    @array_location_stacks << lineno
  end

  def on_array(*args)
    res = super
    res[1][1].shift if (ary = res.dig(1, 1, 0)) && ary.is_a?(Array) && ary[0] == :_rubyfmt_delim # it's done its job
    res << [@array_location_stacks.pop, lineno]
    res
  end

  def on_aref(*_args)
    # This isn't needed, so we just remove it for tracking
    @array_location_stacks.pop
    # The lineno here is actually one line *after*
    # the line of the bracket, so we manually trace
    # the line of the rbracket instead
    super + [[@rbracket_stack.pop, column]]
  end

  def on_aref_field(*_args)
    # This isn't needed, so we just remove it for tracking
    @array_location_stacks.pop
    # The lineno here is actually one line *after*
    # the line of the bracket, so we manually trace
    # the line of the rbracket instead
    super + [[@rbracket_stack.pop, column]]
  end

  def on_kw(kw)
    if stack = @kw_stacks[kw]
      stack << lineno
    end
    super
  end

  def on_super(args)
    [:super, args, start_end_for_keyword('super')]
  end

  def on_return(args)
    [:return, args, start_end_for_keyword('return')]
  end

  def on_return0
    [:return0, start_end_for_keyword('return')]
  end

  def on_when(cond, body, tail)
    [:when, cond, body, tail, start_end_for_keyword('when')]
  end

  def on_case(cond, body)
    [:case, cond, body, start_end_for_keyword('case')]
  end

  def on_yield(arg)
    [:yield, arg, start_end_for_keyword('yield')]
  end

  def on_break(arg)
    [:break, arg, start_end_for_keyword('break')]
  end

  def on_tlambda(*args)
    @tlambda_stack << lineno
    super
  end

  def on_def(*args)
    with_lineno { super }
  end

  def on_defs(*args)
    with_lineno { super }
  end

  def on_class(*args)
    with_lineno { super }
  end

  def on_sclass(*args)
    with_lineno { super }
  end

  def on_module(*args)
    with_lineno { super }
  end

  def on_heredoc_beg(*args, &blk)
    heredoc_parts = @heredoc_regex.match(args[0]).captures
    raise "bad heredoc" unless heredoc_parts.select { |x| x != nil }.count == 2
    @next_heredoc_stack.push([heredoc_parts, [lineno]])
    super
  end

  def on_heredoc_end(*args, &blk)
    # Append current lineno to the current heredoc
    heredoc = @next_heredoc_stack.pop
    heredoc.last.push(lineno)
    @heredoc_stack.push(heredoc)
    super
  end

  def on_symbol_literal(*_args)
    with_lineno { super }
  end

  def on_string_literal(*args, &blk)
    if @heredoc_stack.last
      heredoc_parts = @heredoc_stack.pop
      args.insert(0, [:heredoc_string_literal] + heredoc_parts)
    else
      end_delim, end_line = @string_stack.pop
      start_delim, start_line = @string_stack.pop

      args << [start_line, end_line]

      clean_string_content(start_delim, end_delim, args[0])
    end

    super
  end

  def on_lambda(*args, &blk)
    args.insert(args.length, [@tlambda_stack.pop, lineno])
    [:lambda, *args]
  end

  def on_tstring_beg(*args, &blk)
    @string_stack << [args[0], lineno]
    super
  end

  def on_tstring_end(*args, &blk)
    @string_stack << [args[0], lineno]
    super
  end

  def on_embexpr_beg(*args)
    @embexpr_stack << [lineno]
    super
  end

  def on_embexpr_end(*args)
    # Append end line to make a StartEnd
    @embexpr_stack.last << lineno
    super
  end

  def on_string_embexpr(*args)
    super + [@embexpr_stack.pop]
  end

  def on_dyna_symbol(*args)
    # dyna_symbol expressions still end up calling
    # on_tstring_end, which will append the closing
    # quote to @string_stack. We want to ignore this,
    # so remove it from the stack.
    delim, start_line = @string_stack.pop
    res = super
    clean_string_content(delim, delim, res[1])
    res + [[start_line, lineno]]
  end

  def on_regexp_beg(re_part)
    @regexp_stack << re_part
  end


  def on_regexp_literal(*args)
    args[1] << @regexp_stack.pop
    super(*args)
  end

  def on_comment(comment)
    @comments[lineno] = comment
  end

  def on___end__(val)
    super
    @data_contents_start_line = lineno
  end

  private def start_end_for_keyword(keyword)
    [@kw_stacks[keyword].pop, lineno]
  end

  private def with_lineno(&blk)
    start_line = lineno
    res = yield
    end_line = lineno
    res + [[start_line, end_line]]
  end

  private def clean_string_content(start_delim, end_delim, string_contents)
    if start_delim && end_delim && start_delim != "\""
        if start_delim == "'" || start_delim.start_with?("%q")
          # re-evaluate the string with its own quotes to handle escaping.
          if string_contents[1]
            es = eval("#{start_delim}#{string_contents[1][1]}#{end_delim}")
            # did the original contain \u's?
            have_source_slash_u = string_contents[1][1].include?("\\u")
            # if all chars are unicode definitionally none of them are delimiters so we
            # can skip inspect
            have_all_unicode = es.chars.all? { |x| x.bytes.first >= 128 }

            if have_all_unicode && !have_source_slash_u
              "#{start_delim}#{string_contents[1][1]}#{end_delim}"
            else
              string_contents[1][1] = es.inspect[1..-2]
            end
            # Match at word boundaries and beginning/end of the string
            # so that things like `'\n'` correctly become `"\\n"`
            # instead of rendering as actual whitespace
            #
            # About this regex: `(?<!\\)` does a negative lookup for slashes
            # before the newline escape, which will only match instances
            # like `\\n` and not `\\\\n`
            string_contents[1][1].gsub!(/(?<!\\)\\n/, "\n")
            # This matches a special edge case where the last character on the line of a
            # single-quoted string is "\".
            string_contents[1][1].gsub!(/\\\\\\n/, "\\\\\\\n")
          end
        else
          # find delimiters after an odd number of backslashes, or quotes after even number.
          pattern = /(?<!\\)(\\\\)*(\\#{Regexp.escape(start_delim[-1])}|\\#{Regexp.escape(end_delim)}|")/

          (string_contents[1..-1] || []).each do |part|
            next if part.nil?
            case part[0]
            when :@tstring_content
              part[1] = part[1].gsub(pattern) do |str|
                if str.end_with?('"')
                  # insert needed escape
                  "#{str[0..-2]}\\\""
                else
                  # drop unnecessary escape
                  "#{str[0..-3]}#{str[-1]}"
                end
              end
            when :string_embexpr, :string_dvar
            else
              raise "got #{part[0]} in a #{start_delim}...#{end_delim} string"
            end
          end
        end
      end
  end
end
