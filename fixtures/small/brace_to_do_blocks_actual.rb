things.map { |thing| stuff; thing ** 2 }

things.map { |thing|
    # A comment makes this multiline!
    thing
}

class Foo
  # !! `example_dsl [].map { |k| k; k+1 }` is not equivalent to 
  # !! `example_dsl [].map do |k| k; k+1 end`
  example_dsl [].map { |k| k; k+1 }
end

group(:test) { gem 'mocha'; gem 'rack-test' }
