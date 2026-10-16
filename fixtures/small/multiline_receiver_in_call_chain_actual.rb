(class << object; self; end).foo

Module.new { def foo; end }.foo
