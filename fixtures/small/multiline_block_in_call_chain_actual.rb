some_array.map { |x| a; b }.select {  }

some_array.map { |x|
  a
  b
}.select { }

foo.bar.baz { |x| a; b }

foo.bar do |x| a; b end.select { }

foo.items.map { p(_1) }.each { _1.call! }

foo.items.map { p(_1) }.last

hashes.sort_by { |hsh| hsh[:start_time] }.reverse

-> { a; b }.bar.baz
