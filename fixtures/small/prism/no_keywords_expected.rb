def foo(**nil)
end

def bar(positional, another_positional, **nil)
end

def baz(
  something,
  # No keywords!
  **nil
)
end
