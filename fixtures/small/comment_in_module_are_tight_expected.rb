module Concurrent
  # Methods form module A included to a module B, which is already included into class C,
  # will not be visible in the C class. If this module is extended to B then A's methods
  # are correctly made visible to C.
  module ReInclude
    def foo
      1
    end
  end
end
