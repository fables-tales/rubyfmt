A::Constant::With::A::Singleton::Method.that_calls_another_method(
  formatted: true,
  onto: 0,
  multiple: "lines"
)[0]

A::Constant::With::A::Singleton::Method.that_calls_another_method(
  formatted: true,
  onto: 0,
  multiple: "lines"
)[0].another_call

foo.bar[0].baz

Array[1, 2, 3].map { |x| x * 2 }

matrix[0][1].to_s

A::Constant::With::A::Singleton::Method.that_calls_another_method(
  formatted: true,
  onto: 0,
  multiple: "lines"
)[0].second_call[1].third_call

foo&.bar(
  some: "args"
)[0].baz

hash.fetch(
  key,
  default_value
)[0].process
