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
    @kw_stacks = {
      "return" => [],
      "when" => [],
      "case" => [],
      "yield" => [],
      "break" => [],
      "super" => [],
    }
    @tlambda_stack = []
    @array_location_stacks = []
    @lbrace_stack = []
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

  def on_lbrace(*args)
    @lbrace_stack << [lineno, column]
  end

  def on_brace_block(params, body)
    @lbrace_stack.pop
    super
  end

  def on_hash(assocs)
    [:hash, assocs, @lbrace_stack.pop]
  end

  def on_lbracket(*args)
    @array_location_stacks << [lineno, column]
  end

  def on_array(*args)
    res = super
    res << @array_location_stacks.pop
    res
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

  def on_when(cond, body, tail)
    [:when, cond, body, tail, @kw_stacks["when"].pop]
  end

  def on_case(cond, body)
    [:case, cond, body, @kw_stacks["case"].pop]
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
end

GC.disable
