$: << "./native"
$: << "./build"
require "rubyfmt.so"
require "rubyfmt"

i = Intermediary.new
10000.times { i << DirectPart.new("a") }
s = Time.now.to_f
Rubyfmt::write_intermediary("foo.txt", i)
e = Time.now.to_f
p(e - s)
i = Intermediary.new
10000.times { i << DirectPart.new("a") }
fp = File.open("hi.txt", "w")
s = Time.now.to_f
i.each { |x| fp.write(x) }
e = Time.now.to_f
p(e - s)
