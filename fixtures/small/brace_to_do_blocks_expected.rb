things.map do |thing|
  stuff
  thing ** 2
end

things.map do |thing|
  # A comment makes this multiline!
  thing
end

class Foo
  # !! `example_dsl [].map { |k| k; k+1 }` is not equivalent to
  # !! `example_dsl [].map do |k| k; k+1 end`
  example_dsl(
    [].map do |k|
      k
      k + 1
    end
  )
end
