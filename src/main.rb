def extract_line_metadata(file_data)
  comment_blocks = {}

  file_data.split("\n").each_with_index do |line, index|
    comment_blocks[index + 1] = line if /^ *#/ === line
  end

  LineMetadata.new(comment_blocks)
end

def main
  if ARGV.first == "-i"
    output_proc = Proc.new { File.open(ARGV[1], "w") }
    file_to_read = File.open(ARGV[1], "r")
  else
    output_proc = Proc.new { $stdout }
    file_to_read = ARGF
  end

  file_data = file_to_read.read
  file_data = file_data.gsub("\r\n", "\n")
  line_metadata = extract_line_metadata(file_data)
  parser = Parser.new(file_data)
  sexp = parser.parse

  if ENV["RUBYFMT_DEBUG"] == "2"
    require "pry"

    binding.pry
  end

  if parser.error?
    if ENV["RUBYFMT_DEBUG"] == "2"
      require "pry"

      binding.pry
    end

    exit(1)
  end

  parser.comments_delete.each do |(start, last)|
    line_metadata.comment_blocks.reject! { |k, v| k >= start && k <= last }
  end

  format_program(line_metadata, sexp, &output_proc)
end

def rubyprof_main
  require "ruby-prof"

  RubyProf.start
  main
  result = RubyProf.stop
  RubyProf::CallStackPrinter.new(result).print(File.open("out.html", "w"))
end

main if __FILE__ == $0
