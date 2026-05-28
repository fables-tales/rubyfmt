def a(...) = raise(...)

def b(...) = raise(self, ...)

def c(...)
  raise(ArgumentError, ...)
end

class Error < StandardError
  def self.call(...) = raise(self, ...)
end
