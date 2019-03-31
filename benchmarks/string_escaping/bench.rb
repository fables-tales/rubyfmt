require 'benchmark/ips'
require 'ripper'

$: << File.dirname(__FILE__)

require 'parser_eval'
require 'parser_munging'

f1 = File.read("#{File.dirname(__FILE__)}/string_literals_stress_test.rb")
f2 = File.read("#{File.dirname(__FILE__)}/rspec_core_notifications_actual.rb")

Benchmark.ips do |x|
  # Configure the number of seconds used during
  # the warmup phase (default 2) and calculation phase (default 5)
  x.config(:time => 10, :warmup => 4)

  # Typical mode, runs the block as many times as it can
  x.report("munging") {
    ParserMunging.new(f1).parse
  }

  x.report("eval") {
    ParserEval.new(f1).parse
  }

  x.report("munging 2") {
    ParserMunging.new(f2).parse
  }

  x.report("eval 2") {
    ParserEval.new(f2).parse
  }
end
