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
    @file_lines = file_data.split("\n")

    @lines_with_any_ruby = {}

    # heredoc stack is the stack of identified heredocs
    @heredoc_stack = []

    # next_heredoc_stack is the type identifiers of the next heredocs, that
    # we haven't emitted yet
    @next_heredoc_stack = []
    @heredoc_regex = /(<<[-~]?)(.*$)/
    @regexp_stack = []
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
    @tlambda_stack = []
    @array_location_stacks = []
    @rbracket_stack = []
    @lbrace_stack = []
    @comments = {}
    @last_ln = 0
  end

  def on_nl(*args)
    @last_ln = lineno+1
    super(*args)
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
      [res, @comments, @lines_with_any_ruby, @last_ln]
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
      parts.tap do |node|
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
        node[1] << part
      end
      super(parts, part)
    end
  end

  def on_next(*_args)
    super + [@kw_stacks['next'].pop]
  end

  def on_if(*_args)
    start_line = @kw_stacks['if'].pop.first
    end_line = lineno
    super + [[start_line, end_line]]
  end

  def on_unless(*_args)
    start_line = @kw_stacks['unless'].pop.first
    end_line = lineno
    super + [[start_line, end_line]]
  end

  def on_else(*_args)
    start_line = @kw_stacks['else'].pop.first
    end_line = lineno
    super + [[start_line, end_line]]
  end

  def on_elsif(*_args)
    start_line = @kw_stacks['elsif'].pop.first
    end_line = lineno
    super + [[start_line, end_line]]
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
    start_line = @kw_stacks["do"].pop.first
    end_line = lineno
    super + [[start_line, end_line]]
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
    super + [[@kw_stacks["while"].pop.first, lineno]]
  end

  def on_until(*args)
    super + [[@kw_stacks["until"].pop.first, lineno]]
  end

  def on_hash(assocs)
    [:hash, assocs, [@lbrace_stack.pop, lineno]]
  end

  def on_zsuper
    # ripper doesn't handle on_zsuper correctly.
    # however `on_kw` catches zsuper, so use that!
    [:zsuper, @kw_stacks["super"].pop]
  end

  def on_yield0
    # ripper doesn't handle on_yield0 correctly.
    # however `on_kw` catches yield0, so use that!
    [:yield0, @kw_stacks["yield"].pop]
  end

  def on_redo
    # ripper doesn't handle on_redo correctly.
    # however `on_kw` catches redo, so use that!
    [:redo, @kw_stacks["redo"].pop]
  end

  def on_begin(*args)
    beg, statements = super
    [beg, [@kw_stacks["begin"].pop.first, lineno], statements]
  end

  def on_rescue(*args)
    super + [[@kw_stacks["rescue"].pop.first, lineno]]
  end

  def on_ensure(*args)
    super + [[@kw_stacks["ensure"].pop.first, lineno]]
  end

  def on_retry
    # ripper doesn't handle on_retry correctly.
    # however `on_kw` catches retry, so use that!
    [:retry, @kw_stacks["retry"].pop]
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
    # The lineno here is actually one line *after*
    # the line of the bracket, so we manually trace
    # the line of the rbracket instead
    super + [[@rbracket_stack.pop, column]]
  end

  def on_aref_field(*_args)
    # The lineno here is actually one line *after*
    # the line of the bracket, so we manually trace
    # the line of the rbracket instead
    super + [[@rbracket_stack.pop, column]]
  end

  def on_kw(kw)
    if stack = @kw_stacks[kw]
      stack << [lineno, column]
    end
    super
  end

  def on_super(args)
    [:super, args, @kw_stacks["super"].pop]
  end

  def on_return(args)
    [:return, args, @kw_stacks["return"].pop]
  end

  def on_return0
    [:return0, @kw_stacks["return"].pop]
  end

  def on_when(cond, body, tail)
    [:when, cond, body, tail, @kw_stacks["when"].pop]
  end

  def on_case(cond, body)
    current_line = lineno
    [:case, cond, body, [@kw_stacks["case"].pop.first, current_line]]
  end

  def on_yield(arg)
    [:yield, arg, @kw_stacks["yield"].pop]
  end

  def on_break(arg)
    [:break, arg, @kw_stacks["break"].pop]
  end

  def on_tlambda(*args)
    @tlambda_stack << [lineno, column]
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

  def on_string_literal(*args, &blk)
    if @heredoc_stack.last
      heredoc_parts = @heredoc_stack.pop
      args.insert(0, [:heredoc_string_literal] + heredoc_parts)
    else
      end_delim, end_line = @string_stack.pop
      start_delim, start_line = @string_stack.pop

      args << [start_line, end_line]

      if start_delim && end_delim && start_delim != "\""
        if start_delim == "'" || start_delim.start_with?("%q")
          # re-evaluate the string with its own quotes to handle escaping.
          if args[0][1]
            args[0][1][1] = eval("#{start_delim}#{args[0][1][1]}#{end_delim}").inspect[1..-2]
          end
        else
          # find delimiters after an odd number of backslashes, or quotes after even number.
          pattern = /(?<!\\)(\\\\)*(\\#{Regexp.escape(start_delim[-1])}|\\#{Regexp.escape(end_delim)}|")/

          (args[0][1..-1] || []).each do |part|
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

    super
  end

  def on_lambda(*args, &blk)
    terminator = @file_lines[lineno - 1]

    if terminator.include?("}")
      args.insert(1, :curly)
    else
      args.insert(1, :do)
    end
    args.insert(args.length, @tlambda_stack.pop)
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

  def on_dyna_symbol(*args)
    # dyna_symbol expressions still end up calling
    # on_tstring_end, which will append the closing
    # quote to @string_stack. We want to ignore this,
    # so remove it from the stack.
    start_line = @string_stack.pop[1]
    super + [[start_line, lineno]]
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

  private def with_lineno(&blk)
    start_line = lineno
    res = yield
    end_line = lineno
    res + [[start_line, end_line]]
  end
end

GC.disable
