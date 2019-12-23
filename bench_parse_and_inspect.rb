$: << "."
require "build/rubyfmt"


def main
  file_data = File.read("fixtures/rspec_core_notifications_actual.rb")
  50.times {
    parser = Parser.new(file_data)
    sexp = parser.parse
    sexp.inspect
  }
  s = Time.now.to_f
  1000.times {
    parser = Parser.new(file_data)
    sexp = parser.parse
    sexp.inspect
  }
  e = Time.now.to_f
  p((e - s) / 1000.0)
end

if __FILE__ == $0
  main
end
