def extract_line_metadata(file_data)
  comment_blocks = {}

  file_data.split("\n").each_with_index do |line, index|
    comment_blocks[index + 1] = line if /^ *#/ === line
  end

  LineMetadata.new(comment_blocks)
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
