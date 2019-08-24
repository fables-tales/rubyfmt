$: << "."
$: << ".."
require "rubyfmt.so"
require "build/rubyfmt"
require "json"
require "pp"


#start_time = Time.now.to_f
#100.times {
#  file_data = File.read("../fixtures/rspec_core_notifications_actual.rb")
#  inspected_parsed = JSON.dump(Parser.new(file_data).parse)
#  Rubyfmt::format_to_stdout(file_data, inspected_parsed)
#}
#end_time = Time.now.to_f
#p((end_time - start_time) / 100)
#
#
file_data = File.read(ARGV[0])
parsed = Parser.new(file_data).parse
pp(parsed)
inspected_parsed = JSON.dump(parsed)
Rubyfmt::format_to_stdout(file_data, inspected_parsed)
