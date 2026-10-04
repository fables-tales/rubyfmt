(class << object; self; end).foo

Module.new { def foo; end }.foo

(begin; self; end).foo

(class Foo; end).nil?

(if cond; value; end).foo

ALL = T.let(
  [RedispatchStatus::Final, RedispatchStatus::Provisional, RedispatchStatus::Pending].map { |status|
    status.serialize.to_sym
  }.freeze,
  T::Array[Symbol]
)

foo.map { |x|
  x + 1
}.freeze
