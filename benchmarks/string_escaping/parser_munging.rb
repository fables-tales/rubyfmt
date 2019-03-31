class ParserMunging < Ripper::SexpBuilderPP
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


  FOUR_SLASHES_IN_TSTRING = "\\\\\\\\"
  LITERAL_DOUBLE_QUOTE_IN_TSTRING = "\""
  TWO_SLASHES_IN_TSTRING = "\\\\"
  ONE_SLASH_IN_TSTRING = "\\"

  # Rubyfmt renders all string literals as double quotes for consistency's sake
  # this method fixes up individual tstring content items to ensure that they
  # represent the same string literal once reformatted. This method is not
  # called on string literals that were already surrounded by double quotes,
  # and so this formatting only applies to tstring_contents from strings that
  # need to be manipulated to correctly reform as a double quoted string
  #
  def fixup_tstring_content_for_double_quotes(string, is_single_quote:, delimiter:)
    # cleanup escaped delimiters
    string.gsub!("#{ONE_SLASH_IN_TSTRING}#{delimiter}", "#{delimiter}")

    # Given '\\a' we get a tstring content with "\\\\a", and we need to keep
    # it the same. However, the next substitution line will replace "\\\\a"
    # with "\\\\\\\\\\a", so we insert "__RUBYFMT_SAFE_QUAD" as a placeholder
    # to sub back with quad slashes.
    string.gsub!(TWO_SLASHES_IN_TSTRING, "__RUBYFMT_SAFE_QUAD")

    # Given '\a' we get a tstring content with "\\a", and we need to replace it
    # with literally "\\\\a", which needs four escaped slashes
    string.gsub!(
      ONE_SLASH_IN_TSTRING,
      FOUR_SLASHES_IN_TSTRING,
    )

    if is_single_quote
      # Given '"', we get a tstring content with "\"", however we need to
      # literally serialize that slash back out, so we replace it with "\\\""
      string.gsub!(
        LITERAL_DOUBLE_QUOTE_IN_TSTRING,
        "#{ONE_SLASH_IN_TSTRING}#{LITERAL_DOUBLE_QUOTE_IN_TSTRING}",
      )
    else
      # deals with e.g. %^\"^ which should format to "\"" (that is, a
      # literal quote)
      string.gsub!(
        "#{TWO_SLASHES_IN_TSTRING}#{LITERAL_DOUBLE_QUOTE_IN_TSTRING}",
        "__RUBYFMT_PLEASE_SERIALIZE_THIS_TO_A_LITERAL_DOUBLE_QUOTE",
      )

      # deals with e.g. %^\\"^ which should format to "\\\"" (that is, an
      # escaped double quote)
      string.gsub!(
        "__RUBYFMT_SAFE_QUAD\"",
        "__RUBYFMT_PLEASE_SERIALIZE_THIS_TO_AN_ESACPED_DOUBLE_QUOTE"
      )

      # deals with bare double quotes, replacing them with \"
      string.gsub!(
        LITERAL_DOUBLE_QUOTE_IN_TSTRING,
        "#{ONE_SLASH_IN_TSTRING}#{LITERAL_DOUBLE_QUOTE_IN_TSTRING}",
      )

      # undoes the literal double quote replacement
      string.gsub!(
        "__RUBYFMT_PLEASE_SERIALIZE_THIS_TO_A_LITERAL_DOUBLE_QUOTE",
        "#{ONE_SLASH_IN_TSTRING}#{LITERAL_DOUBLE_QUOTE_IN_TSTRING}",
      )

      # undoes the escaped double quote replacement
      string.gsub!(
        "__RUBYFMT_PLEASE_SERIALIZE_THIS_TO_AN_ESACPED_DOUBLE_QUOTE",
        "#{FOUR_SLASHES_IN_TSTRING}#{ONE_SLASH_IN_TSTRING}#{LITERAL_DOUBLE_QUOTE_IN_TSTRING}",
      )
    end

    # Fixup the quad safes
    string.gsub!(
      "__RUBYFMT_SAFE_QUAD",
      FOUR_SLASHES_IN_TSTRING,
    )

    # protect against escaped interpolation
    string.gsub!("\#{", "\\\#{")
    string.gsub!("\#@", "\\\#@")
  end

  def on_string_literal(*args, &blk)
    if @heredoc_stack.last
      heredoc_parts = @heredoc_stack.pop
      args.insert(0, [:heredoc_string_literal, heredoc_parts])
    else
      next_string_end_delim = [@file_lines[lineno-1].bytes[column-1]].pack("c*")
      if next_string_end_delim == "'"
        (args || []).each do |part|
          next if part[1].nil?
          case part[1][0]
          when :@tstring_content
            fixup_tstring_content_for_double_quotes(
              part[1][1],
              is_single_quote: true,
              delimiter: next_string_end_delim,
            )
          else
            raise "got non tstring content in single string"
          end
        end
      elsif /[^a-zA-Z0-9]/ === next_string_end_delim && next_string_end_delim != "\""
        next_string_start_delim = @string_stack.pop
        is_single_q_string = next_string_start_delim.start_with?("%q")
        (args[0][1..-1] || []).each do |part|
          next if part.nil?
          case part[0]
          when :@tstring_content
            fixup_tstring_content_for_double_quotes(
              part[1],
              is_single_quote: is_single_q_string,
              delimiter: next_string_end_delim,
            )
          when :string_embexpr
            # this is fine
          else
            raise "got something bad in %q string"
          end
        end
      elsif next_string_end_delim == "\""
      else
        raise "what even is this string type #{next_string_end_delim}"
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

  def on_regexp_beg(re_part)
    @regexp_stack << re_part
  end

  def on_regexp_literal(*args)
    args[1] << @regexp_stack.pop
    super(*args)
  end
end
