def extract_line_metadata(file_data)
  comment_blocks = {}

  file_data.split("\n").each_with_index do |line, index|
    comment_blocks[index + 1] = line if /^ *#/ === line
  end

  LineMetadata.new(comment_blocks)
end

def main
  if ARGV.first == "-i"
    output = StringIO.new
    inline = true
    file_to_read = File.open(ARGV[1], "r")
  else
    output = $stdout
    inline = false
    file_to_read = ARGF
  end

  file_data = file_to_read.read
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
    STDERR.puts "Got a parse error while reading #{file_to_read.to_io.inspect}"
    STDERR.puts parser.error?.inspect
    STDERR.puts "bailing with exit code 1"
    exit 1
  end

  parser.comments_delete.each do |(start, last)|
    line_metadata.comment_blocks.reject! { |k, v| k >= start && k <= last }
  end

  format_program(line_metadata, sexp, output)

  if inline
    output.rewind
    File.open(ARGV[1], "w").write(output.read)
  end
end

main if __FILE__ == $0
