$: << File.dirname(__FILE__)
require "rubyfmt.so"
require "ripper"
require "json"

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
      "yield" => [],
      "break" => [],
    }
    @array_location_stacks = []
  end

  attr_reader :comments_delete

  private


  UNESCAPED_SQUARE_BRACKET=/(?<!\\)((?:\\\\)*[\[\]])/.freeze # square bracket after even number of backslashes (including zero)
  ARRAY_SYMBOLS.each do |event, symbol|
    define_method(:"on_#{event}_new") do
      [event, [], [lineno, column]]
    end

    define_method(:"on_#{event}_add") do |parts, part|
      parts.tap do |node|
        if part[0].is_a?(Array)
          part.each do |sub_part|
            sub_part[1].gsub!(UNESCAPED_SQUARE_BRACKET, "\\\\\\1") if sub_part[0] == :@tstring_content
          end
        else
          part[1].gsub!(UNESCAPED_SQUARE_BRACKET, "\\\\\\1") if part[0] == :@tstring_content
        end
        node[1] << part
      end
    end
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

  def on_return(args)
    [:return, args, @kw_stacks["return"].pop]
  end

  def on_when(cond, body, tail)
    [:when, cond, body, tail, @kw_stacks["when"].pop]
  end

  def on_yield(arg)
    [:yield, arg, @kw_stacks["yield"].pop]
  end

  def on_break(arg)
    [:break, arg, @kw_stacks["break"].pop]
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
end

file_data = File.read(ARGV[0])
parsed = Parser.new(file_data).parse
inspected_parsed = JSON.dump(parsed)
Rubyfmt::format_to_stdout(file_data, inspected_parsed)
STDOUT.close
